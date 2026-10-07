pub mod commands;

use commands::*;
use prompt_application::ApplicationService;
use prompt_core::PromptDocument;
use prompt_persistence::{load_document as load_doc, AppConfig};
use std::path::PathBuf;

/// Launches the PromptForge desktop GUI using Tauri 2.
pub fn run_gui(initial_file: Option<PathBuf>) -> anyhow::Result<()> {
    let config = AppConfig::load_or_default();
    let initial_doc = if let Some(ref path) = initial_file {
        if path.exists() {
            load_doc(path).unwrap_or_else(|_| PromptDocument::default())
        } else {
            PromptDocument::default()
        }
    } else {
        PromptDocument::default()
    };

    let mut service = ApplicationService::new(initial_doc);
    if let Some(ref path) = initial_file {
        if path.exists() {
            service.mark_saved(path.clone());
        }
    }

    let state = AppState::new(service, config);

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            get_document_state,
            execute_command,
            new_document,
            load_document,
            save_document,
            render_xml_content,
            get_presets,
            get_section_types,
            save_section_type,
            delete_section_type,
            get_templates,
            get_config,
            save_config,
            generate_manual_request,
            apply_manual_response,
            execute_automated_refinement,
            accept_refinement,
            reject_refinement,
            accept_all_refinements,
            reject_all_refinements,
            accept_suggested_section,
            pick_open_file,
            pick_save_file,
            export_xml_to_file,
            compute_diff,
            set_tag_color,
            generate_skill_content,
            export_skill_to_file,
            pick_save_skill_file,
            parse_import_preview,
            import_prompt_content,
            pick_import_file,
            list_library_prompts,
            save_to_library,
            load_from_library,
            delete_from_library,
            get_document_variables,
            render_scenario_preview,
            compare_snapshot,
            fork_snapshot,
        ])
        .run(tauri::generate_context!())
        .map_err(|e| anyhow::anyhow!("Tauri runtime error: {e}"))
}
