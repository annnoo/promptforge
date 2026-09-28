use crate::app::PromptForgeGuiApp;
use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Vec2};
use prompt_application::commands::Command;
use prompt_persistence::{load_document, save_document, ProviderType};
use prompt_refinement::RefinementMode;
use std::path::PathBuf;

/// Renders all active modal windows.
pub fn render_modals(app: &mut PromptForgeGuiApp, ctx: &egui::Context) {
    if app.show_refine_modal {
        render_refine_modal(app, ctx);
    }
    if app.show_diff_modal {
        render_diff_modal(app, ctx);
    }
    if app.show_export_manual_modal {
        render_export_manual_modal(app, ctx);
    }
    if app.show_import_manual_modal {
        render_import_manual_modal(app, ctx);
    }
    if app.show_save_as_modal {
        render_save_as_modal(app, ctx);
    }
    if app.show_open_modal {
        render_open_modal(app, ctx);
    }
    if app.show_settings_modal {
        render_settings_modal(app, ctx);
    }
    if app.show_help_modal {
        render_help_modal(app, ctx);
    }
}

fn render_refine_modal(app: &mut PromptForgeGuiApp, ctx: &egui::Context) {
    let mut is_open = true;
    egui::Window::new("Refine Prompt")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(520.0, 420.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.heading("Refinement Mode");
            ui.add_space(4.0);

            ui.radio_value(&mut app.refine_mode, RefinementMode::Conservative, "Conservative");
            ui.label(RichText::new("  Preserve all requirements strictly; improve clarity, precision, and tone.").size(11.0).color(Color32::GRAY));

            ui.radio_value(&mut app.refine_mode, RefinementMode::Expand, "Expand");
            ui.label(RichText::new("  Elaborate instructions where justified, suggest missing sections & constraints.").size(11.0).color(Color32::GRAY));

            ui.radio_value(&mut app.refine_mode, RefinementMode::Critique, "Critique");
            ui.label(RichText::new("  Analyze ambiguities, contradictions, and gaps without modifying document text.").size(11.0).color(Color32::GRAY));

            ui.add_space(8.0);
            ui.separator();
            ui.add_space(4.0);

            ui.heading("Provider Selection");
            ui.add_space(4.0);

            ui.radio_value(&mut app.refine_provider, ProviderType::Manual, "Manual (No API Key Required)");
            ui.label(RichText::new("  Generate request to copy into ChatGPT / Claude, then paste response.").size(11.0).color(Color32::GRAY));

            ui.radio_value(&mut app.refine_provider, ProviderType::OpenAiCompatible, "OpenAI-Compatible HTTP Provider");
            ui.label(RichText::new("  Direct HTTP API integration using your local or cloud provider.").size(11.0).color(Color32::GRAY));

            if app.refine_provider == ProviderType::OpenAiCompatible {
                ui.add_space(4.0);
                Frame::new()
                    .fill(Color32::from_rgb(20, 24, 30))
                    .corner_radius(CornerRadius::same(4))
                    .inner_margin(Margin::same(8))
                    .show(ui, |ui| {
                        ui.horizontal(|ui| {
                            ui.label("Base URL:");
                            ui.text_edit_singleline(&mut app.config.openai_compatible.base_url);
                        });
                        ui.horizontal(|ui| {
                            ui.label("Model:");
                            ui.text_edit_singleline(&mut app.config.openai_compatible.model);
                        });
                        ui.horizontal(|ui| {
                            ui.label("API Key Env:");
                            ui.text_edit_singleline(&mut app.config.openai_compatible.api_key_env_var);
                        });
                    });
            }

            ui.add_space(12.0);
            ui.separator();

            ui.horizontal(|ui| {
                if app.is_refining {
                    ui.spinner();
                    ui.label("Refining prompt...");
                } else if ui.button(RichText::new("Start Refinement").strong()).clicked() {
                    app.start_refinement();
                }

                if ui.button("Cancel").clicked() {
                    app.show_refine_modal = false;
                }
            });
        });

    if !is_open {
        app.show_refine_modal = false;
    }
}

