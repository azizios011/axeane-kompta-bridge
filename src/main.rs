#![windows_subsystem = "windows"]

mod app_state;
mod bin {
    pub mod browser_backend;
    pub mod llm_backend;
}

use std::{
    collections::HashMap,
    net::TcpStream as StdTcpStream,
    path::PathBuf,
    process::{Child, Command},
    sync::{Arc, Mutex},
    time::Duration,
};

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
};
use wry::webview::WebViewBuilder;

use app_state::{initial_state, SharedAppState};
use bin::browser_backend::{compile_payload_from_state, build_injection_script};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener as HttpListener;

const FRONTEND_URL: &str = "http://127.0.0.1:3000";

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(Mutex::new(initial_state()));

    std::thread::spawn({
        let state = state.clone();
        move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                let s1 = state.clone();
                let s2 = state.clone();
                tokio::join!(
                    spawn_websocket_bridge(s1),
                    spawn_http_server(s2),
                );
            });
        }
    });

    let exe_path = std::env::current_exe()?;
    let exe_dir = exe_path.parent().expect("exe has no parent dir");
    let frontend_dir = exe_dir.join("frontend");

    println!("Starting Next.js server from: {}", frontend_dir.display());
    let frontend_process = launch_frontend(&frontend_dir)?;

    wait_for_frontend();

    println!("Axeane frontend is running at {}", FRONTEND_URL);
    println!("WebSocket bridge   -> ws://127.0.0.1:8085");
    println!("HTTP API server    -> http://127.0.0.1:8086");
    println!("Opening native UI window...");

    open_frontend_window(frontend_process);

    Ok(())
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocket bridge — bidirectional: reads commands FROM frontend, sends JS TO browser
// ─────────────────────────────────────────────────────────────────────────────
async fn spawn_websocket_bridge(state: Arc<Mutex<SharedAppState>>) {
    let listener = TcpListener::bind("127.0.0.1:8085")
        .await
        .expect("failed to bind WebSocket bridge on 127.0.0.1:8085");

    while let Ok((stream, _)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(async move {
            if let Ok(mut ws) = accept_async(stream).await {
                {
                    let mut s = state.lock().unwrap();
                    s.status = "Browser extension paired via WebSocket.".to_string();
                }

                loop {
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_millis(300)) => {
                            let (auto_detect, trigger, rows, templates) = {
                                let s = state.lock().unwrap();
                                (s.auto_detect_active, s.trigger_injection, s.csv_rows.clone(), s.templates.clone())
                            };

                            if auto_detect {
                                {
                                    let mut s = state.lock().unwrap();
                                    s.status = "Scanning browser DOM for Axeane elements...".to_string();
                                }
                                let script = bin::browser_backend::compile_page_detection_script();
                                let _ = ws.send(Message::Text(format!("EVAL_REQUEST:{}", script))).await;
                            }

                            if trigger {
                                let json_payload = compile_payload_from_state(&rows, &templates);
                                if json_payload == "[]" {
                                    let mut s = state.lock().unwrap();
                                    s.status = "Warning: No rows to inject. Import a PDF first.".to_string();
                                    s.trigger_injection = false;
                                    continue;
                                }
                                let journal = { let s = state.lock().unwrap(); s.journal_code.clone() };
                                let injection = build_injection_script(&json_payload, &journal);
                                let _ = ws.send(Message::Text(injection)).await;
                                {
                                    let mut s = state.lock().unwrap();
                                    s.trigger_injection = false;
                                    s.status = "Injection dispatched to browser.".to_string();
                                }
                            }
                        }

                        msg = ws.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    handle_incoming_message(&state, &text);
                                }
                                Some(Ok(Message::Close(_))) | None => break,
                                _ => {}
                            }
                        }
                    }
                }

                let mut s = state.lock().unwrap();
                s.status = "Browser extension disconnected.".to_string();
            }
        });
    }
}

