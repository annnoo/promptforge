use crate::app::PromptForgeGuiApp;
use eframe::egui::{self, Color32, CornerRadius, Frame, Margin, RichText, Sense, Stroke, Vec2};
use prompt_application::commands::Command;
use prompt_core::{
    template::{ALL_PRESETS, ALL_TEMPLATES},
    RenderOptions, RenderStage,
};

/// Renders the top menu and quick action bar.
pub fn render_top_bar(app: &mut PromptForgeGuiApp, ui: &mut egui::Ui) {
    egui::MenuBar::new().ui(ui, |ui| {
        ui.menu_button("File", |ui| {
            if ui.button("New Document").clicked() {
                app.service.load_document(
                    ALL_TEMPLATES[0].instantiate(Some("New Prompt")),
                    None,
                );
                app.clear_status();
                ui.close();
            }
            if ui.button("Open... (Ctrl+O)").clicked() {
                app.show_open_modal = true;
                ui.close();
            }
            ui.separator();
            if ui.button("Save (Ctrl+S)").clicked() {
                app.save_current_or_prompt();
                ui.close();
            }
            if ui.button("Save As...").clicked() {
                app.show_save_as_modal = true;
                ui.close();
            }
            ui.separator();
            if ui.button("Quit").clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
                ui.close();
            }
        });

        ui.menu_button("Edit", |ui| {
            if ui
                .add_enabled(app.service.can_undo(), egui::Button::new("Undo (Ctrl+Z)"))
                .clicked()
            {
                let _ = app.service.execute(Command::Undo);
                ui.close();
            }
            if ui
                .add_enabled(app.service.can_redo(), egui::Button::new("Redo (Ctrl+Y)"))
                .clicked()
            {
                let _ = app.service.execute(Command::Redo);
                ui.close();
            }
        });

        ui.menu_button("Refinement", |ui| {
            if ui.button("Refine Prompt... (Ctrl+R)").clicked() {
                app.show_refine_modal = true;
                ui.close();
            }
            if ui.button("Export Manual Request...").clicked() {
                app.open_export_manual_modal();
                ui.close();
            }
            if ui.button("Import Model Response...").clicked() {
                app.show_import_manual_modal = true;
                ui.close();
            }
            ui.separator();
            let has_pending = app
                .service
                .state()
                .pending_refinements
                .as_ref()
                .is_some_and(|p| !p.is_empty());
            if ui
                .add_enabled(has_pending, egui::Button::new("Review Pending Changes..."))
                .clicked()
            {
                app.show_diff_modal = true;
                ui.close();
            }
        });

        ui.menu_button("Help", |ui| {
            if ui.button("Keybindings & Usage").clicked() {
                app.show_help_modal = true;
                ui.close();
            }
            if ui.button("Settings...").clicked() {
                app.show_settings_modal = true;
                ui.close();
            }
        });

        ui.separator();

        // Quick action buttons in top bar
        if ui
            .add_enabled(app.service.can_undo(), egui::Button::new("⮌ Undo"))
            .clicked()
        {
            let _ = app.service.execute(Command::Undo);
        }
        if ui
            .add_enabled(app.service.can_redo(), egui::Button::new("⮎ Redo"))
            .clicked()
        {
            let _ = app.service.execute(Command::Redo);
        }
        if ui.button("💾 Save").clicked() {
            app.save_current_or_prompt();
        }

        ui.separator();

        if ui
            .button(RichText::new("✨ Refine Prompt").color(Color32::from_rgb(120, 200, 255)).strong())
            .clicked()
        {
            app.show_refine_modal = true;
        }

        // Show pending changes badge if any exist
        if let Some(ref pending) = app.service.state().pending_refinements {
            if !pending.changes.is_empty() {
                let badge_text = format!("Review Refinements ({} pending)", pending.changes.len());
                if ui
                    .button(RichText::new(badge_text).color(Color32::from_rgb(255, 200, 80)).strong())
                    .clicked()
                {
                    app.show_diff_modal = true;
                }
            }
        }

        // Right side dirty flag and status
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if let Some((ref msg, _)) = app.status_message {
                ui.label(RichText::new(msg).color(Color32::YELLOW));
            } else if app.service.is_dirty() {
                ui.label(RichText::new("● Unsaved changes").color(Color32::from_rgb(255, 180, 60)));
            } else {
                ui.label(RichText::new("Saved").color(Color32::DARK_GRAY));
            }
        });
    });
}

