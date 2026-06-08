mod app_state;
mod bin {
    pub mod browser_backend;
    pub mod llm_backend;
}

use std::{
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
use bin::browser_backend::compile_payload_from_state;
use futures_util::SinkExt;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;

const FRONTEND_URL: &str = "http://127.0.0.1:3000";

fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let state = Arc::new(Mutex::new(initial_state()));
    
    std::thread::spawn({
        let state = state.clone();
        move || {
            let rt = tokio::runtime::Runtime::new().unwrap();
            rt.block_on(async move {
                spawn_browser_websocket_bridge(state);
                let pending: std::future::Pending<()> = std::future::pending();
                pending.await;
            });
        }
    });

    let project_root = std::env::current_dir()?;
    let frontend_dir = project_root.join("frontend");
    
    println!("Starting Next.js server...");
    let frontend_process = launch_frontend(&frontend_dir)?;

    wait_for_frontend();
    
    println!("Axeane frontend is running at {}", FRONTEND_URL);
    println!("Automation WebSocket bridge is listening on 127.0.0.1:8085");
    println!("Opening native UI Window...");

    open_frontend_window(frontend_process);

    Ok(())
}

fn spawn_browser_websocket_bridge(state: Arc<Mutex<SharedAppState>>) {
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:8085")
            .await
            .expect("failed to bind browser WebSocket bridge on 127.0.0.1:8085");

        while let Ok((stream, _)) = listener.accept().await {
            let local_app_state = state.clone();
            tokio::spawn(async move {
                if let Ok(mut ws) = accept_async(stream).await {
                    {
                        let mut s = local_app_state.lock().unwrap();
                        s.status = "Isolated workflow target tab paired via background WebSocket channel.".to_string();
                    }

                    loop {
                        tokio::time::sleep(Duration::from_millis(300)).await;

                        let auto_detect_active = {
                            let s = local_app_state.lock().unwrap();
                            s.auto_detect_active
                        };

                        if auto_detect_active {
                            {
                                let mut s = local_app_state.lock().unwrap();
                                s.status = "Scanning browser DOM for Axeane elements (ecritureForm, cc_0_3)...".to_string();
                            }

                            let detection_script = bin::browser_backend::compile_page_detection_script();
                            let _ = ws.send(Message::Text(format!("EVAL_REQUEST:{detection_script}"))).await;

                            let mut s = local_app_state.lock().unwrap();
                            s.status = "Axeane layout detected! Initiating automation sync inject loop...".to_string();
                            s.auto_detect_active = false;
                            s.trigger_injection = true;
                        }

                        let trigger = {
                            let s = local_app_state.lock().unwrap();
                            s.trigger_injection
                        };

                        if trigger {
                            let json_payload = {
                                let s = local_app_state.lock().unwrap();
                                compile_payload_from_state(&s.csv_rows, &s.templates)
                            };
                            let injection_payload = format!(
                                r#"
                                (function() {{
                                    const payloadData = {};
                                    const form = document.getElementById('ecritureForm');
                                    if(!form) {{ alert('Form viewport targets not visible.'); return; }}
                                    const $scope = angular.element(form).scope();
                                    if($scope) {{
                                        let idx = 0;
                                        function pushLoop() {{
                                            if(idx >= payloadData.length) return;
                                            const entry = payloadData[idx];
                                            $scope.$apply(function() {{
                                                if(!$scope.ecritureGrouping) $scope.ecritureGrouping = {{}};
                                                $scope.ecritureGrouping.dateOperation = entry.date;
                                                $scope.ecritureGrouping.libelle = "Facture Ref: " + entry.ref;
                                                if(!$scope.ecritureGrouping.lignes) $scope.ecritureGrouping.lignes = [];
                                                $scope.ecritureGrouping.lignes = entry.lignes.map(l => ({{
                                                    compte: l.compte,
                                                    libelle: "Facture Ref: " + entry.ref,
                                                    montantDebit: l.type === 'debit' ? l.amount : 0,
                                                    montantCredit: l.type === 'credit' ? l.amount : 0
                                                }}));
                                            }});
                                            idx++;
                                            setTimeout(pushLoop, 1100);
                                        }}
                                        pushLoop();
                                    }}
                                }})();
                                "#,
                                json_payload
                            );
                            let _ = ws.send(Message::Text(injection_payload)).await;
                            {
                                let mut s = local_app_state.lock().unwrap();
                                s.trigger_injection = false;
                                s.status = "Injection transaction block macros dispatched downstream.".to_string();
                            }
                        }
                    }
                }
            });
        }
    });
}

fn launch_frontend(frontend_dir: &PathBuf) -> std::io::Result<Child> {
    let script = if frontend_dir.join(".next").exists() { "start" } else { "dev" };

    Command::new("npm.cmd")
        .arg("run")
        .arg(script)
        .arg("--")
        .arg("--hostname")
        .arg("127.0.0.1")
        .arg("--port")
        .arg("3000")
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

        if let Event::WindowEvent {
            event: WindowEvent::CloseRequested,
            ..
        } = event
        {
            let _ = frontend_process.kill();
            *control_flow = ControlFlow::Exit;
        }
    });
}

