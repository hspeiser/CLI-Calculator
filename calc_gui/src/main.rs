use calc_core::Engine;

struct AppState {
    engine: Engine,
    input: String,
    outputs: Vec<String>,
}

impl eframe::App for AppState {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Reactive Calculator");
            ui.separator();

            egui::ScrollArea::vertical().auto_shrink([false; 2]).show(ui, |ui| {
                // Multiline editor: one cell per line
                let response = egui::TextEdit::multiline(&mut self.input)
                    .desired_rows(16)
                    .code_editor()
                    .hint_text("Type expressions, one per line. r1 = 10kΩ, r2 = 15kΩ, r1 // r2 ...")
                    .show(ui);

                if response.response.changed() {
                    self.outputs.clear();
                    for line in self.input.lines() {
                        let trimmed = line.trim();
                        if trimmed.is_empty() || trimmed.starts_with('#') { continue; }
                        match self.engine.eval_cell(trimmed) {
                            Ok(out) => self.outputs.push(out.value.display()),
                            Err(e) => self.outputs.push(format!("error: {}", e)),
                        }
                    }
                }

                ui.separator();
                for (i, out) in self.outputs.iter().enumerate() {
                    ui.monospace(format!("{:>3}: {}", i + 1, out));
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Calc GUI",
        options,
        Box::new(|_cc| Box::new(AppState { engine: Engine::new(), input: String::new(), outputs: Vec::new() })),
    )
}
