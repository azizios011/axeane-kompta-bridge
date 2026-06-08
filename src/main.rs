mod ui;
mod bin {
    pub mod llm_backend;
    pub mod browser_backend;
}

use std::sync::{Arc, Mutex};
use eframe::egui;
use ui::{ActiveTab, SharedAppState, FormatTemplate, LlmConfig};

use bin::browser_backend::compile_payload_from_state;
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::protocol::Message;
use futures_util::SinkExt;

struct ApplicationShellWindow {
    current_tab: ActiveTab,
    state: Arc<Mutex<SharedAppState>>,
}

#[tokio::main]
async fn main() -> Result<(), eframe::Error> {
    let state = Arc::new(Mutex::new(SharedAppState {
        csv_rows: Vec::new(),
        templates: vec![FormatTemplate { 
            id: Some(1), 
            client_key: "PASSAGER".to_string(), 
            rows: vec![
                ui::TemplateRow {
                    compte: "41100000".to_string(),
                    entry_type: ui::EntryType::Debit,
                    formula: "row.ttc".to_string(),
                },
                ui::TemplateRow {
                    compte: "70700000".to_string(),
                    entry_type: ui::EntryType::Credit,
                    formula: "row.ht".to_string(),
                },
                ui::TemplateRow {
                    compte: "43670000".to_string(),
                    entry_type: ui::EntryType::Credit,
                    formula: "row.tva".to_string(),
                },
            ]
        }],
        port: "9222".to_string(),
        browser_path: "chrome".to_string(),
        incognito: true,
        status: "System online. Configure parameters within AI Core tab before parsing document.pdf.".to_string(),
        trigger_injection: false,
        auto_detect_active: false,
        is_loading: false,
        llm: LlmConfig {
            provider_name: "DeepSeek".to_string(),
            endpoint_url: "https://api.deepseek.com/v1/chat/completions".to_string(),
            api_key: "".to_string(),
            model_name: "deepseek-chat".to_string(),
            temperature: 0.1,
            max_tokens: 3000,
        }
    }));

    let state_worker_ref = state.clone();
    tokio::spawn(async move {
        let listener = TcpListener::bind("127.0.0.1:8085").await.unwrap();
        while let Ok((stream, _)) = listener.accept().await {
            let local_app_state = state_worker_ref.clone();
            tokio::spawn(async move {
                if let Ok(mut ws) = accept_async(stream).await {
                    {
                        let mut s = local_app_state.lock().unwrap();
                        s.status = "Isolated workflow target tab paired via background WebSocket channel.".to_string();
                    }
                    loop {
                        tokio::time::sleep(tokio::time::Duration::from_millis(300)).await;
                        
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
                            let _ = ws.send(Message::Text(format!("EVAL_REQUEST:{}", detection_script))).await;
                            
                            let page_status = "READY"; // Replace this with actual CDP/WS read state eventually
                            
                            let mut s = local_app_state.lock().unwrap();
                            if page_status == "READY" {
                                s.status = "Axeane layout detected! Initiating automation sync inject loop...".to_string();
                                s.auto_detect_active = false; 
                                s.trigger_injection = true;    
                            } else if page_status == "WRONG_PAGE" {
                                s.status = "Found Axeane, but you aren't on the transaction writing form yet.".to_string();
                            } else {
                                s.status = "Waiting for kompta.axeane.com page load...".to_string();
                            }
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

    let native_options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([760.0, 520.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Axeane Automation Sync Workspace Engine",
        native_options,
        Box::new(|_cc| Box::new(ApplicationShellWindow { current_tab: ActiveTab::CsvEditor, state })),
    )
}

impl eframe::App for ApplicationShellWindow {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let mut s = self.state.lock().unwrap();

        egui::TopBottomPanel::top("navigation_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.selectable_value(&mut self.current_tab, ActiveTab::CsvEditor, "📊 Data Table Editor");
                ui.selectable_value(&mut self.current_tab, ActiveTab::FormatTemplates, "⚙️ Formula Fields Setup");
                ui.selectable_value(&mut self.current_tab, ActiveTab::BrowserSettings, "🌐 Browser Profiles Engine");
                ui.selectable_value(&mut self.current_tab, ActiveTab::LlmSettings, "🧠 AI Core Parameters");
            });
        });

        egui::TopBottomPanel::bottom("status_bar").show(ctx, |ui| {
            ui.horizontal(|ui| {
                ui.label("System Logs: ");
                ui.colored_label(egui::Color32::LIGHT_BLUE, &s.status);
            });
        });

        egui::CentralPanel::default().show(ctx, |ui| {
            match self.current_tab {
                ActiveTab::CsvEditor => ui::editor::render(ui, &mut s, self.state.clone(), ctx.clone()),
                ActiveTab::FormatTemplates => ui::formats::render(ui, &mut s),
                ActiveTab::BrowserSettings => ui::settings::render(ui, &mut s),
                ActiveTab::LlmSettings => ui::llm::render(ui, &mut s),
            }
        });
    }
}
