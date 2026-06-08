use eframe::egui;
use crate::ui::SharedAppState;

pub fn render(ui: &mut egui::Ui, s: &mut SharedAppState) {
    ui.heading("🧠 AI Orchestration & Context Configuration Panel");
    ui.label("Dynamically point data pipelines to specialized localized nodes or secure remote endpoints.");
    ui.add_space(10.0);

    egui::Grid::new("llm_dynamic_configurations")
        .num_columns(2)
        .spacing([15.0, 10.0])
        .striped(true)
        .show(ui, |ui| {
            ui.label("Core Provider Tag:");
            ui.text_edit_singleline(&mut s.llm.provider_name);
            ui.end_row();

            ui.label("Base URL Target Endpoint:");
            ui.text_edit_singleline(&mut s.llm.endpoint_url);
            ui.end_row();

            ui.label("Bearer / Client API Security Token:");
            ui.text_edit_singleline(&mut s.llm.api_key);
            ui.end_row();

            ui.label("Target Architecture Model Name:");
            ui.text_edit_singleline(&mut s.llm.model_name);
            ui.end_row();

            ui.label("Context Generation Temperature:");
            ui.add(egui::Slider::new(&mut s.llm.temperature, 0.0..=2.0).step_by(0.1));
            ui.end_row();

            ui.label("Max Tokens Sequence Constraint:");
            ui.add(egui::DragValue::new(&mut s.llm.max_tokens).speed(50.0).clamp_range(1..=16384));
            ui.end_row();
        });
}
