use prompt_application::{ApplicationService, Command, PendingRefinements, ProposedSectionChange};
use prompt_core::{
    calculate_prompt_telemetry, compare_document_with_snapshot, extract_document_variables,
    generate_skill_markdown, import_prompt_from_text, render_interpolated_xml, render_xml,
    ALL_PRESETS, ALL_TEMPLATES, ImportFormat, PromptDocument, PromptSection, RenderOptions,
    RenderStage, SectionPreset, SkillExportOptions, SnapshotComparison, StarterTemplate,
};
use prompt_persistence::{
    delete_from_library as delete_doc_from_lib, export_skill, export_text, list_saved_prompts,
    load_document as load_doc, load_from_library as load_doc_from_lib, save_document as save_doc,
    save_to_library as save_doc_to_lib, AppConfig, SavedPromptSummary,
};
use prompt_refinement::{
    compute_line_diff, DiffLine, OpenAiCompatibleProvider, RefinementEngine, RefinementMode,
};
use serde::Serialize;
use std::collections::HashMap;
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
pub async fn test_ai_connection(
    config: prompt_persistence::OpenAiCompatibleConfig,
) -> Result<String, String> {
    let provider = OpenAiCompatibleProvider::with_api_key(
        config.base_url,
        config.model,
        config.api_key_env_var,
        config.api_key,
        config.timeout_seconds,
    );
    provider
        .test_connection()
        .await
        .map_err(|e| e.to_string())
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

    let provider = OpenAiCompatibleProvider::with_api_key(
        cfg.base_url,
        cfg.model,
        cfg.api_key_env_var,
        cfg.api_key,
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

#[derive(Debug, Clone, Serialize)]
pub struct ImportPreviewDto {
    pub format: ImportFormat,
    pub title: String,
    pub description: String,
    pub section_count: usize,
    pub sections: Vec<PromptSection>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ImportFileContent {
    pub path: String,
    pub file_name: String,
    pub content: String,
}

#[tauri::command]
pub fn set_tag_color(tag: String, color: String, state: State<AppState>) -> Result<AppConfig, String> {
    let mut config = state.config.lock().unwrap();
    config.set_tag_color(tag, color);
    config.save().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub fn generate_skill_content(
    stage: String,
    name: Option<String>,
    description: Option<String>,
    state: State<AppState>,
) -> Result<String, String> {
    let service = state.service.lock().unwrap();
    let render_stage = match stage.to_lowercase().as_str() {
        "draft" => RenderStage::Draft,
        _ => RenderStage::Final,
    };
    let options = SkillExportOptions {
        stage: render_stage,
        custom_name: name,
        custom_description: description,
    };
    generate_skill_markdown(service.document(), options).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn export_skill_to_file(
    path: String,
    stage: String,
    name: Option<String>,
    description: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let service = state.service.lock().unwrap();
    let render_stage = match stage.to_lowercase().as_str() {
        "draft" => RenderStage::Draft,
        _ => RenderStage::Final,
    };
    let options = SkillExportOptions {
        stage: render_stage,
        custom_name: name,
        custom_description: description,
    };
    export_skill(Path::new(&path), service.document(), options).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn pick_save_skill_file(default_name: Option<String>) -> Option<String> {
    let dialog = rfd::FileDialog::new()
        .add_filter("Agent Skill (Markdown)", &["md", "markdown"])
        .set_file_name(default_name.unwrap_or_else(|| "SKILL.md".to_string()));
    dialog.save_file().map(|p| p.to_string_lossy().to_string())
}

#[tauri::command]
pub fn parse_import_preview(content: String) -> Result<ImportPreviewDto, String> {
    let imported = import_prompt_from_text(&content).map_err(|e| e.to_string())?;
    Ok(ImportPreviewDto {
        format: imported.format,
        title: imported.title,
        description: imported.description,
        section_count: imported.sections.len(),
        sections: imported.sections,
    })
}

#[tauri::command]
pub fn import_prompt_content(
    content: String,
    mode: String,
    state: State<AppState>,
) -> Result<DocumentStateDto, String> {
    let imported = import_prompt_from_text(&content).map_err(|e| e.to_string())?;
    let mut service = state.service.lock().unwrap();

    if mode.to_lowercase() == "append" {
        service
            .append_sections(imported.sections)
            .map_err(|e| e.to_string())?;
    } else {
        let doc = imported.to_document().map_err(|e| e.to_string())?;
        service.load_document(doc, None);
    }

    drop(service);
    Ok(state.to_dto())
}

#[tauri::command]
pub fn pick_import_file() -> Option<ImportFileContent> {
    let file = rfd::FileDialog::new()
        .add_filter(
            "Prompt Formats",
            &["json", "prompt.json", "xml", "md", "txt"],
        )
        .pick_file()?;

    let content = std::fs::read_to_string(&file).ok()?;
    let file_name = file
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("import_file")
        .to_string();

    Some(ImportFileContent {
        path: file.to_string_lossy().to_string(),
        file_name,
        content,
    })
}

#[tauri::command]
pub fn list_library_prompts(_state: State<AppState>) -> Result<Vec<SavedPromptSummary>, String> {
    let library_dir = AppConfig::library_dir()
        .ok_or_else(|| "Could not determine library directory".to_string())?;
    list_saved_prompts(&library_dir).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn save_to_library(state: State<AppState>) -> Result<SavedPromptSummary, String> {
    let library_dir = AppConfig::library_dir()
        .ok_or_else(|| "Could not determine library directory".to_string())?;
    let service = state.service.lock().unwrap();
    save_doc_to_lib(&library_dir, service.document()).map_err(|e| e.to_string())
}

#[tauri::command]
pub fn load_from_library(id: String, state: State<AppState>) -> Result<DocumentStateDto, String> {
    let library_dir = AppConfig::library_dir()
        .ok_or_else(|| "Could not determine library directory".to_string())?;
    let doc = load_doc_from_lib(&library_dir, &id).map_err(|e| e.to_string())?;

    let mut service = state.service.lock().unwrap();
    service.load_document(doc, None);
    drop(service);

    Ok(state.to_dto())
}

#[tauri::command]
pub fn delete_from_library(id: String, _state: State<AppState>) -> Result<(), String> {
    let library_dir = AppConfig::library_dir()
        .ok_or_else(|| "Could not determine library directory".to_string())?;
    delete_doc_from_lib(&library_dir, &id).map_err(|e| e.to_string())?;
    Ok(())
}

#[derive(Debug, Clone, Serialize)]
pub struct ScenarioPreviewDto {
    pub rendered_xml: String,
    pub char_count: usize,
    pub word_count: usize,
    pub estimated_tokens: usize,
    pub all_variables: Vec<String>,
    pub unresolved_variables: Vec<String>,
}

#[tauri::command]
pub fn get_document_variables(state: State<AppState>) -> Vec<String> {
    let service = state.service.lock().unwrap();
    extract_document_variables(service.document())
}

#[tauri::command]
pub fn render_scenario_preview(
    scenario_id: Option<String>,
    custom_values: Option<HashMap<String, String>>,
    stage: String,
    clean: bool,
    state: State<AppState>,
) -> Result<ScenarioPreviewDto, String> {
    let service = state.service.lock().unwrap();
    let doc = service.document();
    let all_variables = extract_document_variables(doc);

    let mut values = HashMap::new();
    if let Some(ref sc_id) = scenario_id {
        if let Some(scenario) = doc.scenarios.iter().find(|s| s.id == *sc_id) {
            for (k, v) in &scenario.variables {
                values.insert(k.clone(), v.clone());
            }
        }
    }
    if let Some(custom) = custom_values {
        for (k, v) in custom {
            values.insert(k, v);
        }
    }

    let render_stage = match stage.to_lowercase().as_str() {
        "final" => RenderStage::Final,
        _ => RenderStage::Draft,
    };
    let options = RenderOptions {
        stage: render_stage,
        include_ids: !clean,
        pretty: true,
    };

    let rendered_xml = render_interpolated_xml(doc, options, &values).map_err(|e| e.to_string())?;
    let (char_count, word_count, estimated_tokens) = calculate_prompt_telemetry(&rendered_xml);

    let unresolved_variables = all_variables
        .iter()
        .filter(|v| {
            !values.contains_key(*v) || values.get(*v).map(|s| s.trim().is_empty()).unwrap_or(true)
        })
        .cloned()
        .collect();

    Ok(ScenarioPreviewDto {
        rendered_xml,
        char_count,
        word_count,
        estimated_tokens,
        all_variables,
        unresolved_variables,
    })
}

#[tauri::command]
pub fn compare_snapshot(
    snapshot_id: String,
    state: State<AppState>,
) -> Result<SnapshotComparison, String> {
    let service = state.service.lock().unwrap();
    let doc = service.document();
    let snapshot = doc
        .snapshots
        .iter()
        .find(|s| s.id == snapshot_id)
        .ok_or_else(|| format!("Snapshot '{snapshot_id}' not found"))?;

    Ok(compare_document_with_snapshot(doc, snapshot))
}

#[tauri::command]
pub fn fork_snapshot(
    snapshot_id: String,
    new_title: String,
    state: State<AppState>,
) -> Result<DocumentStateDto, String> {
    let mut service = state.service.lock().unwrap();
    let snapshot = service
        .document()
        .snapshots
        .iter()
        .find(|s| s.id == snapshot_id)
        .cloned()
        .ok_or_else(|| format!("Snapshot '{snapshot_id}' not found"))?;

    let mut new_doc = PromptDocument::new(new_title, &service.document().description);
    new_doc.sections = snapshot.sections;
    service.load_document(new_doc, None);
    drop(service);

    Ok(state.to_dto())
}


