use crate::editor::TextEditor;
use crate::terminal::{init_terminal, restore_terminal};
use crate::view::render_ui;
use arboard::Clipboard;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use prompt_application::{
    commands::Command,
    state::{PendingRefinements, ProposedSectionChange, SuggestedSection},
    ApplicationService,
};
use prompt_core::{
    template::{ALL_PRESETS, ALL_TEMPLATES},
    PromptDocument, RenderOptions, RenderStage,
};
use prompt_persistence::{load_document, save_document, AppConfig, ProviderType};
use prompt_refinement::{
    generate_manual_request, parse_manual_response, OpenAiCompatibleProvider, RefinementChangeset,
    RefinementEngine, RefinementMode,
};
use std::path::PathBuf;
use std::sync::mpsc::{channel, Receiver};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActivePanel {
    Sections,
    Editor,
    XmlPreview,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    Help,
    AddSection {
        selected_index: usize,
        is_custom: bool,
        custom_tag: String,
    },
    RenameSection {
        new_tag: String,
    },
    ConfirmDelete,
    SaveFile {
        path_input: String,
    },
    OpenFile {
        path_input: String,
    },
    RefinePrompt {
        selected_mode: RefinementMode,
        selected_provider: ProviderType,
    },
    ReviewRefinements {
        selected_change_index: usize,
    },
    ExportManualRequest {
        path_or_info: String,
    },
    ImportManualResponse {
        editor: TextEditor,
    },
    ConfirmQuit,
}

pub struct AppState {
    pub service: ApplicationService,
    pub config: AppConfig,
    pub selected_section_index: usize,
    pub active_panel: ActivePanel,
    pub editing_brief: bool,
    pub editor: TextEditor,
    pub active_modal: ActiveModal,
    pub preview_stage: RenderStage,
    pub preview_include_ids: bool,
    pub status_message: Option<String>,
    pub should_quit: bool,
    // Async refinement channel
    refinement_rx: Option<Receiver<Result<RefinementChangeset, String>>>,
}

impl AppState {
    pub fn new(initial_doc: Option<PromptDocument>, file_path: Option<PathBuf>) -> Self {
        let config = AppConfig::load_or_default();
        let doc = initial_doc.unwrap_or_else(|| {
            ALL_TEMPLATES[0].instantiate(Some("New Prompt Document"))
        });
        let mut service = ApplicationService::new(doc);
        if let Some(ref path) = file_path {
            service.mark_saved(path.clone());
        }

        Self {
            service,
            config,
            selected_section_index: 0,
            active_panel: ActivePanel::Sections,
            editing_brief: false,
            editor: TextEditor::new(),
            active_modal: ActiveModal::None,
            preview_stage: RenderStage::Draft,
            preview_include_ids: true,
            status_message: None,
            should_quit: false,
            refinement_rx: None,
        }
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.status_message = Some(msg.into());
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

    pub fn enter_inline_edit(&mut self) {
        if let Some(sec) = self.service.document().sections.get(self.selected_section_index) {
            if sec.locked {
                self.set_status(format!("Section <{}> is locked against editing", sec.tag));
                return;
            }
            self.editor = TextEditor::from_text(&sec.brief);
            self.editing_brief = true;
            self.clear_status();
        }
    }

    pub fn save_inline_edit(&mut self) {
        if self.editing_brief {
            if let Some(sec) = self.service.document().sections.get(self.selected_section_index) {
                let id = sec.id;
                let text = self.editor.get_text();
                let _ = self.service.execute(Command::UpdateBrief { id, text });
            }
            self.editing_brief = false;
            self.set_status("Brief updated");
        }
    }

    pub fn cancel_inline_edit(&mut self) {
        self.editing_brief = false;
        self.clear_status();
    }
}

/// Runs the interactive terminal UI loop.
pub async fn run_tui(initial_file: Option<PathBuf>) -> anyhow::Result<()> {
    let initial_doc = if let Some(ref path) = initial_file {
        if path.exists() {
            Some(load_document(path)?)
        } else {
            None
        }
    } else {
        None
    };

    let mut app = AppState::new(initial_doc, initial_file);
    let mut terminal = init_terminal()?;

    let res = run_loop(&mut terminal, &mut app).await;

    restore_terminal(&mut terminal)?;
    res
}

async fn run_loop(
    terminal: &mut crate::terminal::TuiTerminal,
    app: &mut AppState,
) -> anyhow::Result<()> {
    loop {
        // Poll for async refinement results
        if let Some(ref rx) = app.refinement_rx {
            if let Ok(res) = rx.try_recv() {
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
                        app.service.set_pending_refinements(pending);
                        app.active_modal = ActiveModal::ReviewRefinements {
                            selected_change_index: 0,
                        };
                        app.set_status("Refinements received! Review changes.");
                    }
                    Err(err) => {
                        app.set_status(format!("Refinement failed: {err}"));
                    }
                }
                app.refinement_rx = None;
            }
        }

