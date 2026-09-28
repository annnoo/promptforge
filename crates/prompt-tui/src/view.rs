use crate::app::{ActiveModal, ActivePanel, AppState};
use prompt_core::{template::ALL_PRESETS, RenderOptions, RenderStage};
use prompt_refinement::{compute_line_diff, DiffTag};
use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Clear, List, ListItem, Paragraph, Wrap},
    Frame,
};

/// Renders the complete TUI frame.
pub fn render_ui(frame: &mut Frame, app: &mut AppState) {
    let size = frame.area();

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(12),       // Main (Sections + Detail/Editor)
            Constraint::Length(10),     // XML Preview
            Constraint::Length(1),      // Footer
        ])
        .split(size);

    render_header(frame, app, chunks[0]);
    render_main_area(frame, app, chunks[1]);
    render_xml_preview(frame, app, chunks[2]);
    render_footer(frame, app, chunks[3]);

    // Render active modal on top if any
    render_modal(frame, app, size);
}

fn render_header(frame: &mut Frame, app: &AppState, area: Rect) {
    let dirty_indicator = if app.service.is_dirty() { " [● UNSAVED]" } else { "" };
    let file_info = app
        .service
        .current_file_path()
        .map(|p| p.display().to_string())
        .unwrap_or_else(|| "Untitled".to_string());

    let title_line = Line::from(vec![
        Span::styled(" PROMPTFORGE ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::styled(
            format!("  Doc: {} ", app.service.document().title),
            Style::default().fg(Color::White).bold(),
        ),
        Span::styled(dirty_indicator, Style::default().fg(Color::Yellow).bold()),
        Span::styled(format!("  ({file_info})"), Style::default().fg(Color::DarkGray)),
    ]);

    let stage_name = match app.preview_stage {
        RenderStage::Draft => "Draft",
        RenderStage::Final => "Final",
    };
    let right_status = format!("Stage: {stage_name} | XML v1 ");

    let header_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

    let header_para = Paragraph::new(title_line)
        .block(header_block)
        .right_aligned();

    let right_span = Span::styled(right_status, Style::default().fg(Color::Gray));
    frame.render_widget(header_para, area);

    // Overlay right aligned text
    let right_rect = Rect {
        x: area.x + area.width.saturating_sub(30),
        y: area.y + 1,
        width: 28,
        height: 1,
    };
    frame.render_widget(Paragraph::new(Line::from(right_span)).right_aligned(), right_rect);
}

fn render_main_area(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(35), // Sections List
            Constraint::Percentage(65), // Detail / Editor
        ])
        .split(area);

    render_sections_list(frame, app, main_chunks[0]);
    render_detail_editor(frame, app, main_chunks[1]);
}

fn render_sections_list(frame: &mut Frame, app: &mut AppState, area: Rect) {
    let is_focused = app.active_panel == ActivePanel::Sections && !app.editing_brief;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let sections = &app.service.document().sections;
    let items: Vec<ListItem> = sections
        .iter()
        .enumerate()
        .map(|(idx, sec)| {
            let is_selected = idx == app.selected_section_index;
            let marker = if is_selected { "> " } else { "  " };

            let mut flags = String::new();
            if sec.locked {
                flags.push_str(" [LOCK]");
            }
            if !sec.enabled {
                flags.push_str(" [OFF]");
            }
            if sec.has_refinement() {
                flags.push_str(" [REFINED]");
            }

            let style = if is_selected {
                Style::default().fg(Color::Yellow).bold()
            } else if !sec.enabled {
                Style::default().fg(Color::DarkGray)
            } else {
                Style::default().fg(Color::White)
            };

            let line = Line::from(vec![
                Span::styled(marker, Style::default().fg(Color::Cyan).bold()),
                Span::styled(&sec.tag, style),
                Span::styled(flags, Style::default().fg(Color::Magenta)),
            ]);

            ListItem::new(line)
        })
        .collect();

    let list_block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(" SECTIONS ", Style::default().fg(border_color).bold()));

    let list = List::new(items).block(list_block);
    frame.render_widget(list, area);
}