fn render_diff_modal(app: &mut PromptForgeGuiApp, ctx: &egui::Context) {
    let mut is_open = true;
    egui::Window::new("Review Refinement Changes")
        .open(&mut is_open)
        .collapsible(false)
        .default_size(Vec2::new(760.0, 560.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            let pending_opt = app.service.state().pending_refinements.clone();
            let pending = match pending_opt {
                Some(p) if !p.is_empty() => p,
                _ => {
                    ui.label("No pending refinements to review.");
                    if ui.button("Close").clicked() {
                        app.show_diff_modal = false;
                    }
                    return;
                }
            };

            ui.horizontal(|ui| {
                ui.label(RichText::new(format!("Pending Proposed Changes: {}", pending.changes.len())).strong());
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    if ui.button(RichText::new("Reject All").color(Color32::from_rgb(255, 120, 120))).clicked() {
                        let _ = app.service.execute(Command::RejectAllRefinements);
                        app.set_status("All refinements rejected");
                        app.show_diff_modal = false;
                    }
                    if ui.button(RichText::new("Accept All").color(Color32::from_rgb(100, 220, 140)).strong()).clicked() {
                        let _ = app.service.execute(Command::ApplyAllRefinements);
                        app.set_status("All refinements accepted!");
                        app.show_diff_modal = false;
                    }
                });
            });

            ui.separator();

            egui::ScrollArea::vertical()
                .id_salt("diff_scroll")
                .show(ui, |ui| {
                    let mut accept_id = None;
                    let mut reject_id = None;
                    let mut accept_suggestion_idx = None;
                    let mut dismiss_suggestion_idx = None;

                    // Critique if present
                    if let Some(ref critique) = pending.critique {
                        Frame::new()
                            .fill(Color32::from_rgb(30, 25, 40))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(10))
                            .show(ui, |ui| {
                                ui.label(RichText::new("Model Critique & Analysis:").strong().color(Color32::from_rgb(200, 160, 255)));
                                ui.label(critique);
                            });
                        ui.add_space(8.0);
                    }

                    // Proposed section changes
                    for change in &pending.changes {
                        let section = app.service.document().section(change.section_id);
                        let tag = section.map(|s| s.tag.as_str()).unwrap_or("section");
                        let orig_text = section.map(|s| s.brief.as_str()).unwrap_or("");

                        Frame::group(ui.style())
                            .fill(Color32::from_rgb(24, 27, 34))
                            .corner_radius(CornerRadius::same(6))
                            .inner_margin(Margin::same(10))
                            .show(ui, |ui| {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("<{tag}>")).color(Color32::CYAN).strong());
                                    ui.label(RichText::new(format!("Reason: {}", change.reason)).size(11.0).color(Color32::GRAY));

                                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                        if ui.button(RichText::new("Reject").color(Color32::from_rgb(255, 120, 120))).clicked() {
                                            reject_id = Some(change.section_id);
                                        }
                                        if ui.button(RichText::new("Accept").color(Color32::from_rgb(100, 220, 140)).strong()).clicked() {
                                            accept_id = Some(change.section_id);
                                        }
                                    });
                                });

                                ui.add_space(6.0);

                                // Side by side comparison: Left Original, Right Proposed
                                ui.columns(2, |cols| {
                                    cols[0].vertical(|ui| {
                                        ui.label(RichText::new("Original Brief").size(11.0).color(Color32::from_rgb(255, 150, 150)));
                                        Frame::new()
                                            .fill(Color32::from_rgb(34, 22, 22))
                                            .corner_radius(CornerRadius::same(4))
                                            .inner_margin(Margin::same(6))
                                            .show(ui, |ui| {
                                                ui.label(orig_text);
                                            });
                                    });

                                    cols[1].vertical(|ui| {
                                        ui.label(RichText::new("Proposed Refinement").size(11.0).color(Color32::from_rgb(150, 255, 180)));
                                        Frame::new()
                                            .fill(Color32::from_rgb(20, 34, 24))
                                            .corner_radius(CornerRadius::same(4))
                                            .inner_margin(Margin::same(6))
                                            .show(ui, |ui| {
                                                ui.label(&change.proposed_text);
                                            });
                                    });
                                });
                            });
                        ui.add_space(8.0);
                    }

                    // Suggested new sections
                    if !pending.suggested_sections.is_empty() {
                        ui.heading("Suggested New Sections");
                        ui.add_space(4.0);
                        for (s_idx, suggestion) in pending.suggested_sections.iter().enumerate() {
                            Frame::group(ui.style())
                                .fill(Color32::from_rgb(22, 26, 32))
                                .corner_radius(CornerRadius::same(6))
                                .inner_margin(Margin::same(10))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.label(RichText::new(format!("Suggested: <{}>", suggestion.tag)).color(Color32::YELLOW).strong());
                                        ui.label(RichText::new(&suggestion.reason).size(11.0).color(Color32::GRAY));

                                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                            if ui.button("Dismiss").clicked() {
                                                dismiss_suggestion_idx = Some(s_idx);
                                            }
                                            if ui.button(RichText::new("+ Add to Document").color(Color32::GREEN)).clicked() {
                                                accept_suggestion_idx = Some(s_idx);
                                            }
                                        });
                                    });
                                    ui.add_space(4.0);
                                    ui.label(&suggestion.content);
                                });
                            ui.add_space(6.0);
                        }
                    }

                    // Open questions & warnings
                    if !pending.open_questions.is_empty() {
                        ui.heading("Open Questions");
                        for q in &pending.open_questions {
                            ui.label(format!("• {q}"));
                        }
                        ui.add_space(6.0);
                    }

                    if !pending.warnings.is_empty() {
                        ui.heading("Warnings");
                        for w in &pending.warnings {
                            ui.label(RichText::new(format!("⚠ {w}")).color(Color32::from_rgb(255, 180, 80)));
                        }
                    }

                    // Apply actions
                    if let Some(id) = accept_id {
                        let _ = app.service.execute(Command::ApplyRefinement { id });
                    }
                    if let Some(id) = reject_id {
                        let _ = app.service.execute(Command::RejectRefinement { id });
                    }
                    if let Some(idx) = accept_suggestion_idx {
                        let _ = app.service.accept_suggested_section(idx);
                    }
                    if let Some(idx) = dismiss_suggestion_idx {
                        let _ = app.service.dismiss_suggested_section(idx);
                    }
                });
        });

    if !is_open {
        app.show_diff_modal = false;
    }
}