        terminal.draw(|frame| render_ui(frame, app))?;

        if app.should_quit {
            break;
        }

        if event::poll(Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(app, key);
                }
            }
        }
    }
    Ok(())
}

fn handle_key(app: &mut AppState, key: KeyEvent) {
    if app.active_modal != ActiveModal::None {
        handle_modal_key(app, key);
        return;
    }

    if app.editing_brief {
        handle_editor_key(app, key);
        return;
    }

    // Normal command mode keys
    match (key.code, key.modifiers) {
        // Quit
        (KeyCode::Char('q'), _) => {
            if app.service.is_dirty() {
                app.active_modal = ActiveModal::ConfirmQuit;
            } else {
                app.should_quit = true;
            }
        }

        // Help
        (KeyCode::Char('?'), _) => {
            app.active_modal = ActiveModal::Help;
        }

        // Navigation
        (KeyCode::Char('j') | KeyCode::Down, KeyModifiers::NONE) => {
            let len = app.service.document().sections.len();
            if len > 0 && app.selected_section_index + 1 < len {
                app.selected_section_index += 1;
            }
        }
        (KeyCode::Char('k') | KeyCode::Up, KeyModifiers::NONE) => {
            if app.selected_section_index > 0 {
                app.selected_section_index -= 1;
            }
        }

        // Move section
        (KeyCode::Char('J') | KeyCode::Down, KeyModifiers::SHIFT) => {
            if let Some(sec) = app.service.document().sections.get(app.selected_section_index) {
                let id = sec.id;
                let _ = app.service.execute(Command::MoveDown { id });
                let len = app.service.document().sections.len();
                if app.selected_section_index + 1 < len {
                    app.selected_section_index += 1;
                }
            }
        }
        (KeyCode::Char('K') | KeyCode::Up, KeyModifiers::SHIFT) => {
            if let Some(sec) = app.service.document().sections.get(app.selected_section_index) {
                let id = sec.id;
                let _ = app.service.execute(Command::MoveUp { id });
                if app.selected_section_index > 0 {
                    app.selected_section_index -= 1;
                }
            }
        }

        // Add Section
        (KeyCode::Char('a'), KeyModifiers::NONE) => {
            app.active_modal = ActiveModal::AddSection {
                selected_index: 0,
                is_custom: false,
                custom_tag: String::new(),
            };
        }

        // Edit Brief
        (KeyCode::Char('e') | KeyCode::Enter, KeyModifiers::NONE) => {
            app.enter_inline_edit();
        }

        // Rename Tag
        (KeyCode::Char('n') | KeyCode::Char('R'), KeyModifiers::NONE) => {
            if let Some(sec) = app.service.document().sections.get(app.selected_section_index) {
                app.active_modal = ActiveModal::RenameSection {
                    new_tag: sec.tag.clone(),
                };
            }
        }

        // Delete
        (KeyCode::Char('d'), KeyModifiers::NONE) => {
            if !app.service.document().sections.is_empty() {
                app.active_modal = ActiveModal::ConfirmDelete;
            }
        }

        // Duplicate
        (KeyCode::Char('D') | KeyCode::Char('c'), KeyModifiers::NONE) => {
            if let Some(sec) = app.service.document().sections.get(app.selected_section_index) {
                let id = sec.id;
                let _ = app.service.execute(Command::DuplicateSection { id });
                app.set_status("Section duplicated");
            }
        }

        // Toggle Lock
        (KeyCode::Char('l'), KeyModifiers::NONE) => {
            if let Some(sec) = app.service.document().sections.get(app.selected_section_index) {
                let id = sec.id;
                let _ = app.service.execute(Command::ToggleLock { id });
                let is_locked = app.service.document().section(id).is_some_and(|s| s.locked);
                app.set_status(if is_locked { "Section locked" } else { "Section unlocked" });
            }
        }

        // Toggle Enabled
        (KeyCode::Char('x') | KeyCode::Char(' '), KeyModifiers::NONE) => {
            if let Some(sec) = app.service.document().sections.get(app.selected_section_index) {
                let id = sec.id;
                let _ = app.service.execute(Command::ToggleEnabled { id });
            }
        }

        // Preview Stage Toggle
        (KeyCode::Char('v'), KeyModifiers::NONE) => {
            app.preview_stage = match app.preview_stage {
                RenderStage::Draft => RenderStage::Final,
                RenderStage::Final => RenderStage::Draft,
            };
            app.set_status(format!("Preview stage: {:?}", app.preview_stage));
        }

        // Preview IDs Toggle
        (KeyCode::Char('i'), KeyModifiers::NONE) => {
            app.preview_include_ids = !app.preview_include_ids;
            app.set_status(if app.preview_include_ids {
                "XML preview: with internal IDs"
            } else {
                "XML preview: clean (no IDs)"
            });
        }

        // Copy XML to clipboard
        (KeyCode::Char('y'), KeyModifiers::NONE) => {
            let options = RenderOptions {
                stage: app.preview_stage,
                include_ids: app.preview_include_ids,
                pretty: true,
            };
            if let Ok(xml) = app.service.render_xml(options) {
                app.copy_to_clipboard(&xml);
            }
        }

        // Undo
        (KeyCode::Char('u'), KeyModifiers::NONE) => {
            if app.service.can_undo() {
                let _ = app.service.execute(Command::Undo);
                app.set_status("Undo successful");
            } else {
                app.set_status("Nothing to undo");
            }
        }

        // Redo
        (KeyCode::Char('r'), KeyModifiers::CONTROL) => {
            if app.service.can_redo() {
                let _ = app.service.execute(Command::Redo);
                app.set_status("Redo successful");
            } else {
                app.set_status("Nothing to redo");
            }
        }

        // Refine Prompt
        (KeyCode::Char('r'), KeyModifiers::NONE) => {
            app.active_modal = ActiveModal::RefinePrompt {
                selected_mode: RefinementMode::Conservative,
                selected_provider: app.config.active_provider,
            };
        }

        // Export manual request
        (KeyCode::Char('m'), KeyModifiers::NONE) => {
            let req = generate_manual_request(app.service.document(), RefinementMode::Conservative)
                .unwrap_or_default();
            app.copy_to_clipboard(&req);
            app.active_modal = ActiveModal::ExportManualRequest {
                path_or_info: "Refinement request has been generated and copied to system clipboard!\n\nPaste it into ChatGPT, Claude, or any LLM, then use 'I' (Shift+i) to import the response."
                    .to_string(),
            };
        }

        // Import manual response
        (KeyCode::Char('I'), KeyModifiers::SHIFT) | (KeyCode::Char('I'), KeyModifiers::NONE) => {
            app.active_modal = ActiveModal::ImportManualResponse {
                editor: TextEditor::new(),
            };
        }

        // Save
        (KeyCode::Char('s'), KeyModifiers::NONE) => {
            if let Some(path) = app.service.current_file_path() {
                let path = path.to_path_buf();
                match save_document(&path, app.service.document()) {
                    Ok(_) => {
                        app.service.mark_saved(path);
                        app.set_status("Document saved successfully");
                    }
                    Err(e) => app.set_status(format!("Save error: {e}")),
                }
            } else {
                app.active_modal = ActiveModal::SaveFile {
                    path_input: format!("{}.prompt.json", app.service.document().title.to_lowercase().replace(' ', "_")),
                };
            }
        }

        // Open
        (KeyCode::Char('o'), KeyModifiers::NONE) => {
            app.active_modal = ActiveModal::OpenFile {
                path_input: String::new(),
            };
        }

        _ => {}
    }
}

