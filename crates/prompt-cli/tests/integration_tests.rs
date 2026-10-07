use prompt_application::{ApplicationService, Command};
use prompt_core::{
    validate_tag_name, PromptDocument, PromptSection, RenderOptions, RenderStage, ALL_PRESETS,
    TEMPLATE_ENGINEERING, TEMPLATE_GENERAL, TEMPLATE_RESEARCH,
};
use prompt_persistence::{load_document, save_document, AppConfig};
use prompt_refinement::{
    compute_line_diff, generate_manual_request, parse_manual_response, DiffTag,
    RefinementChangeset, RefinementError, RefinementMode, SectionChange,
};
use std::process::Command as SysCommand;
use tempfile::tempdir;
use uuid::Uuid;

// 1. Document creation and serialization
#[test]
fn test_document_creation_and_serialization() {
    let mut doc = PromptDocument::new("Test Document", "Description of test document");
    let sec1 = PromptSection::new("role", "Senior Rust Engineer").unwrap();
    let sec2 = PromptSection::new("task", "Implement high-performance cache").unwrap();
    doc.add_section(sec1).unwrap();
    doc.add_section(sec2).unwrap();

    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("doc.prompt.json");
    save_document(&file_path, &doc).unwrap();

    let restored = load_document(&file_path).unwrap();

    assert_eq!(doc.id, restored.id);
    assert_eq!(doc.title, restored.title);
    assert_eq!(doc.sections.len(), restored.sections.len());
    assert_eq!(doc.sections[0].tag, restored.sections[0].tag);
    assert_eq!(doc.sections[1].brief, restored.sections[1].brief);
}

// 2. XML rendering: draft vs final, with and without section IDs
#[test]
fn test_xml_rendering_draft_vs_final_with_and_without_ids() {
    let mut doc = PromptDocument::new("Prompt", "Description");
    let mut sec = PromptSection::new("role", "Initial Draft Role").unwrap();
    sec.refined = Some("Polished Final Role".to_string());
    doc.add_section(sec).unwrap();

    // Draft with IDs
    let draft_xml = prompt_core::render_xml(
        &doc,
        RenderOptions {
            stage: RenderStage::Draft,
            include_ids: true,
            pretty: true,
        },
    )
    .unwrap();
    assert!(draft_xml.contains("Initial Draft Role"));
    assert!(!draft_xml.contains("Polished Final Role"));
    assert!(draft_xml.contains("id=\""));

    // Final clean (without IDs)
    let final_clean_xml = prompt_core::render_xml(
        &doc,
        RenderOptions {
            stage: RenderStage::Final,
            include_ids: false,
            pretty: true,
        },
    )
    .unwrap();
    assert!(!final_clean_xml.contains("Initial Draft Role"));
    assert!(final_clean_xml.contains("Polished Final Role"));
    assert!(!final_clean_xml.contains("id=\""));
    assert!(final_clean_xml.contains("<role>"));
}

// 3. Invalid XML tag rejection
#[test]
fn test_invalid_xml_tag_rejection() {
    assert!(validate_tag_name("valid_tag").is_ok());
    assert!(validate_tag_name("valid-tag").is_ok());
    assert!(validate_tag_name("role").is_ok());

    // Invalid: empty
    assert!(validate_tag_name("").is_err());
    // Invalid: spaces
    assert!(validate_tag_name("invalid tag").is_err());
    // Invalid: starts with number
    assert!(validate_tag_name("1invalid").is_err());
    // Invalid: starts with xml
    assert!(validate_tag_name("xml_tag").is_err());
    assert!(validate_tag_name("XML").is_err());
    // Invalid: special symbols
    assert!(validate_tag_name("tag$name").is_err());
    assert!(validate_tag_name("<tag>").is_err());
}