/// Renders the left palette with presets, templates, and document metadata.
pub fn render_left_palette(app: &mut PromptForgeGuiApp, ui: &mut egui::Ui) {
    ui.heading("Palette & Meta");
    ui.add_space(6.0);

    // Document metadata section
    egui::CollapsingHeader::new(RichText::new("Document Metadata").strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.label(RichText::new("Title:").size(11.0).color(Color32::GRAY));
            let mut title = app.service.document().title.clone();
            if ui.text_edit_singleline(&mut title).changed() {
                let _ = app.service.execute(Command::UpdateTitle { title });
            }

            ui.add_space(4.0);
            ui.label(RichText::new("Description:").size(11.0).color(Color32::GRAY));
            let mut desc = app.service.document().description.clone();
            if ui
                .add(
                    egui::TextEdit::multiline(&mut desc)
                        .desired_rows(2)
                        .desired_width(f32::INFINITY),
                )
                .changed()
            {
                let _ = app.service.execute(Command::UpdateDescription { description: desc });
            }

            ui.add_space(4.0);
            ui.label(
                RichText::new(format!(
                    "Schema v{} • {} section(s)",
                    app.service.document().schema_version,
                    app.service.document().sections.len()
                ))
                .size(11.0)
                .color(Color32::DARK_GRAY),
            );
        });

    ui.separator();

    // Section presets
    egui::CollapsingHeader::new(RichText::new("Insert Preset Section").strong())
        .default_open(true)
        .show(ui, |ui| {
            egui::ScrollArea::vertical()
                .id_salt("preset_scroll")
                .max_height(240.0)
                .show(ui, |ui| {
                    for preset in ALL_PRESETS {
                        ui.horizontal(|ui| {
                            let btn = egui::Button::new(format!("+ {}", preset.name))
                                .min_size(Vec2::new(ui.available_width(), 22.0));
                            if ui.add(btn).on_hover_text(preset.description).clicked() {
                                let _ = app.service.execute(Command::AddPreset {
                                    preset_tag: preset.tag.to_string(),
                                });
                            }
                        });
                        ui.add_space(2.0);
                    }
                });
        });

    ui.separator();

    // Custom tag section
    egui::CollapsingHeader::new(RichText::new("Custom Section").strong())
        .default_open(true)
        .show(ui, |ui| {
            ui.label(RichText::new("XML Tag Name:").size(11.0).color(Color32::GRAY));
            ui.horizontal(|ui| {
                ui.text_edit_singleline(&mut app.custom_tag_input);
                if ui.button("Add").clicked() && !app.custom_tag_input.trim().is_empty() {
                    let tag = app.custom_tag_input.trim().to_string();
                    match app.service.execute(Command::AddSection {
                        tag,
                        brief: String::new(),
                    }) {
                        Ok(_) => {
                            app.custom_tag_input.clear();
                        }
                        Err(e) => {
                            app.set_status(format!("Invalid tag: {e}"));
                        }
                    }
                }
            });
        });

    ui.separator();

    // Starter templates
    egui::CollapsingHeader::new(RichText::new("Starter Templates").strong())
        .default_open(false)
        .show(ui, |ui| {
            for template in ALL_TEMPLATES {
                if ui
                    .button(template.name)
                    .on_hover_text(template.description)
                    .clicked()
                {
                    app.service.load_document(template.instantiate(None), None);
                }
                ui.add_space(2.0);
            }
        });
}