fn handle_editor_key(app: &mut AppState, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.save_inline_edit();
        }
        KeyCode::Enter => {
            app.editor.insert_newline();
        }
        KeyCode::Backspace => {
            app.editor.backspace();
        }
        KeyCode::Delete => {
            app.editor.delete_forward();
        }
        KeyCode::Left => {
            app.editor.move_left();
        }
        KeyCode::Right => {
            app.editor.move_right();
        }
        KeyCode::Up => {
            app.editor.move_up();
        }
        KeyCode::Down => {
            app.editor.move_down();
        }
        KeyCode::Home => {
            app.editor.move_home();
        }
        KeyCode::End => {
            app.editor.move_end();
        }
        KeyCode::Char(c) => {
            if key.modifiers == KeyModifiers::CONTROL && c == 's' {
                app.save_inline_edit();
            } else {
                app.editor.insert_char(c);
            }
        }
        _ => {}
    }
}

fn handle_modal_key(app: &mut AppState, key: KeyEvent) {
    let modal = std::mem::replace(&mut app.active_modal, ActiveModal::None);

    match modal {
        ActiveModal::None => {}
        ActiveModal::Help => {
            if !(key.code == KeyCode::Esc || key.code == KeyCode::Char('q') || key.code == KeyCode::Enter) {
                app.active_modal = ActiveModal::Help;
            }
        }
        ActiveModal::AddSection {
            mut selected_index,
            mut is_custom,
            mut custom_tag,
        } => match key.code {
            KeyCode::Esc => {}
            KeyCode::Tab => {
                is_custom = !is_custom;
                app.active_modal = ActiveModal::AddSection {
                    selected_index,
                    is_custom,
                    custom_tag,
                };
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if !is_custom && selected_index > 0 {
                    selected_index -= 1;
                }
                app.active_modal = ActiveModal::AddSection {
                    selected_index,
                    is_custom,
                    custom_tag,
                };
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if !is_custom && selected_index + 1 < ALL_PRESETS.len() {
                    selected_index += 1;
                }
                app.active_modal = ActiveModal::AddSection {
                    selected_index,
                    is_custom,
                    custom_tag,
                };
            }
            KeyCode::Backspace if is_custom => {
                custom_tag.pop();
                app.active_modal = ActiveModal::AddSection {
                    selected_index,
                    is_custom,
                    custom_tag,
                };
            }
            KeyCode::Char(c) if is_custom => {
                custom_tag.push(c);
                app.active_modal = ActiveModal::AddSection {
                    selected_index,
                    is_custom,
                    custom_tag,
                };
            }
            KeyCode::Enter => {
                if is_custom {
                    if !custom_tag.is_empty() {
                        let _ = app.service.execute(Command::AddSection {
                            tag: custom_tag,
                            brief: String::new(),
                        });
                        app.selected_section_index = app.service.document().sections.len().saturating_sub(1);
                        app.enter_inline_edit();
                    }
                } else {
                    let preset = ALL_PRESETS[selected_index];
                    let _ = app.service.execute(Command::AddPreset {
                        preset_tag: preset.tag.to_string(),
                    });
                    app.selected_section_index = app.service.document().sections.len().saturating_sub(1);
                }
            }
            _ => {
                app.active_modal = ActiveModal::AddSection {
                    selected_index,
                    is_custom,
                    custom_tag,
                };
            }
        },
        ActiveModal::RenameSection { mut new_tag } => match key.code {
            KeyCode::Esc => {}
            KeyCode::Backspace => {
                new_tag.pop();
                app.active_modal = ActiveModal::RenameSection { new_tag };
            }
            KeyCode::Char(c) => {
                new_tag.push(c);
                app.active_modal = ActiveModal::RenameSection { new_tag };
            }
            KeyCode::Enter => {
                if let Some(sec) = app.service.document().sections.get(app.selected_section_index) {
                    let id = sec.id;
                    let tag = new_tag.clone();
                    match app.service.execute(Command::RenameSection { id, tag }) {
                        Ok(_) => {
                            app.set_status("Tag renamed successfully");
                        }
                        Err(e) => {
                            app.set_status(format!("Rename error: {e}"));
                            app.active_modal = ActiveModal::RenameSection { new_tag };
                        }
                    }
                }
            }
            _ => {
                app.active_modal = ActiveModal::RenameSection { new_tag };
            }
        },
        ActiveModal::ConfirmDelete => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(sec) = app.service.document().sections.get(app.selected_section_index) {
                    let id = sec.id;
                    let _ = app.service.execute(Command::RemoveSection { id });
                    if app.selected_section_index >= app.service.document().sections.len() {
                        app.selected_section_index = app.service.document().sections.len().saturating_sub(1);
                    }
                    app.set_status("Section deleted");
                }
            }
            _ => {}
        },
        ActiveModal::SaveFile { mut path_input } => match key.code {
            KeyCode::Esc => {}
            KeyCode::Backspace => {
                path_input.pop();
                app.active_modal = ActiveModal::SaveFile { path_input };
            }
            KeyCode::Char(c) => {
                path_input.push(c);
                app.active_modal = ActiveModal::SaveFile { path_input };
            }
            KeyCode::Enter => {
                let path = PathBuf::from(path_input.trim());
                match save_document(&path, app.service.document()) {
                    Ok(_) => {
                        app.service.mark_saved(path);
                        app.set_status("Document saved successfully");
                    }
                    Err(e) => {
                        app.set_status(format!("Save error: {e}"));
                        app.active_modal = ActiveModal::SaveFile { path_input };
                    }
                }
            }
            _ => {
                app.active_modal = ActiveModal::SaveFile { path_input };
            }
        },
        ActiveModal::OpenFile { mut path_input } => match key.code {
            KeyCode::Esc => {}
            KeyCode::Backspace => {
                path_input.pop();
                app.active_modal = ActiveModal::OpenFile { path_input };
            }
            KeyCode::Char(c) => {
                path_input.push(c);
                app.active_modal = ActiveModal::OpenFile { path_input };
            }
            KeyCode::Enter => {
                let path = PathBuf::from(path_input.trim());
                match load_document(&path) {
                    Ok(doc) => {
                        app.service.load_document(doc, Some(path));
                        app.selected_section_index = 0;
                        app.set_status("Document loaded successfully");
                    }
                    Err(e) => {
                        app.set_status(format!("Open error: {e}"));
                        app.active_modal = ActiveModal::OpenFile { path_input };
                    }
                }
            }
            _ => {
                app.active_modal = ActiveModal::OpenFile { path_input };
            }
        },
        ActiveModal::RefinePrompt {
            mut selected_mode,
            mut selected_provider,
        } => match key.code {
            KeyCode::Esc => {}
            KeyCode::Char('1') => {
                selected_mode = RefinementMode::Conservative;
                app.active_modal = ActiveModal::RefinePrompt {
                    selected_mode,
                    selected_provider,
                };
            }
            KeyCode::Char('2') => {
                selected_mode = RefinementMode::Expand;
                app.active_modal = ActiveModal::RefinePrompt {
                    selected_mode,
                    selected_provider,
                };
            }
            KeyCode::Char('3') => {
                selected_mode = RefinementMode::Critique;
                app.active_modal = ActiveModal::RefinePrompt {
                    selected_mode,
                    selected_provider,
                };
            }
            KeyCode::Tab => {
                selected_provider = match selected_provider {
                    ProviderType::Manual => ProviderType::OpenAiCompatible,
                    ProviderType::OpenAiCompatible => ProviderType::Manual,
                };
                app.active_modal = ActiveModal::RefinePrompt {
                    selected_mode,
                    selected_provider,
                };
            }
            KeyCode::Enter => {
                if selected_provider == ProviderType::Manual {
                    let req = generate_manual_request(app.service.document(), selected_mode)
                        .unwrap_or_default();
                    app.copy_to_clipboard(&req);
                    app.active_modal = ActiveModal::ExportManualRequest {
                        path_or_info: "Manual refinement request copied to clipboard!\n\nPaste into your LLM, then press 'I' (Shift+i) in PromptForge to import the result."
                            .to_string(),
                    };
                } else {
                    let (tx, rx) = channel();
                    app.refinement_rx = Some(rx);
                    app.set_status(format!("Sending refinement request to OpenAI ({selected_mode:?})..."));

                    let doc = app.service.document().clone();
                    let cfg = app.config.openai_compatible.clone();

                    tokio::spawn(async move {
                        let provider = OpenAiCompatibleProvider::new(
                            cfg.base_url,
                            cfg.model,
                            cfg.api_key_env_var,
                            cfg.timeout_seconds,
                        );
                        let engine = RefinementEngine::new();
                        let result = engine.execute(&provider, &doc, selected_mode).await;
                        let _ = tx.send(result.map_err(|e| e.to_string()));
                    });
                }
            }
            _ => {
                app.active_modal = ActiveModal::RefinePrompt {
                    selected_mode,
                    selected_provider,
                };
            }
        },
        ActiveModal::ReviewRefinements {
            mut selected_change_index,
        } => match key.code {
            KeyCode::Esc => {}
            KeyCode::Left | KeyCode::Char('h') => {
                selected_change_index = selected_change_index.saturating_sub(1);
                app.active_modal = ActiveModal::ReviewRefinements { selected_change_index };
            }
            KeyCode::Right | KeyCode::Char('l') => {
                if let Some(pending) = &app.service.state().pending_refinements {
                    if selected_change_index + 1 < pending.changes.len() {
                        selected_change_index += 1;
                    }
                }
                app.active_modal = ActiveModal::ReviewRefinements { selected_change_index };
            }
            KeyCode::Char('y') | KeyCode::Enter => {
                if let Some(pending) = &app.service.state().pending_refinements {
                    if let Some(change) = pending.changes.get(selected_change_index) {
                        let id = change.section_id;
                        let _ = app.service.execute(Command::ApplyRefinement { id });
                        app.set_status("Refinement accepted for section");
                    }
                }
                if let Some(pending) = &app.service.state().pending_refinements {
                    if !pending.changes.is_empty() {
                        if selected_change_index >= pending.changes.len() {
                            selected_change_index = pending.changes.len().saturating_sub(1);
                        }
                        app.active_modal = ActiveModal::ReviewRefinements { selected_change_index };
                    }
                }
            }
            KeyCode::Char('n') => {
                if let Some(pending) = &app.service.state().pending_refinements {
                    if let Some(change) = pending.changes.get(selected_change_index) {
                        let id = change.section_id;
                        let _ = app.service.execute(Command::RejectRefinement { id });
                        app.set_status("Refinement rejected for section");
                    }
                }
                if let Some(pending) = &app.service.state().pending_refinements {
                    if !pending.changes.is_empty() {
                        if selected_change_index >= pending.changes.len() {
                            selected_change_index = pending.changes.len().saturating_sub(1);
                        }
                        app.active_modal = ActiveModal::ReviewRefinements { selected_change_index };
                    }
                }
            }
            KeyCode::Char('A') => {
                let _ = app.service.execute(Command::ApplyAllRefinements);
                app.set_status("All refinements accepted!");
            }
            KeyCode::Char('X') => {
                let _ = app.service.execute(Command::RejectAllRefinements);
                app.set_status("All refinements rejected");
            }
            _ => {
                app.active_modal = ActiveModal::ReviewRefinements { selected_change_index };
            }
        },
        ActiveModal::ExportManualRequest { path_or_info } => match key.code {
            KeyCode::Esc | KeyCode::Enter | KeyCode::Char('q') => {}
            KeyCode::Char('y') => {
                let req = generate_manual_request(app.service.document(), RefinementMode::Conservative)
                    .unwrap_or_default();
                app.copy_to_clipboard(&req);
                app.active_modal = ActiveModal::ExportManualRequest { path_or_info };
            }
            _ => {
                app.active_modal = ActiveModal::ExportManualRequest { path_or_info };
            }
        },
        ActiveModal::ImportManualResponse { mut editor } => match key.code {
            KeyCode::Esc => {}
            KeyCode::Enter if key.modifiers == KeyModifiers::CONTROL => {
                let raw_text = editor.get_text();
                match parse_manual_response(&raw_text, app.service.document()) {
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
                        app.service.set_pending_refinements(pending);
                        app.active_modal = ActiveModal::ReviewRefinements {
                            selected_change_index: 0,
                        };
                        app.set_status("Import successful! Review changes.");
                    }
                    Err(e) => {
                        app.set_status(format!("Import parse error: {e}"));
                        app.active_modal = ActiveModal::ImportManualResponse { editor };
                    }
                }
            }
            KeyCode::Enter => {
                editor.insert_newline();
                app.active_modal = ActiveModal::ImportManualResponse { editor };
            }
            KeyCode::Backspace => {
                editor.backspace();
                app.active_modal = ActiveModal::ImportManualResponse { editor };
            }
            KeyCode::Char(c) => {
                editor.insert_char(c);
                app.active_modal = ActiveModal::ImportManualResponse { editor };
            }
            _ => {
                app.active_modal = ActiveModal::ImportManualResponse { editor };
            }
        },
        ActiveModal::ConfirmQuit => match key.code {
            KeyCode::Char('y') | KeyCode::Enter => {
                app.should_quit = true;
            }
            KeyCode::Char('s') => {
                if let Some(path) = app.service.current_file_path() {
                    let path = path.to_path_buf();
                    let _ = save_document(&path, app.service.document());
                }
                app.should_quit = true;
            }
            _ => {}
        },
    }
}