fn handle_incoming_message(state: &Arc<Mutex<SharedAppState>>, text: &str) {
    if text == "TRIGGER_INJECTION" {
        let mut s = state.lock().unwrap();
        if s.csv_rows.is_empty() {
            s.status = "Error: No rows loaded. Import a PDF first.".to_string();
        } else {
            s.trigger_injection = true;
            s.status = "Injection triggered by browser extension.".to_string();
        }
    } else if text == "START_AUTO_DETECT" {
        let mut s = state.lock().unwrap();
        s.auto_detect_active = true;
        s.status = "Auto-detection armed.".to_string();
    } else if let Some(code) = text.strip_prefix("SET_JOURNAL:") {
        let mut s = state.lock().unwrap();
        s.journal_code = code.trim().to_string();
        s.status = format!("Journal set to: {}", code.trim());
    } else if text == "STOP_AUTO_DETECT" {
        let mut s = state.lock().unwrap();
        s.auto_detect_active = false;
        s.status = "Auto-detection stopped.".to_string();
    } else if text.starts_with("EVAL_RESULT:") {
        let result = &text["EVAL_RESULT:".len()..];
        let mut s = state.lock().unwrap();
        if result == "READY" {
            s.status = "Axeane form detected! Ready to inject.".to_string();
            s.auto_detect_active = false;
            if !s.csv_rows.is_empty() {
                s.trigger_injection = true;
            }
        } else if result == "WRONG_PAGE" {
            s.status = "Wrong page — navigate to the Axeane ecriture form.".to_string();
        } else if result == "NO_FORM" {
            s.status = "Form not found on current page.".to_string();
        }
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// HTTP server — handles POST /api/import-pdf from the frontend
// ─────────────────────────────────────────────────────────────────────────────
async fn spawn_http_server(state: Arc<Mutex<SharedAppState>>) {
    let listener = HttpListener::bind("127.0.0.1:8086")
        .await
        .expect("failed to bind HTTP server on 127.0.0.1:8086");

    while let Ok((mut stream, _)) = listener.accept().await {
        let state = state.clone();
        tokio::spawn(async move {
            let mut reader = BufReader::new(&mut stream);

            let mut request_line = String::new();
            let _ = reader.read_line(&mut request_line).await;

            let mut headers = HashMap::new();
            loop {
                let mut line = String::new();
                let _ = reader.read_line(&mut line).await;
                let line = line.trim().to_string();
                if line.is_empty() { break; }
                if let Some((k, v)) = line.split_once(": ") {
                    headers.insert(k.to_lowercase(), v.to_string());
                }
            }

            let is_post_pdf = request_line.trim().starts_with("POST /api/import-pdf");
            if !is_post_pdf {
                let _ = stream.write_all(b"HTTP/1.1 404 Not Found\r\nContent-Length: 0\r\n\r\n").await;
                return;
            }

            let content_length: usize = headers
                .get("content-length")
                .and_then(|v| v.parse().ok())
                .unwrap_or(0);

            let mut body = vec![0u8; content_length];
            let _ = reader.read_exact(&mut body).await;

            let boundary = headers
                .get("content-type")
                .and_then(|ct| ct.split("boundary=").nth(1))
                .map(|b| b.trim().to_string());

            let response = match boundary {
                None => cors_json_response(400, r#"{"error":"Missing multipart boundary"}"#),
                Some(boundary) => {
                    match extract_pdf_from_multipart(&body, &boundary) {
                        None => cors_json_response(400, r#"{"error":"Could not extract PDF from form data"}"#),
                        Some(pdf_bytes) => {
                            let llm_config = {
                                let s = state.lock().unwrap();
                                s.llm.clone()
                            };

                            match bin::llm_backend::rip_pdf_text_from_bytes(&pdf_bytes) {
                                Err(e) => cors_json_response(500, &format!(r#"{{"error":"PDF extraction failed: {}"}}"#, e)),
                                Ok(raw_text) => {
                                    match bin::llm_backend::parse_pdf_via_ai(&raw_text, &llm_config).await {
                                        Err(e) => cors_json_response(500, &format!(r#"{{"error":"LLM parsing failed: {}"}}"#, e)),
                                        Ok(rows) => {
                                            let row_count = rows.len();
                                            {
                                                let mut s = state.lock().unwrap();
                                                s.csv_rows.extend(rows.clone());
                                                s.status = format!("Imported {} rows from PDF via AI.", row_count);
                                            }
                                            let rows_json = serde_json::to_string(&rows).unwrap_or_else(|_| "[]".to_string());
                                            cors_json_response(200, &format!(r#"{{"rows":{}}}"#, rows_json))
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            };

            let _ = stream.write_all(response.as_bytes()).await;
        });
    }
}

fn cors_json_response(status: u16, body: &str) -> String {
    let reason = if status == 200 { "OK" } else if status == 400 { "Bad Request" } else { "Internal Server Error" };
    format!(
        "HTTP/1.1 {} {}\r\nContent-Type: application/json\r\nAccess-Control-Allow-Origin: *\r\nContent-Length: {}\r\n\r\n{}",
        status, reason, body.len(), body
    )
}

fn extract_pdf_from_multipart(body: &[u8], boundary: &str) -> Option<Vec<u8>> {
    let delimiter = format!("--{}", boundary).into_bytes();
    let header_end = b"\r\n\r\n";

    let start = body.windows(delimiter.len())
        .position(|w| w == delimiter.as_slice())?;
    let after_boundary = start + delimiter.len() + 2;

    let header_end_pos = body[after_boundary..]
        .windows(header_end.len())
        .position(|w| w == header_end)?;
    let data_start = after_boundary + header_end_pos + header_end.len();

    let closing = format!("\r\n--{}--", boundary).into_bytes();
    let data_end = body[data_start..]
        .windows(closing.len())
        .position(|w| w == closing.as_slice())
        .map(|p| data_start + p)
        .unwrap_or(body.len());

    Some(body[data_start..data_end].to_vec())
}

fn launch_frontend(frontend_dir: &PathBuf) -> std::io::Result<Child> {
    let script = if frontend_dir.join(".next").exists() { "start" } else { "dev" };
    Command::new("npm.cmd")
        .args(["run", script, "--", "--hostname", "127.0.0.1", "--port", "3000"])
        .current_dir(frontend_dir)
        .spawn()
}

fn wait_for_frontend() {
    for _ in 0..60 {
        if StdTcpStream::connect("127.0.0.1:3000").is_ok() {
            return;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
}

fn open_frontend_window(mut frontend_process: Child) {
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Axeane Automation Bridge")
        .with_inner_size(tao::dpi::LogicalSize::new(1024.0, 768.0))
        .build(&event_loop)
        .unwrap();

    let _webview = WebViewBuilder::new(window)
        .unwrap()
        .with_url(FRONTEND_URL)
        .unwrap()
        .build()
        .unwrap();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        if let Event::WindowEvent { event: WindowEvent::CloseRequested, .. } = event {
            let _ = frontend_process.kill();
            *control_flow = ControlFlow::Exit;
        }
    });
}