/// Renders the center editor containing reorderable section cards.
pub fn render_center_editor(app: &mut PromptForgeGuiApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("Structured Prompt Editor");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("+ Add Preset").clicked() {
                let _ = app.service.execute(Command::AddPreset {
                    preset_tag: "task".to_string(),
                });
            }
        });
    });
    ui.add_space(6.0);

    let doc = app.service.document().clone();
    let total_sections = doc.sections.len();

    if total_sections == 0 {
        ui.vertical_centered(|ui| {
            ui.add_space(40.0);
            ui.label(RichText::new("No prompt sections yet.").size(16.0).color(Color32::GRAY));
            ui.label(RichText::new("Click any preset on the left palette to begin building your prompt.").color(Color32::DARK_GRAY));
            ui.add_space(10.0);
            if ui.button("Add General Task Template").clicked() {
                app.service.load_document(ALL_TEMPLATES[0].instantiate(None), None);
            }
        });
        return;
    }

    // ScrollArea for cards
    egui::ScrollArea::vertical()
        .id_salt("cards_scroll")
        .show(ui, |ui| {
            let mut move_op: Option<(usize, usize)> = None;
            let mut duplicate_id: Option<uuid::Uuid> = None;
            let mut remove_id: Option<uuid::Uuid> = None;
            let mut toggle_lock_id: Option<uuid::Uuid> = None;
            let mut toggle_enabled_id: Option<uuid::Uuid> = None;
            let mut text_update: Option<(uuid::Uuid, String)> = None;
            let mut rename_tag: Option<(uuid::Uuid, String)> = None;
            let mut clear_refinement_id: Option<uuid::Uuid> = None;

            for (idx, section) in doc.sections.iter().enumerate() {
                let _card_id = ui.make_persistent_id(format!("section_card_{}", section.id));
                let is_locked = section.locked;
                let is_enabled = section.enabled;

                let border_stroke = if is_locked {
                    Stroke::new(1.0, Color32::from_rgb(180, 100, 60))
                } else if !is_enabled {
                    Stroke::new(1.0, Color32::from_rgb(60, 65, 75))
                } else if section.has_refinement() {
                    Stroke::new(1.0, Color32::from_rgb(60, 160, 100))
                } else {
                    Stroke::new(1.0, Color32::from_rgb(50, 60, 80))
                };

                let card_bg = if is_locked {
                    Color32::from_rgb(26, 22, 22)
                } else if !is_enabled {
                    Color32::from_rgb(20, 22, 26)
                } else {
                    Color32::from_rgb(28, 32, 42)
                };

                // Render card frame
                Frame::group(ui.style())
                    .fill(card_bg)
                    .stroke(border_stroke)
                    .corner_radius(CornerRadius::same(6))
                    .inner_margin(Margin::same(10))
                    .show(ui, |ui| {
                        // Card Header
                        ui.horizontal(|ui| {
                            // Drag handle
                            let handle_response = ui.add(
                                egui::Button::new(RichText::new(" ⠿ ").size(16.0).color(Color32::GRAY))
                                    .frame(false)
                                    .sense(Sense::drag()),
                            );

                            if handle_response.drag_started() {
                                app.dragging_section_id = Some(section.id);
                            }

                            if handle_response.dragged() {
                                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::Grabbing);
                            }

                            // Tag input / label
                            ui.label(RichText::new("<").color(Color32::CYAN));
                            let mut tag_buf = section.tag.clone();
                            let tag_edit = ui.add(
                                egui::TextEdit::singleline(&mut tag_buf)
                                    .desired_width(120.0)
                                    .font(egui::TextStyle::Monospace),
                            );
                            if tag_edit.lost_focus() && tag_buf != section.tag {
                                rename_tag = Some((section.id, tag_buf));
                            }
                            ui.label(RichText::new(">").color(Color32::CYAN));

                            // Status indicators
                            if is_locked {
                                ui.label(RichText::new("[LOCKED]").size(11.0).color(Color32::from_rgb(255, 140, 60)));
                            }
                            if section.has_refinement() {
                                ui.label(RichText::new("[REFINED]").size(11.0).color(Color32::from_rgb(80, 220, 120)));
                            }

                            // Right aligned card controls
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                // Delete
                                if ui
                                    .button(RichText::new("🗑").color(Color32::from_rgb(255, 100, 100)))
                                    .on_hover_text("Delete section")
                                    .clicked()
                                {
                                    remove_id = Some(section.id);
                                }

                                // Duplicate
                                if ui
                                    .button("⎘")
                                    .on_hover_text("Duplicate section")
                                    .clicked()
                                {
                                    duplicate_id = Some(section.id);
                                }

                                // Move Down
                                if ui
                                    .add_enabled(idx + 1 < total_sections, egui::Button::new("▼"))
                                    .on_hover_text("Move section down")
                                    .clicked()
                                {
                                    move_op = Some((idx, idx + 1));
                                }

                                // Move Up
                                if ui
                                    .add_enabled(idx > 0, egui::Button::new("▲"))
                                    .on_hover_text("Move section up")
                                    .clicked()
                                {
                                    move_op = Some((idx, idx - 1));
                                }

                                // Lock toggle
                                let lock_icon = if is_locked { "🔒 Unlock" } else { "🔓 Lock" };
                                if ui
                                    .button(lock_icon)
                                    .on_hover_text("Lock section against edits or refinement")
                                    .clicked()
                                {
                                    toggle_lock_id = Some(section.id);
                                }

                                // Enabled checkbox
                                let mut enabled_val = is_enabled;
                                if ui.checkbox(&mut enabled_val, "Enabled").changed() {
                                    toggle_enabled_id = Some(section.id);
                                }
                            });
                        });

                        ui.add_space(4.0);

                        // Card Brief Multiline Edit
                        let mut brief_buf = section.brief.clone();
                        let edit_response = ui.add_enabled(
                            !is_locked,
                            egui::TextEdit::multiline(&mut brief_buf)
                                .desired_rows(3)
                                .desired_width(f32::INFINITY)
                                .hint_text("Enter prompt instructions or draft notes for this section..."),
                        );

                        if edit_response.changed() {
                            text_update = Some((section.id, brief_buf));
                        }
                        if edit_response.lost_focus() {
                            app.service.commit_checkpoint();
                        }

                        // If accepted refinement exists, display it
                        if let Some(ref refined_text) = section.refined {
                            ui.add_space(4.0);
                            ui.separator();
                            ui.horizontal(|ui| {
                                ui.label(RichText::new("Accepted Refinement:").size(11.0).color(Color32::from_rgb(80, 220, 120)).strong());
                                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                    if ui.button(RichText::new("Revert to Brief").size(11.0)).clicked() {
                                        clear_refinement_id = Some(section.id);
                                    }
                                });
                            });
                            Frame::new()
                                .fill(Color32::from_rgb(22, 28, 24))
                                .corner_radius(CornerRadius::same(4))
                                .inner_margin(Margin::same(6))
                                .show(ui, |ui| {
                                    ui.label(RichText::new(refined_text).color(Color32::from_rgb(210, 240, 220)));
                                });
                        }
                    });

                // Drop target detection for drag and drop
                if let Some(dragging_id) = app.dragging_section_id {
                    if dragging_id != section.id {
                        let rect = ui.min_rect();
                        if let Some(pos) = ui.input(|i| i.pointer.hover_pos()) {
                            if rect.contains(pos) {
                                // User hovered this card while dragging
                                app.drag_target_index = Some(idx);
                            }
                        }
                    }
                }

                ui.add_space(6.0);
            }

            // Drop release handling
            if ui.input(|i| i.pointer.any_released()) {
                if let (Some(drag_id), Some(target_idx)) = (app.dragging_section_id.take(), app.drag_target_index.take()) {
                    let _ = app.service.execute(Command::MoveSection {
                        id: drag_id,
                        to: target_idx,
                    });
                }
                app.dragging_section_id = None;
                app.drag_target_index = None;
            }

            // Apply queued actions
            if let Some((from, to)) = move_op {
                if let Some(sec) = app.service.document().sections.get(from) {
                    let id = sec.id;
                    let _ = app.service.execute(Command::MoveSection { id, to });
                }
            }
            if let Some(id) = duplicate_id {
                let _ = app.service.execute(Command::DuplicateSection { id });
            }
            if let Some(id) = remove_id {
                let _ = app.service.execute(Command::RemoveSection { id });
            }
            if let Some(id) = toggle_lock_id {
                let _ = app.service.execute(Command::ToggleLock { id });
            }
            if let Some(id) = toggle_enabled_id {
                let _ = app.service.execute(Command::ToggleEnabled { id });
            }
            if let Some((id, text)) = text_update {
                let _ = app.service.update_brief_live(id, text);
            }
            if let Some((id, tag)) = rename_tag {
                match app.service.execute(Command::RenameSection { id, tag }) {
                    Ok(_) => app.set_status("Tag renamed"),
                    Err(e) => app.set_status(format!("Rename error: {e}")),
                }
            }
            if let Some(id) = clear_refinement_id {
                let _ = app.service.execute(Command::ClearRefinement { id });
            }
        });
}

