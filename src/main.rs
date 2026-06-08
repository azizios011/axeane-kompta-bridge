mod app_state;
mod bin {
    pub mod browser_backend;
    pub mod llm_backend;
}

use std::{
    sync::{Arc, Mutex},
    time::Duration,
};

use tauri::Manager;

use app_state::{initial_state, SharedAppState};
use bin::browser_backend::{compile_payload_from_state, build_injection_script};
use futures_util::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

// ─────────────────────────────────────────────────────────────────────────────
// Tauri commands — called from the frontend via invoke()
// ─────────────────────────────────────────────────────────────────────────────

#[tauri::command]
async fn import_pdf(
    bytes: Vec<u8>,
    state: tauri::State<'_, Arc<Mutex<SharedAppState>>>,
    app: tauri::AppHandle,
) -> Result<Vec<app_state::EditableRow>, String> {
    let llm_config = {
        let s = state.lock().unwrap();
        s.llm.clone()
    };

    let raw_text = bin::llm_backend::rip_pdf_text_from_bytes(&bytes)
        .map_err(|e| e.to_string())?;

    let rows = bin::llm_backend::parse_pdf_via_ai(&raw_text, &llm_config)
        .await
        .map_err(|e| format!("LLM parsing failed: {}", e))?;

    let row_count = rows.len();
    {
        let mut s = state.lock().unwrap();
        s.csv_rows.extend(rows.clone());
        s.status = format!("Imported {} rows from PDF via AI.", row_count);
    }

    emit_status(&state, &app);
    Ok(rows)
}

#[tauri::command]
fn trigger_injection(
    state: tauri::State<'_, Arc<Mutex<SharedAppState>>>,
    app: tauri::AppHandle,
) {
    let mut s = state.lock().unwrap();
    if s.csv_rows.is_empty() {
        s.status = "Error: No rows loaded. Import a PDF first.".to_string();
    } else {
        s.trigger_injection = true;
        s.status = "Injection triggered.".to_string();
    }
    emit_status(&state, &app);
}

#[tauri::command]
fn start_auto_detect(
    state: tauri::State<'_, Arc<Mutex<SharedAppState>>>,
    app: tauri::AppHandle,
) {
    let mut s = state.lock().unwrap();
    s.auto_detect_active = true;
    s.status = "Auto-detection armed.".to_string();
    emit_status(&state, &app);
}

#[tauri::command]
fn stop_auto_detect(
    state: tauri::State<'_, Arc<Mutex<SharedAppState>>>,
    app: tauri::AppHandle,
) {
    let mut s = state.lock().unwrap();
    s.auto_detect_active = false;
    s.status = "Auto-detection stopped.".to_string();
    emit_status(&state, &app);
}

#[tauri::command]
fn set_journal(
    code: String,
    state: tauri::State<'_, Arc<Mutex<SharedAppState>>>,
    app: tauri::AppHandle,
) {
    let mut s = state.lock().unwrap();
    s.journal_code = code.trim().to_string();
    s.status = format!("Journal set to: {}", code.trim());
    emit_status(&state, &app);
}

fn emit_status(state: &Arc<Mutex<SharedAppState>>, app: &tauri::AppHandle) {
    let status = { state.lock().unwrap().status.clone() };
    let _ = app.emit_all("status-update", &status);
}

// ─────────────────────────────────────────────────────────────────────────────
// WebSocket bridge — background task for browser extension communication
// ─────────────────────────────────────────────────────────────────────────────

async fn spawn_websocket_bridge(state: Arc<Mutex<SharedAppState>>, app: tauri::AppHandle) {
    let listener = TcpListener::bind("127.0.0.1:8085")
        .await
        .expect("failed to bind WebSocket bridge on 127.0.0.1:8085");

    while let Ok((stream, _)) = listener.accept().await {
        let state = state.clone();
        let app = app.clone();
        tokio::spawn(async move {
            if let Ok(mut ws) = accept_async(stream).await {
                {
                    let mut s = state.lock().unwrap();
                    s.status = "Browser extension paired via WebSocket.".to_string();
                    emit_status(&state, &app);
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
                                    emit_status(&state, &app);
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
                                    emit_status(&state, &app);
                                    continue;
                                }
                                let journal = { let s = state.lock().unwrap(); s.journal_code.clone() };
                                let injection = build_injection_script(&json_payload, &journal);
                                let _ = ws.send(Message::Text(injection)).await;
                                {
                                    let mut s = state.lock().unwrap();
                                    s.trigger_injection = false;
                                    s.status = "Injection dispatched to browser.".to_string();
                                    emit_status(&state, &app);
                                }
                            }
                        }

                        msg = ws.next() => {
                            match msg {
                                Some(Ok(Message::Text(text))) => {
                                    handle_incoming_message(&state, &app, &text);
                                }
                                Some(Ok(Message::Close(_))) | None => break,
                                _ => {}
                            }
                        }
                    }
                }

                let mut s = state.lock().unwrap();
                s.status = "Browser extension disconnected.".to_string();
                emit_status(&state, &app);
            }
        });
    }
}

fn handle_incoming_message(
    state: &Arc<Mutex<SharedAppState>>,
    app: &tauri::AppHandle,
    text: &str,
) {
    if text == "TRIGGER_INJECTION" {
        let mut s = state.lock().unwrap();
        if s.csv_rows.is_empty() {
            s.status = "Error: No rows loaded. Import a PDF first.".to_string();
        } else {
            s.trigger_injection = true;
            s.status = "Injection triggered by browser extension.".to_string();
        }
        emit_status(state, app);
    } else if text == "START_AUTO_DETECT" {
        let mut s = state.lock().unwrap();
        s.auto_detect_active = true;
        s.status = "Auto-detection armed.".to_string();
        emit_status(state, app);
    } else if let Some(code) = text.strip_prefix("SET_JOURNAL:") {
        let mut s = state.lock().unwrap();
        s.journal_code = code.trim().to_string();
        s.status = format!("Journal set to: {}", code.trim());
        emit_status(state, app);
    } else if text == "STOP_AUTO_DETECT" {
        let mut s = state.lock().unwrap();
        s.auto_detect_active = false;
        s.status = "Auto-detection stopped.".to_string();
        emit_status(state, app);
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
        emit_status(state, app);
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Entry point
// ─────────────────────────────────────────────────────────────────────────────

fn main() {
    let state = Arc::new(Mutex::new(initial_state()));
    let state_clone = state.clone();

    tauri::Builder::default()
        .manage(state)
        .setup(move |app| {
            let handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                spawn_websocket_bridge(state_clone, handle).await;
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            import_pdf,
            trigger_injection,
            start_auto_detect,
            stop_auto_detect,
            set_journal,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