// 4. Reordering sections changes rendered XML
#[test]
fn test_reordering_sections_changes_rendered_xml() {
    let mut doc = PromptDocument::new("Reorder Test", "");
    doc.add_section(PromptSection::new("first", "First Content").unwrap()).unwrap();
    doc.add_section(PromptSection::new("second", "Second Content").unwrap()).unwrap();

    let xml1 = prompt_core::render_xml(&doc, RenderOptions::final_clean()).unwrap();
    let idx_first_1 = xml1.find("<first>").unwrap();
    let idx_second_1 = xml1.find("<second>").unwrap();
    assert!(idx_first_1 < idx_second_1);

    // Reorder: move index 0 down to index 1
    doc.move_section(0, 1).unwrap();

    let xml2 = prompt_core::render_xml(&doc, RenderOptions::final_clean()).unwrap();
    let idx_first_2 = xml2.find("<first>").unwrap();
    let idx_second_2 = xml2.find("<second>").unwrap();
    assert!(idx_second_2 < idx_first_2);
}

// 5. Locked sections cannot be modified or deleted
#[test]
fn test_locked_sections_cannot_be_modified_or_deleted() {
    let mut doc = PromptDocument::new("Locked Test", "");
    let mut sec = PromptSection::new("constraints", "Do not modify").unwrap();
    sec.locked = true;
    let sec_id = sec.id;
    doc.add_section(sec).unwrap();

    let mut service = ApplicationService::new(doc);

    // Attempt to delete locked section
    let res_delete = service.execute(Command::RemoveSection { id: sec_id });
    assert!(res_delete.is_err());

    // Attempt to rename tag of locked section
    let res_tag = service.execute(Command::RenameSection {
        id: sec_id,
        tag: "new_tag".to_string(),
    });
    assert!(res_tag.is_err());

    // Attempt to update brief of locked section
    let res_brief = service.execute(Command::UpdateBrief {
        id: sec_id,
        text: "Hacked content".to_string(),
    });
    assert!(res_brief.is_err());

    // Verify content remained intact
    assert_eq!(
        service.document().section(sec_id).unwrap().brief,
        "Do not modify"
    );
}

// 6. Duplicate section generates new UUID and preserves content
#[test]
fn test_duplicate_section_generates_new_uuid_and_preserves_content() {
    let mut doc = PromptDocument::new("Dup Test", "");
    let mut sec = PromptSection::new("example", "Input: A -> Output: B").unwrap();
    sec.refined = Some("Refined Example".to_string());
    sec.locked = true;
    let original_id = sec.id;
    doc.add_section(sec).unwrap();

    let dup = doc.duplicate_section(original_id).unwrap();
    assert_ne!(dup.id, original_id);
    assert_eq!(dup.tag, "example");
    assert_eq!(dup.brief, "Input: A -> Output: B");
    assert_eq!(dup.refined.as_deref(), Some("Refined Example"));
    assert!(!dup.locked, "Duplicated section should default to unlocked");
    assert_eq!(doc.sections.len(), 2);
}

// 7. Undo/redo behavior across structural commands
#[test]
fn test_undo_redo_behavior_across_structural_commands() {
    let doc = PromptDocument::new("Undo Test", "");
    let mut service = ApplicationService::new(doc);

    // Initial state: 0 sections
    assert_eq!(service.document().sections.len(), 0);

    // Step 1: Add section 1
    service
        .execute(Command::AddSection {
            tag: "role".to_string(),
            brief: "Architect".to_string(),
        })
        .unwrap();
    assert_eq!(service.document().sections.len(), 1);
    let id1 = service.document().sections[0].id;

    // Step 2: Add section 2
    service
        .execute(Command::AddSection {
            tag: "task".to_string(),
            brief: "Build service".to_string(),
        })
        .unwrap();
    assert_eq!(service.document().sections.len(), 2);
    let id2 = service.document().sections[1].id;

    // Step 3: Remove section 1
    service.execute(Command::RemoveSection { id: id1 }).unwrap();
    assert_eq!(service.document().sections.len(), 1);
    assert_eq!(service.document().sections[0].id, id2);

    // Undo step 3 (restore section 1)
    assert!(service.undo());
    assert_eq!(service.document().sections.len(), 2);
    assert_eq!(service.document().sections[0].id, id1);

    // Undo step 2 (remove section 2)
    assert!(service.undo());
    assert_eq!(service.document().sections.len(), 1);
    assert_eq!(service.document().sections[0].id, id1);

    // Undo step 1 (back to empty)
    assert!(service.undo());
    assert_eq!(service.document().sections.len(), 0);

    // Redo step 1
    assert!(service.redo());
    assert_eq!(service.document().sections.len(), 1);

    // Redo step 2
    assert!(service.redo());
    assert_eq!(service.document().sections.len(), 2);

    // Redo step 3
    assert!(service.redo());
    assert_eq!(service.document().sections.len(), 1);
    assert_eq!(service.document().sections[0].id, id2);
}

