use crate::modals::render_modals;
use crate::panels::{render_center_editor, render_left_palette, render_right_preview, render_top_bar};
use crate::theme::apply_theme;
use arboard::Clipboard;
use eframe::egui::{self, Key};
use prompt_application::{
    commands::Command,
    state::{PendingRefinements, ProposedSectionChange, SuggestedSection},
    ApplicationService,
};
use prompt_core::{template::ALL_TEMPLATES, PromptDocument, RenderStage};
use prompt_persistence::{save_document, AppConfig, ProviderType};
use prompt_refinement::{
    generate_manual_request, parse_manual_response, OpenAiCompatibleProvider, RefinementChangeset,
    RefinementEngine, RefinementMode,
};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use uuid::Uuid;

pub struct PromptForgeGuiApp {
    pub service: ApplicationService,
    pub config: AppConfig,
    pub preview_stage: RenderStage,
    pub preview_include_ids: bool,
    pub custom_tag_input: String,
    pub dragging_section_id: Option<Uuid>,
    pub drag_target_index: Option<usize>,
    pub status_message: Option<(String, Instant)>,

    // Modals
    pub show_refine_modal: bool,
    pub show_diff_modal: bool,
    pub show_export_manual_modal: bool,
    pub show_import_manual_modal: bool,
    pub show_save_as_modal: bool,
    pub show_open_modal: bool,
    pub show_settings_modal: bool,
    pub show_help_modal: bool,

    // Refinement state
    pub refine_mode: RefinementMode,
    pub refine_provider: ProviderType,
    pub is_refining: bool,
    pub manual_export_text: String,
    pub manual_import_text: String,
    pub save_as_path: String,
    pub open_file_path: String,

    // Background channel
    refine_rx: Option<Receiver<Result<RefinementChangeset, String>>>,
}