fn render_detail_editor(frame: &mut Frame, app: &AppState, area: Rect) {
    let is_focused = app.active_panel == ActivePanel::Editor || app.editing_brief;
    let border_color = if app.editing_brief {
        Color::Green
    } else if is_focused {
        Color::Cyan
    } else {
        Color::DarkGray
    };

    let selected_section = app
        .service
        .document()
        .sections
        .get(app.selected_section_index);

    if let Some(sec) = selected_section {
        let title = if app.editing_brief {
            format!(" EDITING BRIEF: <{}> (Esc to Save) ", sec.tag)
        } else {
            format!(" SECTION: <{}> ", sec.tag)
        };

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .border_style(Style::default().fg(border_color))
            .title(Span::styled(title, Style::default().fg(border_color).bold()));

        if app.editing_brief {
            // Render text from live editor
            let lines: Vec<Line> = app
                .editor
                .lines
                .iter()
                .enumerate()
                .map(|(r, line)| {
                    if r == app.editor.cursor_row {
                        // Highlight cursor row or position
                        Line::from(vec![
                            Span::raw(line),
                            Span::styled(" ", Style::default().bg(Color::White)),
                        ])
                    } else {
                        Line::from(line.as_str())
                    }
                })
                .collect();

            let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
            frame.render_widget(para, area);
        } else {
            let mut text_lines = Vec::new();
            text_lines.push(Line::from(vec![
                Span::styled("Tag: ", Style::default().fg(Color::DarkGray)),
                Span::styled(&sec.tag, Style::default().fg(Color::Cyan).bold()),
                Span::styled(" | ID: ", Style::default().fg(Color::DarkGray)),
                Span::styled(sec.id.to_string(), Style::default().fg(Color::DarkGray)),
            ]));
            text_lines.push(Line::from(""));

            text_lines.push(Line::from(Span::styled(
                "--- User Brief (Original) ---",
                Style::default().fg(Color::Yellow),
            )));
            for line in sec.brief.lines() {
                text_lines.push(Line::from(line));
            }

            if let Some(ref refined) = sec.refined {
                text_lines.push(Line::from(""));
                text_lines.push(Line::from(Span::styled(
                    "--- Accepted Refinement ---",
                    Style::default().fg(Color::Green),
                )));
                for line in refined.lines() {
                    text_lines.push(Line::from(line));
                }
            }

            let para = Paragraph::new(text_lines)
                .block(block)
                .wrap(Wrap { trim: false });
            frame.render_widget(para, area);
        }
    } else {
        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" SECTION DETAIL ");
        let empty_msg = Paragraph::new("No sections present. Press 'a' to add a section.")
            .block(block)
            .alignment(Alignment::Center);
        frame.render_widget(empty_msg, area);
    }
}

fn render_xml_preview(frame: &mut Frame, app: &AppState, area: Rect) {
    let is_focused = app.active_panel == ActivePanel::XmlPreview;
    let border_color = if is_focused { Color::Cyan } else { Color::DarkGray };

    let options = RenderOptions {
        stage: app.preview_stage,
        include_ids: app.preview_include_ids,
        pretty: true,
    };

    let xml_content = app
        .service
        .render_xml(options)
        .unwrap_or_else(|e| format!("<!-- XML Error: {e} -->"));

    let id_status = if app.preview_include_ids { "With IDs" } else { "Clean (No IDs)" };
    let title = format!(
        " XML PREVIEW ({:?}, {}) [p to focus, v to toggle stage, i to toggle IDs, y to copy] ",
        app.preview_stage, id_status
    );

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
        .title(Span::styled(title, Style::default().fg(border_color).bold()));

    let lines: Vec<Line> = xml_content
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let style = if trimmed.starts_with("<!--") {
                Style::default().fg(Color::DarkGray)
            } else if trimmed.starts_with('<') && trimmed.ends_with('>') {
                Style::default().fg(Color::Cyan)
            } else {
                Style::default().fg(Color::White)
            };
            Line::styled(line, style)
        })
        .collect();

    let para = Paragraph::new(lines).block(block);
    frame.render_widget(para, area);
}