// 8. Refinement engine: Conservative vs Expand mode prompt differences
#[test]
fn test_refinement_engine_conservative_vs_expand_mode() {
    let mut doc = PromptDocument::new("Mode Test", "Description");
    doc.add_section(PromptSection::new("task", "Build an authentication service").unwrap()).unwrap();

    let conservative_req = generate_manual_request(&doc, RefinementMode::Conservative).unwrap();
    let expand_req = generate_manual_request(&doc, RefinementMode::Expand).unwrap();
    let critique_req = generate_manual_request(&doc, RefinementMode::Critique).unwrap();

    assert!(conservative_req.contains("MODE: CONSERVATIVE"));
    assert!(conservative_req.contains("strictly to improve clarity, precision, and phrasing"));

    assert!(expand_req.contains("MODE: EXPAND"));
    assert!(expand_req.contains("elaborating where instructions are vague"));

    assert!(critique_req.contains("MODE: CRITIQUE"));
    assert!(critique_req.contains("ambiguities, contradictions, missing information"));
}

// 9. Manual refinement export and import flow
#[test]
fn test_manual_refinement_export_and_import_flow() {
    let mut doc = PromptDocument::new("Refine Test", "Flow test");
    let sec = PromptSection::new("role", "Rough role notes").unwrap();
    let sec_id = sec.id;
    doc.add_section(sec).unwrap();

    // 1. Export manual request
    let export_text = generate_manual_request(&doc, RefinementMode::Conservative).unwrap();
    assert!(export_text.contains(&sec_id.to_string()));

    // 2. Simulate JSON response from external LLM
    let mock_json_response = format!(
        r#"{{
            "changes": [
                {{
                    "section_id": "{}",
                    "refined": "Distinguished Systems Engineer specializing in distributed databases.",
                    "reason": "Elevated clarity and professional specificity."
                }}
            ],
            "suggested_sections": [],
            "open_questions": [],
            "warnings": []
        }}"#,
        sec_id
    );

    let changeset = parse_manual_response(&mock_json_response, &doc).unwrap();
    assert_eq!(changeset.changes.len(), 1);
    assert_eq!(changeset.changes[0].section_id, sec_id);
    assert!(changeset.changes[0].refined.contains("Distinguished Systems Engineer"));

    // 3. Simulate XML response from external LLM
    let mock_xml_response = format!(
        r#"<prompt_refinement>
            <section id="{}">
                Lead Architect for Mission-Critical Infrastructure.
            </section>
        </prompt_refinement>"#,
        sec_id
    );

    let xml_changeset = parse_manual_response(&mock_xml_response, &doc).unwrap();
    assert_eq!(xml_changeset.changes.len(), 1);
    assert_eq!(xml_changeset.changes[0].section_id, sec_id);
    assert!(xml_changeset.changes[0]
        .refined
        .contains("Lead Architect for Mission-Critical Infrastructure"));
}

// 10. Changeset validation rejects changes targeting unknown UUIDs
#[test]
fn test_changeset_validation_rejects_unknown_uuid() {
    let mut doc = PromptDocument::new("Doc", "");
    doc.add_section(PromptSection::new("role", "Tester").unwrap()).unwrap();

    let unknown_id = Uuid::new_v4();
    let changeset = RefinementChangeset {
        changes: vec![SectionChange {
            section_id: unknown_id,
            refined: "Should fail".to_string(),
            reason: String::new(),
        }],
        suggested_sections: Vec::new(),
        open_questions: Vec::new(),
        warnings: Vec::new(),
        critique: None,
    };

    let val_result = changeset.validate_against_document(&doc);
    assert!(val_result.is_err());
    match val_result {
        Err(RefinementError::UnknownSectionId(id)) => assert_eq!(id, unknown_id),
        other => panic!("Expected UnknownSectionId, got: {:?}", other),
    }
}

