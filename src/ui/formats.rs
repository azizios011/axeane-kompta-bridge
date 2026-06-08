use eframe::egui;
use crate::ui::SharedAppState;

pub fn render(ui: &mut egui::Ui, s: &mut SharedAppState) {
    ui.heading("⚙️ Formula Rules & Ledger Mappings Manager");
    ui.label("Map matching criteria and dynamic string scripts using variables: row.ttc, row.ht, row.tva.");
    ui.add_space(5.0);

    if ui.button("➕ Generate Client Template Mapping Card").clicked() {
        s.templates.push(crate::ui::FormatTemplate {
            id: None,
            client_key: "NEW COMPANY IDENTIFIER".to_string(),
            rows: vec![
                crate::ui::TemplateRow {
                    compte: "41100000".to_string(),
                    entry_type: crate::ui::EntryType::Debit,
                    formula: "row.ttc".to_string(),
                },
                crate::ui::TemplateRow {
                    compte: "70700000".to_string(),
                    entry_type: crate::ui::EntryType::Credit,
                    formula: "row.ht".to_string(),
                },
                crate::ui::TemplateRow {
                    compte: "43670000".to_string(),
                    entry_type: crate::ui::EntryType::Credit,
                    formula: "row.tva".to_string(),
                },
            ],
        });
    }
    ui.add_space(5.0);

    egui::ScrollArea::vertical().show(ui, |ui| {
        let mut target_deletes = Vec::new();
        for (idx, t) in s.templates.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label("Identity Matching Keyword: ");
                    ui.text_edit_singleline(&mut t.client_key);
                    ui.add_space(20.0);
                    if ui.button("❌ Remove Template").clicked() {
                        target_deletes.push(idx);
                    }
                });
                ui.separator();
                
                let mut row_deletes = Vec::new();
                for (r_idx, r) in t.rows.iter_mut().enumerate() {
                    ui.horizontal(|ui| {
                        ui.label("Compte:");
                        ui.text_edit_singleline(&mut r.compte);
                        
                        egui::ComboBox::from_id_source(format!("type_{}_{}", idx, r_idx))
                            .selected_text(if r.entry_type == crate::ui::EntryType::Debit { "Debit" } else { "Credit" })
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut r.entry_type, crate::ui::EntryType::Debit, "Debit");
                                ui.selectable_value(&mut r.entry_type, crate::ui::EntryType::Credit, "Credit");
                            });
                            
                        ui.label("Formula:");
                        ui.text_edit_singleline(&mut r.formula);
                        
                        if ui.button("🗑").clicked() {
                            row_deletes.push(r_idx);
                        }
                    });
                }
                
                for r_idx in row_deletes.into_iter().rev() {
                    t.rows.remove(r_idx);
                }
                
                if ui.button("➕ Add Ledger Row").clicked() {
                    t.rows.push(crate::ui::TemplateRow {
                        compte: "".to_string(),
                        entry_type: crate::ui::EntryType::Debit,
                        formula: "0.0".to_string(),
                    });
                }
            });
            ui.add_space(5.0);
        }
        for idx in target_deletes.into_iter().rev() {
            s.templates.remove(idx);
        }
    });
}