/// Renders the live right-hand preview with XML rendering, controls, and validation feedback.
pub fn render_right_preview(app: &mut PromptForgeGuiApp, ui: &mut egui::Ui) {
    ui.horizontal(|ui| {
        ui.heading("Live XML Preview");
        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
            if ui.button("📋 Copy XML").clicked() {
                let options = RenderOptions {
                    stage: app.preview_stage,
                    include_ids: app.preview_include_ids,
                    pretty: true,
                };
                if let Ok(xml) = app.service.render_xml(options) {
                    app.copy_to_clipboard(&xml);
                }
            }
        });
    });
    ui.add_space(4.0);

    // Options toolbar
    ui.horizontal(|ui| {
        ui.label(RichText::new("Stage:").size(12.0).color(Color32::GRAY));
        ui.selectable_value(&mut app.preview_stage, RenderStage::Draft, "Draft");
        ui.selectable_value(&mut app.preview_stage, RenderStage::Final, "Final");

        ui.separator();
        ui.checkbox(&mut app.preview_include_ids, "Include IDs");
    });
    ui.add_space(4.0);

    // Render XML
    let options = RenderOptions {
        stage: app.preview_stage,
        include_ids: app.preview_include_ids,
        pretty: true,
    };

    match app.service.render_xml(options) {
        Ok(xml) => {
            ui.label(RichText::new("✔ Valid XML").size(11.0).color(Color32::from_rgb(80, 200, 120)));
            ui.add_space(2.0);

            Frame::new()
                .fill(Color32::from_rgb(14, 16, 20))
                .corner_radius(CornerRadius::same(4))
                .stroke(Stroke::new(1.0, Color32::from_rgb(35, 40, 52)))
                .inner_margin(Margin::same(8))
                .show(ui, |ui| {
                    egui::ScrollArea::both()
                        .id_salt("xml_preview_scroll")
                        .show(ui, |ui| {
                            ui.add(
                                egui::TextEdit::multiline(&mut xml.as_str())
                                    .font(egui::TextStyle::Monospace)
                                    .desired_width(f32::INFINITY)
                                    .interactive(false),
                            );
                        });
                });
        }
        Err(err) => {
            ui.label(RichText::new(format!("⚠ Validation Error: {err}")).size(12.0).color(Color32::from_rgb(255, 100, 100)));
        }
    }
}
