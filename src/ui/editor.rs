use eframe::egui;
use egui_extras::{TableBuilder, Column};
use crate::ui::SharedAppState;
use crate::bin::llm_backend::{rip_pdf_text, parse_pdf_via_ai};
use std::sync::Arc;

pub fn render(ui: &mut egui::Ui, s: &mut SharedAppState, state_ctx: Arc<std::sync::Mutex<SharedAppState>>, egui_ctx: egui::Context) {
    ui.heading("📊 Extracted Row Items Spreadsheet Grid");
    ui.label("Modify parameters inside cells dynamically before executing browser delivery macros.");
    ui.add_space(5.0);

    if s.is_loading {
        ui.vertical_centered(|ui| {
            ui.add_space(60.0);
            ui.add(egui::Spinner::new().size(45.0));
            ui.add_space(15.0);
            ui.colored_label(egui::Color32::LIGHT_BLUE, "Isolated AI backend ripping ledger structures... Parsing rows downstream.");
            ui.add_space(60.0);
        });
        return; 
    }

    ui.horizontal(|ui| {
        if ui.button("📂 Import document.pdf with Dynamic LLM").clicked() {
            s.is_loading = true;
            s.status = "Initializing non-blocking background runtime parser token pipelines...".to_string();
            
            let thread_state_ref = state_ctx.clone();
            let thread_gui_ctx = egui_ctx.clone();
            let llm_snapshot = s.llm.clone();

            tokio::spawn(async move {
                match rip_pdf_text("document.pdf") {
                    Ok(extracted_text) => {
                        match parse_pdf_via_ai(&extracted_text, &llm_snapshot).await {
                            Ok(new_rows) => {
                                let mut app = thread_state_ref.lock().unwrap();
                                app.csv_rows = new_rows;
                                app.is_loading = false;
                                app.status = "Inference complete! Table populated successfully.".to_string();
                            }
                            Err(e) => {
                                let mut app = thread_state_ref.lock().unwrap();
                                app.is_loading = false;
                                app.status = format!("AI Parsing Core Failure: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        let mut app = thread_state_ref.lock().unwrap();
                        app.is_loading = false;
                        app.status = format!("PDF Ripping Error: {}", e);
                    }
                }
                thread_gui_ctx.request_repaint();
            });
        }

        if ui.button("➕ Insert Custom Row").clicked() {
            s.csv_rows.push(crate::ui::EditableRow {
                client: "PASSAGER".to_string(),
                reference: "FC/2026".to_string(),
                date: "08/06/2026".to_string(),
                ttc: "0.0".to_string(),
                ht: "0.0".to_string(),
                tva: "0.0".to_string(),
            });
        }

        let btn_label = if s.auto_detect_active {
            "⏳ Monitoring Browser... Stop"
        } else {
            "🚀 Start Auto-Detection Filling"
        };

        let btn_color = if s.auto_detect_active {
            egui::Color32::from_rgb(220, 100, 50) // Warning Orange
        } else {
            egui::Color32::from_rgb(40, 120, 240) // Action Blue
        };

        if ui.add(egui::Button::new(btn_label).fill(btn_color)).clicked() {
            if s.csv_rows.is_empty() {
                s.status = "Error: Cannot start automation tracking with an empty spreadsheet grid.".to_string();
            } else {
                s.auto_detect_active = !s.auto_detect_active;
                if s.auto_detect_active {
                    s.status = "Automation listener armed. Navigate to your Axeane ledger sheet to begin.".to_string();
                } else {
                    s.status = "Auto-detection monitoring suspended manually.".to_string();
                }
            }
        }
    });

    ui.add_space(10.0);

    let mut rows_to_delete = Vec::new();
    TableBuilder::new(ui)
        .striped(true)
        .resizable(true)
        .column(Column::initial(140.0))
        .column(Column::initial(110.0))
        .column(Column::initial(90.0)) 
        .column(Column::initial(80.0)) 
        .column(Column::initial(80.0)) 
        .column(Column::initial(80.0)) 
        .column(Column::remainder())   
        .header(20.0, |mut header| {
            header.col(|ui| { ui.strong("Client Match Key"); });
            header.col(|ui| { ui.strong("Invoice Ref"); });
            header.col(|ui| { ui.strong("Op Date"); });
            header.col(|ui| { ui.strong("Total TTC"); });
            header.col(|ui| { ui.strong("Base HT"); });
            header.col(|ui| { ui.strong("VAT Amount"); });
            header.col(|ui| { ui.strong("Actions"); });
        })
        .body(|body| {
            let total_rows = s.csv_rows.len();
            body.rows(25.0, total_rows, |mut row| {
                let index = row.index();
                if let Some(row_item) = s.csv_rows.get_mut(index) {
                    row.col(|ui| { ui.text_edit_singleline(&mut row_item.client); });
                    row.col(|ui| { ui.text_edit_singleline(&mut row_item.reference); });
                    row.col(|ui| { ui.text_edit_singleline(&mut row_item.date); });
                    row.col(|ui| { ui.text_edit_singleline(&mut row_item.ttc); });
                    row.col(|ui| { ui.text_edit_singleline(&mut row_item.ht); });
                    row.col(|ui| { ui.text_edit_singleline(&mut row_item.tva); });
                    row.col(|ui| {
                        if ui.button("🗑").clicked() {
                            rows_to_delete.push(index);
                        }
                    });
                }
            });
        });

    for idx in rows_to_delete.into_iter().rev() {
        s.csv_rows.remove(idx);
    }
}