// 11. Changeset validation rejects changes targeting locked sections
#[test]
fn test_changeset_validation_rejects_locked_section() {
    let mut doc = PromptDocument::new("Doc", "");
    let mut sec = PromptSection::new("role", "Locked Tester").unwrap();
    sec.locked = true;
    let sec_id = sec.id;
    doc.add_section(sec).unwrap();

    let changeset = RefinementChangeset {
        changes: vec![SectionChange {
            section_id: sec_id,
            refined: "Attempted modification".to_string(),
            reason: String::new(),
        }],
        suggested_sections: Vec::new(),
        open_questions: Vec::new(),
        warnings: Vec::new(),
        critique: None,
    };

    let val_result = changeset.validate_against_document(&doc);
    assert!(val_result.is_err());
    match val_result {
        Err(RefinementError::SectionLocked(id)) => assert_eq!(id, sec_id),
        other => panic!("Expected SectionLocked, got: {:?}", other),
    }
}

// 12. Diff computation detects insertions, deletions, modifications
#[test]
fn test_diff_computation_detects_changes() {
    let original = "Line 1: Context\nLine 2: Target\nLine 3: End";
    let modified = "Line 1: Context\nLine 2: New Target\nLine 2.5: Inserted\nLine 3: End";

    let diff = compute_line_diff(original, modified);

    assert!(diff.iter().any(|d| d.tag == DiffTag::Equal && d.text == "Line 1: Context"));
    assert!(diff.iter().any(|d| d.tag == DiffTag::Delete && d.text == "Line 2: Target"));
    assert!(diff.iter().any(|d| d.tag == DiffTag::Insert && d.text == "Line 2: New Target"));
    assert!(diff.iter().any(|d| d.tag == DiffTag::Insert && d.text == "Line 2.5: Inserted"));
    assert!(diff.iter().any(|d| d.tag == DiffTag::Equal && d.text == "Line 3: End"));
}

// 13. Provider fallback / manual mode when no API keys are present
#[test]
fn test_provider_fallback_manual_mode() {
    let doc = TEMPLATE_ENGINEERING.instantiate(None);
    let request_str = generate_manual_request(&doc, RefinementMode::Conservative).unwrap();

    // Verify it generates complete prompt without network or API key
    assert!(request_str.contains("CRITICAL PRIME DIRECTIVES"));
    assert!(request_str.contains("Document Title: Software Engineering Task"));
    assert!(request_str.contains("<role id="));
    assert!(request_str.contains("<constraints id="));
}