fn render_export_manual_modal(app: &mut PromptForgeGuiApp, ctx: &egui::Context) {
    let mut is_open = true;
    egui::Window::new("Manual Refinement Request")
        .open(&mut is_open)
        .collapsible(false)
        .default_size(Vec2::new(640.0, 480.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label("Copy the instructions below and paste them into ChatGPT, Claude, or another LLM:");
            ui.add_space(4.0);

            if ui.button("📋 Copy to Clipboard").clicked() {
                let text = app.manual_export_text.clone();
                app.copy_to_clipboard(&text);
            }

            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .id_salt("manual_export_scroll")
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut app.manual_export_text.as_str())
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .interactive(false),
                    );
                });
        });

    if !is_open {
        app.show_export_manual_modal = false;
    }
}

fn render_import_manual_modal(app: &mut PromptForgeGuiApp, ctx: &egui::Context) {
    let mut is_open = true;
    egui::Window::new("Import Refinement Response")
        .open(&mut is_open)
        .collapsible(false)
        .default_size(Vec2::new(640.0, 480.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label("Paste the model's response below (JSON changeset or XML prompt):");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                if ui.button("Import & Review Changes").clicked() {
                    app.import_manual_response();
                }
                if ui.button("Cancel").clicked() {
                    app.show_import_manual_modal = false;
                }
            });

            ui.add_space(4.0);
            egui::ScrollArea::vertical()
                .id_salt("manual_import_scroll")
                .show(ui, |ui| {
                    ui.add(
                        egui::TextEdit::multiline(&mut app.manual_import_text)
                            .font(egui::TextStyle::Monospace)
                            .desired_width(f32::INFINITY)
                            .desired_rows(18)
                            .hint_text("Paste JSON or XML response here..."),
                    );
                });
        });

    if !is_open {
        app.show_import_manual_modal = false;
    }
}