fn render_footer(frame: &mut Frame, app: &AppState, area: Rect) {
    let status_or_help = if let Some(ref msg) = app.status_message {
        Line::from(vec![
            Span::styled(" >> ", Style::default().fg(Color::Yellow).bold()),
            Span::styled(msg, Style::default().fg(Color::White).bold()),
        ])
    } else {
        Line::from(vec![
            Span::styled("j/k", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" nav | "),
            Span::styled("e", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" edit | "),
            Span::styled("a", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" add | "),
            Span::styled("J/K", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" move | "),
            Span::styled("l", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" lock | "),
            Span::styled("x", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" toggle | "),
            Span::styled("r", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" refine | "),
            Span::styled("u", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" undo | "),
            Span::styled("s", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" save | "),
            Span::styled("?", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" help | "),
            Span::styled("q", Style::default().fg(Color::Cyan).bold()),
            Span::raw(" quit"),
        ])
    };

    let para = Paragraph::new(status_or_help).alignment(Alignment::Left);
    frame.render_widget(para, area);
}

fn render_modal(frame: &mut Frame, app: &AppState, screen_size: Rect) {
    match &app.active_modal {
        ActiveModal::None => {}
        ActiveModal::Help => render_help_modal(frame, screen_size),
        ActiveModal::AddSection {
            selected_index,
            is_custom,
            custom_tag,
        } => render_add_section_modal(frame, screen_size, *selected_index, *is_custom, custom_tag),
        ActiveModal::RenameSection { new_tag } => {
            render_rename_modal(frame, screen_size, new_tag);
        }
        ActiveModal::ConfirmDelete => {
            render_confirm_delete_modal(frame, app, screen_size);
        }
        ActiveModal::SaveFile { path_input } => {
            render_input_modal(frame, screen_size, "SAVE DOCUMENT", "Enter file path (.prompt.json):", path_input);
        }
        ActiveModal::OpenFile { path_input } => {
            render_input_modal(frame, screen_size, "OPEN DOCUMENT", "Enter file path to load:", path_input);
        }
        ActiveModal::RefinePrompt {
            selected_mode,
            selected_provider,
        } => {
            render_refine_config_modal(frame, screen_size, *selected_mode, *selected_provider);
        }
        ActiveModal::ReviewRefinements {
            selected_change_index,
        } => {
            render_review_refinements_modal(frame, app, screen_size, *selected_change_index);
        }
        ActiveModal::ExportManualRequest { path_or_info } => {
            render_info_modal(frame, screen_size, "MANUAL REFINEMENT REQUEST", path_or_info);
        }
        ActiveModal::ImportManualResponse { editor } => {
            render_import_response_modal(frame, screen_size, editor);
        }
        ActiveModal::ConfirmQuit => {
            render_confirm_quit_modal(frame, screen_size);
        }
    }
}

fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
    let popup_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(r);

    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .split(popup_layout[1])[1]
}