// 14. CLI commands work end-to-end (render, validate, refine)
#[test]
fn test_cli_commands_end_to_end() {
    let temp_dir = tempdir().unwrap();
    let doc_path = temp_dir.path().join("cli_test.prompt.json");
    let bin_path = env!("CARGO_BIN_EXE_promptforge");

    // 1. promptforge new --template engineering --output <path>
    let status_new = SysCommand::new(bin_path)
        .args([
            "new",
            "--template",
            "engineering",
            "--output",
            doc_path.to_str().unwrap(),
        ])
        .status()
        .expect("Failed to execute promptforge new");
    assert!(status_new.success());
    assert!(doc_path.exists());

    // 2. promptforge validate <path>
    let status_validate = SysCommand::new(bin_path)
        .args(["validate", doc_path.to_str().unwrap()])
        .status()
        .expect("Failed to execute promptforge validate");
    assert!(status_validate.success());

    // 3. promptforge render <path> --stage draft
    let output_render_draft = SysCommand::new(bin_path)
        .args(["render", doc_path.to_str().unwrap(), "--stage", "draft"])
        .output()
        .expect("Failed to execute promptforge render draft");
    assert!(output_render_draft.status.success());
    let draft_xml = String::from_utf8(output_render_draft.stdout).unwrap();
    assert!(draft_xml.contains("<prompt>"));
    assert!(draft_xml.contains("id=\""));

    // 4. promptforge render <path> --stage final --clean
    let output_render_clean = SysCommand::new(bin_path)
        .args([
            "render",
            doc_path.to_str().unwrap(),
            "--stage",
            "final",
            "--clean",
        ])
        .output()
        .expect("Failed to execute promptforge render clean");
    assert!(output_render_clean.status.success());
    let clean_xml = String::from_utf8(output_render_clean.stdout).unwrap();
    assert!(clean_xml.contains("<prompt>"));
    assert!(!clean_xml.contains("id=\""));

    // 5. promptforge refine <path> --manual
    let output_refine_manual = SysCommand::new(bin_path)
        .args(["refine", doc_path.to_str().unwrap(), "--manual"])
        .output()
        .expect("Failed to execute promptforge refine --manual");
    assert!(output_refine_manual.status.success());
    let manual_text = String::from_utf8(output_refine_manual.stdout).unwrap();
    assert!(manual_text.contains("SYSTEM INSTRUCTIONS"));

    // 6. promptforge refine <path> --apply <response_file> --in-place
    let doc = load_document(&doc_path).unwrap();
    let role_id = doc.sections[0].id;
    let resp_file = temp_dir.path().join("mock_response.json");
    std::fs::write(
        &resp_file,
        format!(
            r#"{{
                "changes": [
                    {{
                        "section_id": "{}",
                        "refined": "Principal Staff Software Engineer.",
                        "reason": "Elevated persona depth."
                    }}
                ],
                "suggested_sections": [],
                "open_questions": [],
                "warnings": []
            }}"#,
            role_id
        ),
    )
    .unwrap();

    let status_apply = SysCommand::new(bin_path)
        .args([
            "refine",
            doc_path.to_str().unwrap(),
            "--apply",
            resp_file.to_str().unwrap(),
            "--in-place",
        ])
        .status()
        .expect("Failed to execute promptforge refine --apply");
    assert!(status_apply.success());

    // Verify document was updated with refined content
    let updated_doc = load_document(&doc_path).unwrap();
    assert_eq!(
        updated_doc.sections[0].refined.as_deref(),
        Some("Principal Staff Software Engineer.")
    );
}

// 15. Configuration persistence (save/load)
#[test]
fn test_configuration_persistence() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.toml");

    let mut config = AppConfig::default();
    config.openai_compatible.model = "claude-3-5-sonnet".to_string();
    config.openai_compatible.base_url = "https://custom.api.endpoint/v1".to_string();
    config.theme = "light".to_string();

    config.save_to(&config_path).unwrap();
    assert!(config_path.exists());

    let loaded = AppConfig::load_from(&config_path).unwrap();
    assert_eq!(loaded.openai_compatible.model, "claude-3-5-sonnet");
    assert_eq!(
        loaded.openai_compatible.base_url,
        "https://custom.api.endpoint/v1"
    );
    assert_eq!(loaded.theme, "light");
}

// 16. Starter template instantiation produces valid documents
#[test]
fn test_starter_templates_produce_valid_documents() {
    let general = TEMPLATE_GENERAL.instantiate(None);
    let eng = TEMPLATE_ENGINEERING.instantiate(None);
    let research = TEMPLATE_RESEARCH.instantiate(None);

    assert!(general.validate().is_ok());
    assert!(eng.validate().is_ok());
    assert!(research.validate().is_ok());

    assert!(!general.sections.is_empty());
    assert!(!eng.sections.is_empty());
    assert!(!research.sections.is_empty());

    // Verify presets are all valid XML tags
    for preset in ALL_PRESETS {
        assert!(validate_tag_name(preset.tag).is_ok());
    }
}