fn render_save_as_modal(app: &mut PromptForgeGuiApp, ctx: &egui::Context) {
    let mut is_open = true;
    egui::Window::new("Save Document As")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(450.0, 160.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label("File path (.prompt.json):");
            ui.text_edit_singleline(&mut app.save_as_path);
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Save").clicked() {
                    let path = PathBuf::from(app.save_as_path.trim());
                    match save_document(&path, app.service.document()) {
                        Ok(_) => {
                            app.service.mark_saved(path);
                            app.set_status("Saved successfully");
                            app.show_save_as_modal = false;
                        }
                        Err(e) => app.set_status(format!("Save error: {e}")),
                    }
                }
                if ui.button("Cancel").clicked() {
                    app.show_save_as_modal = false;
                }
            });
        });

    if !is_open {
        app.show_save_as_modal = false;
    }
}

fn render_open_modal(app: &mut PromptForgeGuiApp, ctx: &egui::Context) {
    let mut is_open = true;
    egui::Window::new("Open Document")
        .open(&mut is_open)
        .collapsible(false)
        .resizable(false)
        .default_size(Vec2::new(450.0, 160.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.label("File path to load:");
            ui.text_edit_singleline(&mut app.open_file_path);
            ui.add_space(8.0);

            ui.horizontal(|ui| {
                if ui.button("Open").clicked() {
                    let path = PathBuf::from(app.open_file_path.trim());
                    match load_document(&path) {
                        Ok(doc) => {
                            app.service.load_document(doc, Some(path));
                            app.set_status("Document opened successfully");
                            app.show_open_modal = false;
                        }
                        Err(e) => app.set_status(format!("Open error: {e}")),
                    }
                }
                if ui.button("Cancel").clicked() {
                    app.show_open_modal = false;
                }
            });
        });

    if !is_open {
        app.show_open_modal = false;
    }
}

fn render_settings_modal(app: &mut PromptForgeGuiApp, ctx: &egui::Context) {
    let mut is_open = true;
    egui::Window::new("Settings")
        .open(&mut is_open)
        .collapsible(false)
        .default_size(Vec2::new(480.0, 320.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.heading("OpenAI-Compatible Provider Defaults");
            ui.add_space(4.0);

            ui.horizontal(|ui| {
                ui.label("Base URL:");
                ui.text_edit_singleline(&mut app.config.openai_compatible.base_url);
            });
            ui.horizontal(|ui| {
                ui.label("Default Model:");
                ui.text_edit_singleline(&mut app.config.openai_compatible.model);
            });
            ui.horizontal(|ui| {
                ui.label("API Key Env Var:");
                ui.text_edit_singleline(&mut app.config.openai_compatible.api_key_env_var);
            });

            ui.add_space(8.0);
            if ui.button("Save Settings to Config").clicked() {
                match app.config.save() {
                    Ok(_) => app.set_status("Settings saved successfully"),
                    Err(e) => app.set_status(format!("Failed to save settings: {e}")),
                }
                app.show_settings_modal = false;
            }
        });

    if !is_open {
        app.show_settings_modal = false;
    }
}

fn render_help_modal(app: &mut PromptForgeGuiApp, ctx: &egui::Context) {
    let mut is_open = true;
    egui::Window::new("Help & Keyboard Shortcuts")
        .open(&mut is_open)
        .collapsible(false)
        .default_size(Vec2::new(480.0, 360.0))
        .anchor(egui::Align2::CENTER_CENTER, Vec2::ZERO)
        .show(ctx, |ui| {
            ui.heading("PromptForge Workflow");
            ui.label("1. Build your prompt by adding structured sections (role, context, task, constraints).");
            ui.label("2. Write your initial brief notes into each section card.");
            ui.label("3. Click 'Refine Prompt' (or Ctrl+R) to refine via an LLM or Manual mode.");
            ui.label("4. Review changes side-by-side, then accept or reject proposals.");
            ui.label("5. Copy clean or internal XML prompt for use with any AI agent.");
            ui.add_space(8.0);

            ui.heading("Shortcuts");
            ui.label("• Ctrl+S: Quick Save");
            ui.label("• Ctrl+O: Open Document");
            ui.label("• Ctrl+Z: Undo");
            ui.label("• Ctrl+Y: Redo");
            ui.label("• Ctrl+R: Refine Prompt");
        });

    if !is_open {
        app.show_help_modal = false;
    }
}
