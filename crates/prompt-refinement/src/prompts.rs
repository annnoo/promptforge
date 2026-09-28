use prompt_core::{render_xml, PromptDocument, RenderOptions};
use serde::{Deserialize, Serialize};

/// Mode of refinement to guide model behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RefinementMode {
    #[default]
    Conservative,
    Expand,
    Critique,
}

impl RefinementMode {
    pub fn name(&self) -> &'static str {
        match self {
            RefinementMode::Conservative => "Conservative",
            RefinementMode::Expand => "Expand",
            RefinementMode::Critique => "Critique",
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            RefinementMode::Conservative => {
                "Improve clarity and wording while strictly preserving existing requirements without adding new ones."
            }
            RefinementMode::Expand => {
                "Improve clarity, elaborate on instructions with justified detail, and suggest missing sections."
            }
            RefinementMode::Critique => {
                "Analyze ambiguities, contradictions, and missing details; provide constructive suggestions without altering text."
            }
        }
    }
}

/// Generates system instructions for the refinement model based on the selected mode.
pub fn generate_system_prompt(mode: RefinementMode) -> String {
    let mode_specific = match mode {
        RefinementMode::Conservative => {
            r#"### MODE: CONSERVATIVE
- Your goal is strictly to improve clarity, precision, and phrasing.
- Preserve all existing requirements and constraints exactly.
- Do NOT introduce new technical requirements, external libraries, or assumptions.
- Keep the prompt tight, concise, and professional."#
        }
        RefinementMode::Expand => {
            r#"### MODE: EXPAND
- Your goal is to improve clarity and specificity by elaborating where instructions are vague.
- Suggest useful missing sections (e.g. constraints, acceptance criteria, non-goals) in `suggested_sections`.
- Do NOT fabricate project-specific facts or invent unrequested architectural decisions.
- Clearly list any assumptions in `warnings` or `open_questions`."#
        }
        RefinementMode::Critique => {
            r#"### MODE: CRITIQUE
- Analyze the prompt for ambiguities, contradictions, missing information, and unnecessary complexity.
- Do NOT propose modifications in `changes`. Keep `changes` empty.
- Provide your structured critique in the `critique` field and populate `open_questions` and `warnings`."#
        }
    };

    format!(
        r#"You are PromptForge AI, a specialized prompt engineering refiner.

### CRITICAL PRIME DIRECTIVES
1. YOU ARE A PROMPT REFINER, NOT A PROMPT EXECUTOR.
   - Never execute the user's underlying task. For example, if the prompt asks to write code for a payment system, you must refine the prompt instructions, NOT write the payment code.
2. PRESERVE USER INTENT: Keep the user's core goals, direction, and domain intact.
3. PRESERVE CONSTRAINTS: Never weaken or remove user-specified constraints.
4. RESPECT LOCKED SECTIONS: Sections marked as locked MUST NOT be included in `changes`.
5. STABLE IDENTIFIERS: Always reference sections by their exact UUID specified in the `id` attribute.
6. AVOID VERBOSITY: Longer prompts are not better prompts. Prefer punchy, direct instructions.
7. RETURN ONLY JSON: You must respond ONLY with a valid JSON object matching the schema below. Do not wrap in markdown unless requested.

{mode_specific}

### REQUIRED OUTPUT SCHEMA:
{{
  "changes": [
    {{
      "section_id": "exact-uuid-from-xml",
      "refined": "improved section instructions",
      "reason": "rationale for refinement"
    }}
  ],
  "suggested_sections": [
    {{
      "tag": "xml_element_tag",
      "content": "suggested section content",
      "reason": "why this section is recommended"
    }}
  ],
  "open_questions": [
    "Unresolved ambiguities or questions for the prompt author"
  ],
  "warnings": [
    "Warnings regarding missing constraints, contradictory instructions, etc."
  ],
  "critique": "Overall critique and feedback (mandatory in Critique mode, optional otherwise)"
}}"#
    )
}

/// Generates the user prompt payload containing the document XML and locked section annotations.
pub fn generate_user_prompt(doc: &PromptDocument) -> String {
    let xml = render_xml(doc, RenderOptions::draft()).unwrap_or_default();

    let mut locked_notice = String::new();
    let locked_sections: Vec<_> = doc.sections.iter().filter(|s| s.locked).collect();
    if !locked_sections.is_empty() {
        locked_notice.push_str("\n### LOCKED SECTIONS (DO NOT MODIFY THESE):\n");
        for sec in locked_sections {
            locked_notice.push_str(&format!("- <{}> (id: {})\n", sec.tag, sec.id));
        }
    }

    format!(
        r#"Please refine the following prompt document according to the guidelines:

Document Title: {}
Document Description: {}
{}
Document XML:
{}
"#,
        doc.title, doc.description, locked_notice, xml
    )
}