impl PromptForgeGuiApp {
    pub fn new(initial_doc: Option<PromptDocument>, file_path: Option<PathBuf>) -> Self {
        let config = AppConfig::load_or_default();
        let doc = initial_doc.unwrap_or_else(|| ALL_TEMPLATES[0].instantiate(Some("New Prompt Document")));
        let mut service = ApplicationService::new(doc);
        if let Some(ref path) = file_path {
            service.mark_saved(path.clone());
        }

        Self {
            service,
            config,
            preview_stage: RenderStage::Draft,
            preview_include_ids: true,
            custom_tag_input: String::new(),
            dragging_section_id: None,
            drag_target_index: None,
            status_message: None,

            show_refine_modal: false,
            show_diff_modal: false,
            show_export_manual_modal: false,
            show_import_manual_modal: false,
            show_save_as_modal: false,
            show_open_modal: false,
            show_settings_modal: false,
            show_help_modal: false,

            refine_mode: RefinementMode::Conservative,
            refine_provider: ProviderType::Manual,
            is_refining: false,
            manual_export_text: String::new(),
            manual_import_text: String::new(),
            save_as_path: String::new(),
            open_file_path: String::new(),

            refine_rx: None,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some((msg.into(), Instant::now()));
    }

    pub fn clear_status(&mut self) {
        self.status_message = None;
    }

    pub fn copy_to_clipboard(&mut self, text: &str) {
        match Clipboard::new() {
            Ok(mut clipboard) => match clipboard.set_text(text) {
                Ok(_) => self.set_status("Copied to system clipboard!"),
                Err(e) => self.set_status(format!("Clipboard error: {e}")),
            },
            Err(e) => self.set_status(format!("Clipboard init error: {e}")),
        }
    }

    pub fn save_current_or_prompt(&mut self) {
        if let Some(path) = self.service.current_file_path() {
            let path = path.to_path_buf();
            match save_document(&path, self.service.document()) {
                Ok(_) => {
                    self.service.mark_saved(path);
                    self.set_status("Document saved successfully");
                }
                Err(e) => self.set_status(format!("Save error: {e}")),
            }
        } else {
            let default_name = format!(
                "{}.prompt.json",
                self.service.document().title.to_lowercase().replace(' ', "_")
            );
            self.save_as_path = default_name;
            self.show_save_as_modal = true;
        }
    }

    pub fn open_export_manual_modal(&mut self) {
        self.manual_export_text =
            generate_manual_request(self.service.document(), self.refine_mode).unwrap_or_default();
        self.show_export_manual_modal = true;
    }

    pub fn import_manual_response(&mut self) {
        let raw = self.manual_import_text.trim();
        if raw.is_empty() {
            self.set_status("Please paste the response first");
            return;
        }

        match parse_manual_response(raw, self.service.document()) {
            Ok(changeset) => {
                let pending = PendingRefinements {
                    changes: changeset
                        .changes
                        .into_iter()
                        .map(|c| ProposedSectionChange {
                            section_id: c.section_id,
                            proposed_text: c.refined,
                            reason: c.reason,
                        })
                        .collect(),
                    suggested_sections: changeset
                        .suggested_sections
                        .into_iter()
                        .map(|s| SuggestedSection {
                            tag: s.tag,
                            content: s.content,
                            reason: s.reason,
                        })
                        .collect(),
                    open_questions: changeset.open_questions,
                    warnings: changeset.warnings,
                    critique: changeset.critique,
                };

                self.service.set_pending_refinements(pending);
                self.show_import_manual_modal = false;
                self.show_diff_modal = true;
                self.set_status("Refinements imported successfully!");
            }
            Err(e) => {
                self.set_status(format!("Import error: {e}"));
            }
        }
    }

    pub fn start_refinement(&mut self) {
        if self.refine_provider == ProviderType::Manual {
            self.show_refine_modal = false;
            self.open_export_manual_modal();
        } else {
            let (tx, rx) = channel();
            self.refine_rx = Some(rx);
            self.is_refining = true;
            self.set_status(format!("Requesting refinement from OpenAI ({:?})...", self.refine_mode));

            let doc = self.service.document().clone();
            let cfg = self.config.openai_compatible.clone();
            let mode = self.refine_mode;

            tokio::spawn(async move {
                let provider = OpenAiCompatibleProvider::new(
                    cfg.base_url,
                    cfg.model,
                    cfg.api_key_env_var,
                    cfg.timeout_seconds,
                );
                let engine = RefinementEngine::new();
                let result = engine.execute(&provider, &doc, mode).await;
                let _ = tx.send(result.map_err(|e| e.to_string()));
            });
        }
    }

    fn check_background_refinement(&mut self) {
        if let Some(ref rx) = self.refine_rx {
            if let Ok(res) = rx.try_recv() {
                self.is_refining = false;
                self.refine_rx = None;

                match res {
                    Ok(changeset) => {
                        let pending = PendingRefinements {
                            changes: changeset
                                .changes
                                .into_iter()
                                .map(|c| ProposedSectionChange {
                                    section_id: c.section_id,
                                    proposed_text: c.refined,
                                    reason: c.reason,
                                })
                                .collect(),
                            suggested_sections: changeset
                                .suggested_sections
                                .into_iter()
                                .map(|s| SuggestedSection {
                                    tag: s.tag,
                                    content: s.content,
                                    reason: s.reason,
                                })
                                .collect(),
                            open_questions: changeset.open_questions,
                            warnings: changeset.warnings,
                            critique: changeset.critique,
                        };

                        self.service.set_pending_refinements(pending);
                        self.show_refine_modal = false;
                        self.show_diff_modal = true;
                        self.set_status("Refinement complete! Review changes.");
                    }
                    Err(err) => {
                        self.set_status(format!("Refinement failed: {err}"));
                    }
                }
            }
        }
    }

    fn handle_keyboard_shortcuts(&mut self, ctx: &egui::Context) {
        ctx.input(|i| {
            if i.modifiers.command || i.modifiers.ctrl {
                if i.key_pressed(Key::S) {
                    self.save_current_or_prompt();
                } else if i.key_pressed(Key::O) {
                    self.show_open_modal = true;
                } else if i.key_pressed(Key::Z) {
                    let _ = self.service.execute(Command::Undo);
                } else if i.key_pressed(Key::Y) {
                    let _ = self.service.execute(Command::Redo);
                } else if i.key_pressed(Key::R) {
                    self.show_refine_modal = true;
                }
            }
        });
    }
}

impl eframe::App for PromptForgeGuiApp {
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        apply_theme(ui.ctx());
        self.check_background_refinement();
        self.handle_keyboard_shortcuts(ui.ctx());

        // Clear expired status messages after 5 seconds
        if let Some((_, created)) = self.status_message {
            if created.elapsed() > Duration::from_secs(5) {
                self.status_message = None;
            }
        }

        // Top menu / quick bar
        egui::Panel::top("top_menu").show(ui, |ui| {
            render_top_bar(self, ui);
        });

        // Left palette
        egui::Panel::left("left_palette")
            .resizable(true)
            .default_size(260.0)
            .min_size(200.0)
            .max_size(380.0)
            .show(ui, |ui| {
                render_left_palette(self, ui);
            });

        // Right preview
        egui::Panel::right("right_preview")
            .resizable(true)
            .default_size(320.0)
            .min_size(240.0)
            .max_size(480.0)
            .show(ui, |ui| {
                render_right_preview(self, ui);
            });

        // Center editor
        egui::CentralPanel::default().show(ui, |ui| {
            render_center_editor(self, ui);
        });

        // Modals
        render_modals(self, ui.ctx());
    }
}