// 17. Corrupted document recovery: graceful error handling, no crash
#[test]
fn test_corrupted_document_recovery() {
    let temp_dir = tempdir().unwrap();
    let corrupt_file = temp_dir.path().join("corrupt.prompt.json");

    // Write malformed JSON
    std::fs::write(&corrupt_file, "{ not valid json ...").unwrap();

    let load_res = load_document(&corrupt_file);
    assert!(load_res.is_err(), "Loading corrupted document must return an error");

    // Write missing required fields
    std::fs::write(&corrupt_file, r#"{"schema_version": 999}"#).unwrap();
    let load_res2 = load_document(&corrupt_file);
    assert!(load_res2.is_err());
}

// 18. Clean export produces valid XML parseable by standard XML parsers
#[test]
fn test_clean_export_valid_xml_parseable() {
    let doc = TEMPLATE_ENGINEERING.instantiate(None);
    let xml = prompt_core::render_xml(&doc, RenderOptions::final_clean()).unwrap();

    // Parse with quick_xml reader
    let mut reader = quick_xml::Reader::from_str(&xml);
    reader.config_mut().trim_text(true);

    let mut count = 0;
    let mut buf = Vec::new();
    loop {
        match reader.read_event_into(&mut buf) {
            Ok(quick_xml::events::Event::Start(_)) => {
                count += 1;
            }
            Ok(quick_xml::events::Event::Eof) => break,
            Err(e) => panic!("XML parsing failed on clean export: {:?}", e),
            _ => {}
        }
        buf.clear();
    }

    assert!(count >= 5, "Expected at least 5 XML elements parsed, got {}", count);
}

// 19. Agent skill export generation and persistence
#[test]
fn test_skill_export_generation_and_persistence() {
    let mut doc = PromptDocument::new("Rust Security Auditor", "Audits code for memory hazards.");
    let s1 = PromptSection::new("role", "Senior Security Auditor").unwrap();
    let s2 = PromptSection::new("task", "Find undefined behavior").unwrap();
    doc.add_section(s1).unwrap();
    doc.add_section(s2).unwrap();

    let temp_dir = tempdir().unwrap();
    let skill_path = temp_dir.path().join("SKILL.md");

    prompt_persistence::export_skill(
        &skill_path,
        &doc,
        prompt_core::SkillExportOptions::default(),
    )
    .unwrap();

    let content = std::fs::read_to_string(&skill_path).unwrap();
    assert!(content.starts_with("---\nname: rust-security-auditor\n"));
    assert!(content.contains("description: Audits code for memory hazards."));
    assert!(content.contains("# Rust Security Auditor"));
    assert!(content.contains("<role>"));
    assert!(content.contains("Senior Security Auditor"));
}

// 20. Multi-format prompt import (XML, Markdown, Agent Skill, JSON)
#[test]
fn test_multi_format_prompt_import() {
    // A. XML Import
    let xml = "<prompt><role>DB Admin</role><task>Optimize indexes</task></prompt>";
    let imported_xml = prompt_core::import_prompt_from_text(xml).unwrap();
    assert_eq!(imported_xml.format, prompt_core::ImportFormat::Xml);
    assert_eq!(imported_xml.sections.len(), 2);
    assert_eq!(imported_xml.sections[0].tag, "role");

    // B. Markdown Import
    let md = "# Code Reviewer\nReviewing code.\n\n## Role\nPrincipal Engineer\n\n## Constraints\nNo unwraps";
    let imported_md = prompt_core::import_prompt_from_text(md).unwrap();
    assert_eq!(imported_md.format, prompt_core::ImportFormat::Markdown);
    assert_eq!(imported_md.title, "Code Reviewer");
    assert_eq!(imported_md.sections.len(), 2);

    // C. Agent Skill Import
    let skill = "---\nname: my-skill\ndescription: Test skill\n---\n# My Skill\n\n```xml\n<prompt><task>Run benchmarks</task></prompt>\n```";
    let imported_skill = prompt_core::import_prompt_from_text(skill).unwrap();
    assert_eq!(imported_skill.format, prompt_core::ImportFormat::Skill);
    assert_eq!(imported_skill.sections[0].tag, "task");
    assert_eq!(imported_skill.sections[0].brief, "Run benchmarks");
}

// 21. Prompt Library roundtrip save, list, load, and delete
#[test]
fn test_prompt_library_save_load_delete_roundtrip() {
    let temp_dir = tempdir().unwrap();
    let mut doc = PromptDocument::new("Refactoring Assistant", "Assists in cleaning code.");
    let mut sec = PromptSection::new("role", "Refactoring Expert").unwrap();
    sec.tags = vec!["clean-code".to_string(), "refactor".to_string()];
    doc.add_section(sec).unwrap();

    // Save to library
    let summary = prompt_persistence::save_to_library(temp_dir.path(), &doc).unwrap();
    assert_eq!(summary.title, "Refactoring Assistant");
    assert_eq!(summary.tags, vec!["clean-code".to_string(), "refactor".to_string()]);

    // List library
    let list = prompt_persistence::list_saved_prompts(temp_dir.path()).unwrap();
    assert_eq!(list.len(), 1);
    assert_eq!(list[0].id, doc.id.to_string());

    // Load from library
    let loaded = prompt_persistence::load_from_library(temp_dir.path(), &doc.id.to_string()).unwrap();
    assert_eq!(loaded.id, doc.id);
    assert_eq!(loaded.title, doc.title);

    // Delete from library
    let deleted = prompt_persistence::delete_from_library(temp_dir.path(), &doc.id.to_string()).unwrap();
    assert!(deleted);
    let list_after = prompt_persistence::list_saved_prompts(temp_dir.path()).unwrap();
    assert_eq!(list_after.len(), 0);
}

// 22. Tag color configuration persistence
#[test]
fn test_tag_colors_configuration_persistence() {
    let temp_dir = tempdir().unwrap();
    let config_path = temp_dir.path().join("config.json");

    let mut cfg = AppConfig::default();
    cfg.set_tag_color("persona".to_string(), "violet".to_string());
    cfg.set_tag_color("security".to_string(), "rose".to_string());
    cfg.save_to(&config_path).unwrap();

    let loaded = AppConfig::load_from(&config_path).unwrap();
    assert_eq!(loaded.tag_colors.get("persona").unwrap(), "violet");
    assert_eq!(loaded.tag_colors.get("security").unwrap(), "rose");
}

// 23. Dynamic prompt variables, scenario interpolation, and telemetry
#[test]
fn test_dynamic_variables_and_scenario_interpolation() {
    let mut doc = PromptDocument::new("Variable Assistant", "Testing variables");
    let sec1 = PromptSection::new("role", "You are an assistant for {{target_role}}.").unwrap();
    let sec2 = PromptSection::new("task", "Analyze {{target_framework}} in {{environment}}.").unwrap();
    doc.add_section(sec1).unwrap();
    doc.add_section(sec2).unwrap();

    // Variable extraction
    let vars = prompt_core::extract_document_variables(&doc);
    assert_eq!(vars, vec!["environment", "target_framework", "target_role"]);

    // Test scenario interpolation
    let mut scenario_vars = std::collections::HashMap::new();
    scenario_vars.insert("target_role".to_string(), "DevOps Engineer".to_string());
    scenario_vars.insert("target_framework".to_string(), "Kubernetes".to_string());
    scenario_vars.insert("environment".to_string(), "Production".to_string());

    let rendered_xml = prompt_core::render_interpolated_xml(
        &doc,
        RenderOptions {
            stage: RenderStage::Final,
            include_ids: false,
            pretty: true,
        },
        &scenario_vars,
    )
    .unwrap();

    assert!(rendered_xml.contains("You are an assistant for DevOps Engineer."));
    assert!(rendered_xml.contains("Analyze Kubernetes in Production."));

    // Telemetry
    let (char_count, word_count, estimated_tokens) =
        prompt_core::calculate_prompt_telemetry(&rendered_xml);
    assert!(char_count > 0);
    assert!(word_count > 0);
    assert!(estimated_tokens > 0);

    // Save scenario via ApplicationService command
    let mut service = ApplicationService::new(doc);
    let scenario = prompt_core::TestScenario {
        id: "scen-1".to_string(),
        name: "DevOps Prod Scenario".to_string(),
        description: Some("Prod testing matrix".to_string()),
        variables: scenario_vars,
    };
    service.execute(Command::SaveScenario { scenario }).unwrap();
    assert_eq!(service.state().document.scenarios.len(), 1);
    assert_eq!(service.state().document.scenarios[0].name, "DevOps Prod Scenario");

    // Delete scenario
    service.execute(Command::DeleteScenario { id: "scen-1".to_string() }).unwrap();
    assert_eq!(service.state().document.scenarios.len(), 0);
}

// 24. Document snapshots, diff branching, and rollback
#[test]
fn test_snapshot_history_diff_and_restore() {
    let mut doc = PromptDocument::new("Architecture Blueprint", "Initial architectural specification");
    let sec1 = PromptSection::new("role", "Principal Architect").unwrap();
    let sec2 = PromptSection::new("task", "Draft distributed consensus architecture").unwrap();
    doc.add_section(sec1).unwrap();
    doc.add_section(sec2).unwrap();

    let mut service = ApplicationService::new(doc);

    // Create named snapshot v1.0
    service
        .execute(Command::CreateSnapshot {
            name: "v1.0 Baseline".to_string(),
            description: Some("Initial release milestone".to_string()),
        })
        .unwrap();

    assert_eq!(service.state().document.snapshots.len(), 1);
    let snapshot_id = service.state().document.snapshots[0].id.clone();

    // Modify document: edit task, add constraints, rename title
    let task_id = service.state().document.sections[1].id;
    service
        .execute(Command::UpdateTitle {
            title: "Architecture Blueprint v2".to_string(),
        })
        .unwrap();
    service
        .execute(Command::UpdateBrief {
            id: task_id,
            text: "Draft Raft consensus engine with log compaction".to_string(),
        })
        .unwrap();
    service
        .execute(Command::AddSection {
            tag: "constraints".to_string(),
            brief: "Zero unsafe blocks allowed".to_string(),
        })
        .unwrap();

    // Compare active document with snapshot v1.0
    let snapshot = &service.state().document.snapshots[0];
    let comparison = prompt_core::compare_document_with_snapshot(&service.state().document, snapshot);

    assert!(comparison.is_title_changed);
    assert_eq!(comparison.snapshot_title, "Architecture Blueprint");
    assert_eq!(comparison.current_title, "Architecture Blueprint v2");

    // Section diffs
    // Role: Unchanged
    // Task: Modified
    // Constraints: Added
    let role_diff = comparison.section_diffs.iter().find(|d| d.tag == "role").unwrap();
    assert_eq!(role_diff.status, prompt_core::SectionDiffStatus::Unchanged);

    let task_diff = comparison.section_diffs.iter().find(|d| d.tag == "task").unwrap();
    assert_eq!(task_diff.status, prompt_core::SectionDiffStatus::Modified);

    let constr_diff = comparison.section_diffs.iter().find(|d| d.tag == "constraints").unwrap();
    assert_eq!(constr_diff.status, prompt_core::SectionDiffStatus::Added);

    // Restore snapshot v1.0
    service
        .execute(Command::RestoreSnapshot {
            id: snapshot_id.clone(),
        })
        .unwrap();

    assert_eq!(service.state().document.title, "Architecture Blueprint");
    assert_eq!(service.state().document.sections.len(), 2);
    assert_eq!(
        service.state().document.sections[1].brief,
        "Draft distributed consensus architecture"
    );

    // Verify undo works on snapshot restore!
    service.execute(Command::Undo).unwrap();
    assert_eq!(service.state().document.title, "Architecture Blueprint v2");
    assert_eq!(service.state().document.sections.len(), 3);

    // Delete snapshot
    service
        .execute(Command::DeleteSnapshot { id: snapshot_id })
        .unwrap();
    assert_eq!(service.state().document.snapshots.len(), 0);
}


