use prompt_application::{ApplicationService, Command, PendingRefinements, ProposedSectionChange};
use prompt_core::{
    render_xml, ALL_PRESETS, ALL_TEMPLATES, PromptDocument, RenderOptions,
    RenderStage, SectionPreset, StarterTemplate,
};
use prompt_persistence::{export_text, load_document as load_doc, save_document as save_doc, AppConfig};
use prompt_refinement::{
    compute_line_diff, DiffLine, OpenAiCompatibleProvider, RefinementEngine, RefinementMode,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use tauri::State;
use uuid::Uuid;

pub struct AppState {
    pub service: Mutex<ApplicationService>,
    pub config: Mutex<AppConfig>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DocumentStateDto {
    pub document: PromptDocument,
    pub dirty: bool,
    pub current_file_path: Option<String>,
    pub can_undo: bool,
    pub can_redo: bool,
    pub pending_refinements: Option<PendingRefinements>,
}

impl AppState {
    pub fn new(service: ApplicationService, config: AppConfig) -> Self {
        Self {
            service: Mutex::new(service),
            config: Mutex::new(config),
        }
    }

    pub fn to_dto(&self) -> DocumentStateDto {
        let service = self.service.lock().unwrap();
        let state = service.state();
        DocumentStateDto {
            document: state.document.clone(),
            dirty: state.dirty,
            current_file_path: state.current_file_path.as_ref().map(|p| p.to_string_lossy().to_string()),
            can_undo: service.can_undo(),
            can_redo: service.can_redo(),
            pending_refinements: state.pending_refinements.clone(),
        }
    }
}

#[tauri::command]
pub fn get_document_state(state: State<AppState>) -> DocumentStateDto {
    state.to_dto()
}

#[tauri::command]
pub fn execute_command(command: Command, state: State<AppState>) -> Result<DocumentStateDto, String> {
    let mut service = state.service.lock().unwrap();
    service.execute(command).map_err(|e| e.to_string())?;
    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub fn new_document(
    template_id: Option<String>,
    title: Option<String>,
    state: State<AppState>,
) -> Result<DocumentStateDto, String> {
    let doc = if let Some(tmpl_id) = template_id {
        let tmpl = StarterTemplate::find(&tmpl_id)
            .ok_or_else(|| format!("Unknown template '{tmpl_id}'"))?;
        tmpl.instantiate(title.as_deref())
    } else {
        PromptDocument::new(title.as_deref().unwrap_or("Untitled Prompt"), "")
    };

    let mut service = state.service.lock().unwrap();
    service.load_document(doc, None);
    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub fn load_document(path: String, state: State<AppState>) -> Result<DocumentStateDto, String> {
    let path_buf = PathBuf::from(&path);
    let doc = load_doc(&path_buf).map_err(|e| format!("Failed to load document: {e}"))?;

    let mut service = state.service.lock().unwrap();
    service.load_document(doc, Some(path_buf.clone()));
    drop(service);

    // Track in recent files
    let mut config = state.config.lock().unwrap();
    config.add_recent_file(path_buf);
    let _ = config.save();
    drop(config);

    Ok(state.to_dto())
}

#[tauri::command]
pub fn save_document(path: Option<String>, state: State<AppState>) -> Result<DocumentStateDto, String> {
    let mut service = state.service.lock().unwrap();
    let save_path = match path {
        Some(p) => PathBuf::from(p),
        None => service
            .current_file_path()
            .map(|p| p.to_path_buf())
            .ok_or_else(|| "No file path specified for saving".to_string())?,
    };

    save_doc(&save_path, service.document()).map_err(|e| format!("Save error: {e}"))?;
    service.mark_saved(save_path.clone());
    drop(service);

    let mut config = state.config.lock().unwrap();
    config.add_recent_file(save_path);
    let _ = config.save();
    drop(config);

    Ok(state.to_dto())
}

#[tauri::command]
pub fn render_xml_content(stage: String, clean: bool, state: State<AppState>) -> Result<String, String> {
    let service = state.service.lock().unwrap();
    let render_stage = match stage.to_lowercase().as_str() {
        "final" => RenderStage::Final,
        _ => RenderStage::Draft,
    };
    let options = RenderOptions {
        stage: render_stage,
        include_ids: !clean,
        pretty: true,
    };
    render_xml(service.document(), options).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn get_presets() -> &'static [SectionPreset] {
    ALL_PRESETS
}

#[tauri::command]
pub fn get_section_types(state: State<AppState>) -> Vec<prompt_core::SectionType> {
    state.config.lock().unwrap().all_section_types()
}

#[tauri::command]
pub fn save_section_type(
    section_type: prompt_core::SectionType,
    state: State<AppState>,
) -> Result<Vec<prompt_core::SectionType>, String> {
    let mut config = state.config.lock().unwrap();
    config.save_custom_type(section_type);
    config.save().map_err(|e| e.to_string())?;
    Ok(config.all_section_types())
}

#[tauri::command]
pub fn delete_section_type(
    id: String,
    state: State<AppState>,
) -> Result<Vec<prompt_core::SectionType>, String> {
    let mut config = state.config.lock().unwrap();
    config.delete_custom_type(&id);
    config.save().map_err(|e| e.to_string())?;
    Ok(config.all_section_types())
}

#[tauri::command]
pub fn get_templates() -> &'static [StarterTemplate] {
    ALL_TEMPLATES
}

#[tauri::command]
pub fn get_config(state: State<AppState>) -> AppConfig {
    state.config.lock().unwrap().clone()
}

#[tauri::command]
pub fn save_config(config: AppConfig, state: State<AppState>) -> Result<(), String> {
    let mut current = state.config.lock().unwrap();
    *current = config;
    current.save().map_err(|e| e.to_string())
}

#[tauri::command]
pub fn generate_manual_request(mode: String, state: State<AppState>) -> Result<String, String> {
    let service = state.service.lock().unwrap();
    let refinement_mode = match mode.to_lowercase().as_str() {
        "expand" => RefinementMode::Expand,
        "critique" => RefinementMode::Critique,
        _ => RefinementMode::Conservative,
    };
    prompt_refinement::generate_manual_request(service.document(), refinement_mode)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub fn apply_manual_response(response_text: String, state: State<AppState>) -> Result<DocumentStateDto, String> {
    let mut service = state.service.lock().unwrap();
    let changeset = prompt_refinement::parse_manual_response(&response_text, service.document())
        .map_err(|e| format!("Failed to parse refinement response: {e}"))?;

    changeset
        .validate_against_document(service.document())
        .map_err(|e| format!("Validation error: {e}"))?;

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
            .map(|s| prompt_application::SuggestedSection {
                tag: s.tag,
                content: s.content,
                reason: s.reason,
            })
            .collect(),
        open_questions: changeset.open_questions,
        warnings: changeset.warnings,
        critique: changeset.critique,
    };

    service.set_pending_refinements(pending);
    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub async fn execute_automated_refinement(
    mode: String,
    state: State<'_, AppState>,
) -> Result<DocumentStateDto, String> {
    let (doc, cfg, ref_mode) = {
        let service = state.service.lock().unwrap();
        let config = state.config.lock().unwrap();
        let m = match mode.to_lowercase().as_str() {
            "expand" => RefinementMode::Expand,
            "critique" => RefinementMode::Critique,
            _ => RefinementMode::Conservative,
        };
        (service.document().clone(), config.openai_compatible.clone(), m)
    };

    let provider = OpenAiCompatibleProvider::new(
        cfg.base_url,
        cfg.model,
        cfg.api_key_env_var,
        cfg.timeout_seconds,
    );
    let engine = RefinementEngine::new();
    let changeset = engine
        .execute(&provider, &doc, ref_mode)
        .await
        .map_err(|e| format!("Refinement error: {e}"))?;

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
            .map(|s| prompt_application::SuggestedSection {
                tag: s.tag,
                content: s.content,
                reason: s.reason,
            })
            .collect(),
        open_questions: changeset.open_questions,
        warnings: changeset.warnings,
        critique: changeset.critique,
    };

    let mut service = state.service.lock().unwrap();
    service.set_pending_refinements(pending);
    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub fn accept_refinement(section_id: String, state: State<AppState>) -> Result<DocumentStateDto, String> {
    let id = Uuid::parse_str(&section_id).map_err(|e| e.to_string())?;
    let mut service = state.service.lock().unwrap();
    service.execute(Command::ApplyRefinement { id }).map_err(|e| e.to_string())?;
    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub fn reject_refinement(section_id: String, state: State<AppState>) -> Result<DocumentStateDto, String> {
    let id = Uuid::parse_str(&section_id).map_err(|e| e.to_string())?;
    let mut service = state.service.lock().unwrap();
    service.execute(Command::RejectRefinement { id }).map_err(|e| e.to_string())?;
    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub fn accept_all_refinements(state: State<AppState>) -> Result<DocumentStateDto, String> {
    let mut service = state.service.lock().unwrap();
    service.execute(Command::ApplyAllRefinements).map_err(|e| e.to_string())?;
    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub fn reject_all_refinements(state: State<AppState>) -> Result<DocumentStateDto, String> {
    let mut service = state.service.lock().unwrap();
    service.execute(Command::RejectAllRefinements).map_err(|e| e.to_string())?;
    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub fn accept_suggested_section(index: usize, state: State<AppState>) -> Result<DocumentStateDto, String> {
    let mut service = state.service.lock().unwrap();
    service.accept_suggested_section(index).map_err(|e| e.to_string())?;
    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub fn pick_open_file() -> Option<String> {
    rfd::FileDialog::new()
        .add_filter("Prompt Documents", &["prompt.json", "json"])
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn pick_save_file(default_name: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new().add_filter("Prompt Document", &["prompt.json", "json"]);
    if let Some(name) = default_name {
        dialog = dialog.set_file_name(name);
    }
    dialog.save_file().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn export_xml_to_file(path: String, stage: String, clean: bool, state: State<AppState>) -> Result<(), String> {
    let service = state.service.lock().unwrap();
    let render_stage = match stage.to_lowercase().as_str() {
        "final" => RenderStage::Final,
        _ => RenderStage::Draft,
    };
    let options = RenderOptions {
        stage: render_stage,
        include_ids: !clean,
        pretty: true,
    };
    let xml = render_xml(service.document(), options).map_err(|e| e.to_string())?;
    export_text(Path::new(&path), &xml).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn compute_diff(original: String, refined: String) -> Vec<DiffLine> {
    compute_line_diff(&original, &refined)
}
