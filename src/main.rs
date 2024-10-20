use std::fs::File;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use eframe::egui;
use egui_file_dialog::FileDialog;

struct MyApp {
    file_dialog: FileDialog,
    selected_file: Option<PathBuf>,
    search_sequence: String,
    replace_sequence: String,
    status_message: String,
}

impl MyApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            file_dialog: FileDialog::new(),
            selected_file: None,
            search_sequence: String::new(),
            replace_sequence: String::new(),
            status_message: String::new(),
        }
    }

    fn hex_string_to_bytes(hex: &str) -> Vec<u8> {
        hex.split_whitespace()
            .map(|byte_str| {
                if byte_str == "??" {
                    0x00
                } else {
                    u8::from_str_radix(byte_str, 16).expect("Invalid hex string")
                }
            })
            .collect()
    }
    fn patch_code(&mut self, in_file: &PathBuf) -> io::Result<()> {
        let mut input = File::open(in_file)?;
        let mut data = Vec::new();
        input.read_to_end(&mut data)?;

        let orig_bytes = Self::hex_string_to_bytes(&self.search_sequence);
        let repl_bytes = Self::hex_string_to_bytes(&self.replace_sequence);

        if let Some(pos) = data.windows(orig_bytes.len()).position(|window| {
            window
                .iter()
                .zip(&orig_bytes)
                .all(|(w, o)| *o == 0x00 || *w == *o)
        }) {
            self.status_message = format!(
                "Found a match for {} at index {}",
                self.search_sequence, pos
            );

            data.splice(pos..pos + orig_bytes.len(), repl_bytes.iter().cloned());

            self.status_message = format!(
                "Replaced bytes at position {} with {}",
                pos, self.replace_sequence
            );
        } else {
            self.status_message = format!("No match found for {}", self.search_sequence);
            return Ok(());
        }

        let mut backup_file = in_file.clone();
        let original_name = backup_file.file_stem().unwrap().to_str().unwrap();

        let backup_ext = in_file
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");

        let new_name = format!("{}_original.{}", original_name, backup_ext);
        backup_file.set_file_name(new_name);
        std::fs::rename(in_file, backup_file)?;
        let mut output = File::create(in_file)?;
        output.write_all(&data)?;

        self.status_message = format!("Patching complete. Output written to {}", in_file.display());
        Ok(())
    }
}

impl eframe::App for MyApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            if ui.button("Select input file").clicked() {
                self.file_dialog.select_file();
            }

            if let Some(path) = &self.selected_file {
                ui.label(format!("Selected file: {:?}", path));
            } else {
                ui.label("No input file selected");
            }

            self.file_dialog.update(ctx);

            if let Some(path) = self.file_dialog.take_selected() {
                self.selected_file = Some(path.to_path_buf());
            }

            ui.horizontal(|ui| {
                ui.label("Search hex sequence:");
                ui.text_edit_singleline(&mut self.search_sequence);
            });

            ui.horizontal(|ui| {
                ui.label("Replace hex sequence:");
                ui.text_edit_singleline(&mut self.replace_sequence);
            });

            if ui.button("Patch").clicked() {
                if let Some(in_file) = &self.selected_file.clone() {
                    if self.search_sequence.is_empty() || self.replace_sequence.is_empty() {
                        self.status_message = "Please provide valid hex sequences.".to_string();
                    } else {
                        match self.patch_code(in_file) {
                            Ok(_) => {}
                            Err(err) => {
                                self.status_message = format!("Failed to patch: {}", err);
                            }
                        }
                    }
                } else {
                    self.status_message = "Please select an input file.".to_string();
                }
            }

            ui.label(&self.status_message);
        });
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "Patch File with Hex Sequence",
        eframe::NativeOptions::default(),
        Box::new(|cc| Ok(Box::new(MyApp::new(cc)))),
    )
}