fn render_help_modal(frame: &mut Frame, screen: Rect) {
    let area = centered_rect(60, 70, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(" HELP & KEYBINDINGS ")
        .border_style(Style::default().fg(Color::Cyan));

    let text = vec![
        Line::from(Span::styled("NAVIGATION & EDITING", Style::default().fg(Color::Yellow).bold())),
        Line::from("  j / Down        : Select next section"),
        Line::from("  k / Up          : Select previous section"),
        Line::from("  J / Shift+Down  : Move section down"),
        Line::from("  K / Shift+Up    : Move section up"),
        Line::from("  e               : Edit section brief (Esc to exit & save)"),
        Line::from("  a               : Add section preset or custom XML tag"),
        Line::from("  d               : Delete selected section (with confirmation)"),
        Line::from("  D or c          : Duplicate selected section"),
        Line::from("  n or R          : Rename XML tag of selected section"),
        Line::from("  l               : Toggle lock on selected section"),
        Line::from("  x or Space      : Toggle enabled state"),
        Line::from(""),
        Line::from(Span::styled("XML & EXPORT", Style::default().fg(Color::Yellow).bold())),
        Line::from("  p               : Switch focus to XML preview"),
        Line::from("  v               : Toggle Draft vs Final XML preview"),
        Line::from("  i               : Toggle internal section IDs in XML"),
        Line::from("  y               : Copy XML to system clipboard"),
        Line::from(""),
        Line::from(Span::styled("REFINEMENT WORKFLOW", Style::default().fg(Color::Yellow).bold())),
        Line::from("  r               : Refine prompt (Manual or OpenAI-compatible)"),
        Line::from("  m               : Generate & export manual refinement request"),
        Line::from("  I (Shift+i)     : Import manual refinement response JSON/XML"),
        Line::from(""),
        Line::from(Span::styled("FILE & HISTORY", Style::default().fg(Color::Yellow).bold())),
        Line::from("  s               : Save document"),
        Line::from("  o               : Open document"),
        Line::from("  u               : Undo"),
        Line::from("  Ctrl+r          : Redo"),
        Line::from("  q / Esc         : Close modal or quit"),
    ];

    let para = Paragraph::new(text).block(block).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_add_section_modal(
    frame: &mut Frame,
    screen: Rect,
    selected_index: usize,
    is_custom: bool,
    custom_tag: &str,
) {
    let area = centered_rect(50, 60, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(" ADD SECTION (j/k select, Enter add, Tab toggle custom, Esc cancel) ")
        .border_style(Style::default().fg(Color::Green));

    let mut items = Vec::new();
    for (idx, preset) in ALL_PRESETS.iter().enumerate() {
        let is_sel = !is_custom && idx == selected_index;
        let prefix = if is_sel { "> " } else { "  " };
        let style = if is_sel {
            Style::default().fg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White)
        };
        items.push(ListItem::new(Line::from(vec![
            Span::styled(prefix, Style::default().fg(Color::Green).bold()),
            Span::styled(preset.name, style),
            Span::styled(format!(" <{}>", preset.tag), Style::default().fg(Color::Cyan)),
        ])));
    }

    // Custom option
    let custom_sel = is_custom;
    let custom_prefix = if custom_sel { "> " } else { "  " };
    let custom_style = if custom_sel {
        Style::default().fg(Color::Yellow).bold()
    } else {
        Style::default().fg(Color::White)
    };
    items.push(ListItem::new(Line::from(vec![
        Span::styled(custom_prefix, Style::default().fg(Color::Green).bold()),
        Span::styled("Custom XML Tag: ", custom_style),
        Span::styled(custom_tag, Style::default().fg(Color::Yellow).underlined()),
    ])));

    let list = List::new(items).block(block);
    frame.render_widget(list, area);
}

fn render_rename_modal(frame: &mut Frame, screen: Rect, new_tag: &str) {
    let area = centered_rect(40, 20, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(" RENAME SECTION TAG (Enter to save, Esc to cancel) ")
        .border_style(Style::default().fg(Color::Cyan));

    let content = vec![
        Line::from("New XML Tag:"),
        Line::from(Span::styled(new_tag, Style::default().fg(Color::Yellow).bold())),
    ];

    let para = Paragraph::new(content).block(block).alignment(Alignment::Center);
    frame.render_widget(para, area);
}

fn render_confirm_delete_modal(frame: &mut Frame, app: &AppState, screen: Rect) {
    let area = centered_rect(40, 20, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(" CONFIRM DELETE ")
        .border_style(Style::default().fg(Color::Red));

    let tag = app
        .service
        .document()
        .sections
        .get(app.selected_section_index)
        .map(|s| s.tag.as_str())
        .unwrap_or("section");

    let content = vec![
        Line::from(format!("Are you sure you want to delete <{tag}>?")),
        Line::from(""),
        Line::from(vec![
            Span::styled(" y / Enter ", Style::default().fg(Color::Black).bg(Color::Red).bold()),
            Span::raw(" Yes, delete  |  "),
            Span::styled(" n / Esc ", Style::default().fg(Color::Black).bg(Color::Gray).bold()),
            Span::raw(" Cancel"),
        ]),
    ];

    let para = Paragraph::new(content).block(block).alignment(Alignment::Center);
    frame.render_widget(para, area);
}

fn render_input_modal(frame: &mut Frame, screen: Rect, title: &str, prompt: &str, input: &str) {
    let area = centered_rect(50, 25, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(format!(" {title} "))
        .border_style(Style::default().fg(Color::Cyan));

    let content = vec![
        Line::from(prompt),
        Line::from(""),
        Line::from(Span::styled(format!("> {input}"), Style::default().fg(Color::Yellow).bold())),
        Line::from(""),
        Line::from(Span::styled("(Enter to confirm, Esc to cancel)", Style::default().fg(Color::DarkGray))),
    ];

    let para = Paragraph::new(content).block(block).alignment(Alignment::Center);
    frame.render_widget(para, area);
}

fn render_refine_config_modal(
    frame: &mut Frame,
    screen: Rect,
    mode: prompt_refinement::RefinementMode,
    provider: prompt_persistence::ProviderType,
) {
    let area = centered_rect(60, 45, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(" REFINE PROMPT CONFIGURATION ")
        .border_style(Style::default().fg(Color::Magenta));

    let mode_str = mode.name();
    let mode_desc = mode.description();
    let provider_str = match provider {
        prompt_persistence::ProviderType::Manual => "Manual (No API key needed - export/import)",
        prompt_persistence::ProviderType::OpenAiCompatible => "OpenAI-Compatible HTTP API",
    };

    let content = vec![
        Line::from(Span::styled("Select Refinement Mode (Press 1/2/3):", Style::default().fg(Color::Cyan).bold())),
        Line::from("  1. Conservative: Improve wording, preserve all constraints"),
        Line::from("  2. Expand: Elaborate instructions and suggest missing sections"),
        Line::from("  3. Critique: Analyze ambiguities without editing document"),
        Line::from(""),
        Line::from(vec![
            Span::raw("Active Mode: "),
            Span::styled(mode_str, Style::default().fg(Color::Yellow).bold()),
        ]),
        Line::from(Span::styled(format!("  -> {mode_desc}"), Style::default().fg(Color::Gray))),
        Line::from(""),
        Line::from(Span::styled("Provider (Press Tab to toggle):", Style::default().fg(Color::Cyan).bold())),
        Line::from(vec![
            Span::raw("  "),
            Span::styled(provider_str, Style::default().fg(Color::Green).bold()),
        ]),
        Line::from(""),
        Line::from(Span::styled("Press Enter to Run | Esc to Cancel", Style::default().fg(Color::White).bold())),
    ];

    let para = Paragraph::new(content).block(block).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_review_refinements_modal(
    frame: &mut Frame,
    app: &AppState,
    screen: Rect,
    selected_index: usize,
) {
    let area = centered_rect(80, 80, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(" REVIEW PENDING REFINEMENTS (y: accept, n: reject, A: accept all, X: reject all, Esc: close) ")
        .border_style(Style::default().fg(Color::Yellow));

    let pending = match &app.service.state().pending_refinements {
        Some(p) if !p.is_empty() => p,
        _ => {
            let empty_p = Paragraph::new("No pending refinements to review.")
                .block(block)
                .alignment(Alignment::Center);
            frame.render_widget(empty_p, area);
            return;
        }
    };

    let layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),      // Change selector / tabs
            Constraint::Min(10),       // Diff view
            Constraint::Length(4),      // Actions bar
        ])
        .split(block.inner(area));

    frame.render_widget(block, area);

    // Render selector line
    let mut change_spans = Vec::new();
    for (i, change) in pending.changes.iter().enumerate() {
        let tag = app
            .service
            .document()
            .section(change.section_id)
            .map(|s| s.tag.as_str())
            .unwrap_or("section");

        let is_sel = i == selected_index;
        let style = if is_sel {
            Style::default().fg(Color::Black).bg(Color::Yellow).bold()
        } else {
            Style::default().fg(Color::White).bg(Color::DarkGray)
        };

        change_spans.push(Span::styled(format!(" [{}: <{}>] ", i + 1, tag), style));
        change_spans.push(Span::raw(" "));
    }

    let selector_para = Paragraph::new(vec![
        Line::from(change_spans),
        Line::from(format!("Pending Changes: {} | Use Left/Right or 1..9 to switch", pending.changes.len())),
    ]);
    frame.render_widget(selector_para, layout[0]);

    // Render diff for selected change
    if let Some(change) = pending.changes.get(selected_index) {
        let section = app.service.document().section(change.section_id);
        let orig = section.map(|s| s.brief.as_str()).unwrap_or("");
        let diff_lines = compute_line_diff(orig, &change.proposed_text);

        let mut diff_widget_lines = Vec::new();
        diff_widget_lines.push(Line::from(vec![
            Span::styled("Reason: ", Style::default().fg(Color::Cyan).bold()),
            Span::styled(&change.reason, Style::default().fg(Color::White)),
        ]));
        diff_widget_lines.push(Line::from(""));

        for d in diff_lines {
            let (prefix, style) = match d.tag {
                DiffTag::Equal => ("  ", Style::default().fg(Color::Gray)),
                DiffTag::Delete => ("- ", Style::default().fg(Color::Red)),
                DiffTag::Insert => ("+ ", Style::default().fg(Color::Green).bold()),
            };
            diff_widget_lines.push(Line::styled(format!("{prefix}{}", d.text), style));
        }

        let diff_block = Block::default()
            .borders(Borders::ALL)
            .border_type(BorderType::Rounded)
            .title(" DIFF (Original vs Proposed) ");

        let diff_para = Paragraph::new(diff_widget_lines).block(diff_block);
        frame.render_widget(diff_para, layout[1]);
    }

    // Render action keys
    let actions = Line::from(vec![
        Span::styled(" [y] Accept Section ", Style::default().fg(Color::Black).bg(Color::Green).bold()),
        Span::raw("  "),
        Span::styled(" [n] Reject Section ", Style::default().fg(Color::Black).bg(Color::Red).bold()),
        Span::raw("  "),
        Span::styled(" [A] Accept ALL ", Style::default().fg(Color::Black).bg(Color::Cyan).bold()),
        Span::raw("  "),
        Span::styled(" [X] Reject ALL ", Style::default().fg(Color::Black).bg(Color::Magenta).bold()),
    ]);
    frame.render_widget(Paragraph::new(actions).alignment(Alignment::Center), layout[2]);
}

fn render_info_modal(frame: &mut Frame, screen: Rect, title: &str, info: &str) {
    let area = centered_rect(70, 70, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(format!(" {title} (Esc to close, y to copy) "))
        .border_style(Style::default().fg(Color::Cyan));

    let para = Paragraph::new(info).block(block).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_import_response_modal(frame: &mut Frame, screen: Rect, editor: &crate::editor::TextEditor) {
    let area = centered_rect(75, 75, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(" IMPORT REFINEMENT RESPONSE (Paste JSON or XML, Ctrl+S / Enter to Import, Esc Cancel) ")
        .border_style(Style::default().fg(Color::Green));

    let lines: Vec<Line> = editor
        .lines
        .iter()
        .map(|l| Line::from(l.as_str()))
        .collect();

    let para = Paragraph::new(lines).block(block).wrap(Wrap { trim: false });
    frame.render_widget(para, area);
}

fn render_confirm_quit_modal(frame: &mut Frame, screen: Rect) {
    let area = centered_rect(40, 20, screen);
    frame.render_widget(Clear, area);

    let block = Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Double)
        .title(" UNSAVED CHANGES ")
        .border_style(Style::default().fg(Color::Yellow));

    let content = vec![
        Line::from("You have unsaved changes. Exit anyway?"),
        Line::from(""),
        Line::from(vec![
            Span::styled(" y / Enter ", Style::default().fg(Color::Black).bg(Color::Red).bold()),
            Span::raw(" Yes, exit  |  "),
            Span::styled(" s ", Style::default().fg(Color::Black).bg(Color::Green).bold()),
            Span::raw(" Save first  |  "),
            Span::styled(" Esc ", Style::default().fg(Color::Black).bg(Color::Gray).bold()),
            Span::raw(" Cancel"),
        ]),
    ];

    let para = Paragraph::new(content).block(block).alignment(Alignment::Center);
    frame.render_widget(para, area);
}
