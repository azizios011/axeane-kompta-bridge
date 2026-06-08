use eframe::egui;
use std::process::Command;
use crate::ui::SharedAppState;

pub fn render(ui: &mut egui::Ui, s: &mut SharedAppState) {
    ui.heading("🌐 Remote Proxy Interface Settings");
    ui.add_space(10.0);

    ui.horizontal(|ui| {
        ui.label("Debugging Subprocess Pipeline Socket Port: ");
        ui.text_edit_singleline(&mut s.port);
    });
    
    ui.horizontal(|ui| {
        ui.label("Browser Executable Path: ");
        ui.text_edit_singleline(&mut s.browser_path);
        if ui.button("📂 Browse...").clicked() {
            if let Some(path) = rfd::FileDialog::new()
                .add_filter("Executables", &["exe", "bat", "cmd"])
                .add_filter("All Files", &["*"])
                .pick_file() {
                s.browser_path = path.display().to_string();
            }
        }
    });

    ui.checkbox(&mut s.incognito, "Launch Private / Incognito Session Window Sandbox");
    ui.add_space(15.0);

    if ui.button("🚀 Initialize Isolated Chromium Subprocess instance").clicked() {
        s.status = "Initializing tracking engine flags...".to_string();
        let mut args = vec![
            format!("--remote-debugging-port={}", s.port),
            "https://kompta.axeane.com/".to_string(),
        ];
        if s.incognito { args.push("--incognito".to_string()); }

        let executable = if s.browser_path.is_empty() { "chrome" } else { &s.browser_path };
        if Command::new(executable).args(&args).spawn().is_err() {
            if executable == "chrome" {
                if Command::new("msedge").args(&args).spawn().is_ok() {
                    s.status = format!("Browser sub-instance successfully bound to debug port: {}.", s.port);
                } else {
                    s.status = "Failed to launch Chrome or Edge. Try browsing for your browser executable.".to_string();
                }
            } else {
                s.status = format!("Failed to launch custom browser: {}", executable);
            }
        } else {
            s.status = format!("Browser sub-instance successfully bound to debug port: {}.", s.port);
        }
    }
}
