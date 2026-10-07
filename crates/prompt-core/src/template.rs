use crate::document::PromptDocument;
use crate::section::PromptSection;

use serde::{Deserialize, Serialize};

/// Preset definition for quick creation of standardized prompt sections.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionPreset {
    pub name: &'static str,
    pub tag: &'static str,
    pub description: &'static str,
    pub default_brief: &'static str,
}

pub const PRESET_ROLE: SectionPreset = SectionPreset {
    name: "Role",
    tag: "role",
    description: "The persona, seniority, or specialized perspective the LLM should adopt.",
    default_brief: "Expert software engineer and systems architect.",
};

pub const PRESET_CONTEXT: SectionPreset = SectionPreset {
    name: "Context",
    tag: "context",
    description: "Background information, environment, domain knowledge, or existing system setup.",
    default_brief: "Current system background, architecture, and technology stack.",
};

pub const PRESET_OBJECTIVE: SectionPreset = SectionPreset {
    name: "Objective",
    tag: "objective",
    description: "High-level goal or business outcome to achieve.",
    default_brief: "Primary goal and intended impact of this request.",
};

pub const PRESET_TASK: SectionPreset = SectionPreset {
    name: "Task",
    tag: "task",
    description: "Specific, actionable instructions for what the LLM must execute or produce.",
    default_brief: "Detailed step-by-step task instructions.",
};

pub const PRESET_REQUIREMENTS: SectionPreset = SectionPreset {
    name: "Requirements",
    tag: "requirements",
    description: "Functional and technical requirements that must be met.",
    default_brief: "- Requirement 1\n- Requirement 2\n- Requirement 3",
};

pub const PRESET_CONSTRAINTS: SectionPreset = SectionPreset {
    name: "Constraints",
    tag: "constraints",
    description: "Boundaries, limitations, things to avoid, or mandatory restrictions.",
    default_brief: "- Do not break backward compatibility\n- No third-party network dependencies\n- Keep memory footprint minimal",
};

pub const PRESET_INPUT: SectionPreset = SectionPreset {
    name: "Input",
    tag: "input",
    description: "Data, schema, code, or materials provided for the task.",
    default_brief: "Input data, schema specification, or raw code.",
};

pub const PRESET_EXAMPLES: SectionPreset = SectionPreset {
    name: "Examples",
    tag: "examples",
    description: "Few-shot examples demonstrating input-to-output expectations.",
    default_brief: "Example 1:\nInput: ...\nOutput: ...",
};

pub const PRESET_OUTPUT_FORMAT: SectionPreset = SectionPreset {
    name: "Output Format",
    tag: "output_format",
    description: "Expected structure, schema, language, or serialization format of the response.",
    default_brief: "Provide output as structured JSON / Markdown / XML as specified.",
};

pub const PRESET_ACCEPTANCE_CRITERIA: SectionPreset = SectionPreset {
    name: "Acceptance Criteria",
    tag: "acceptance_criteria",
    description: "Concrete verification points determining when the output is considered done and acceptable.",
    default_brief: "- Passes all automated unit tests\n- Adheres to clean code principles\n- Comprehensive documentation included",
};

pub const PRESET_NON_GOALS: SectionPreset = SectionPreset {
    name: "Non-Goals",
    tag: "non_goals",
    description: "Things explicitly out of scope for this prompt.",
    default_brief: "- Do not refactor existing unrelated modules\n- Do not add external UI dependencies",
};

pub const PRESET_STYLE: SectionPreset = SectionPreset {
    name: "Style",
    tag: "style",
    description: "Tone, voice, formatting preferences, and conciseness guidelines.",
    default_brief: "Concise, precise, and direct. Avoid conversational filler.",
};

pub const PRESET_ADDITIONAL_INSTRUCTIONS: SectionPreset = SectionPreset {
    name: "Additional Instructions",
    tag: "additional_instructions",
    description: "Specialized rules, edge cases, or supplementary guidance.",
    default_brief: "Any edge cases or supplementary notes.",
};

/// All available predefined section presets.
pub const ALL_PRESETS: &[SectionPreset] = &[
    PRESET_ROLE,
    PRESET_CONTEXT,
    PRESET_OBJECTIVE,
    PRESET_TASK,
    PRESET_REQUIREMENTS,
    PRESET_CONSTRAINTS,
    PRESET_INPUT,
    PRESET_EXAMPLES,
    PRESET_OUTPUT_FORMAT,
    PRESET_ACCEPTANCE_CRITERIA,
    PRESET_NON_GOALS,
    PRESET_STYLE,
    PRESET_ADDITIONAL_INSTRUCTIONS,
];

impl SectionPreset {
    /// Creates a new `PromptSection` populated with preset defaults.
    pub fn to_section(&self) -> PromptSection {
        PromptSection::new(self.tag, self.default_brief)
            .expect("preset tags are known valid XML element names")
    }

    /// Finds a preset by tag name or preset name.
    pub fn find_by_tag(tag: &str) -> Option<&'static SectionPreset> {
        ALL_PRESETS
            .iter()
            .find(|p| p.tag.eq_ignore_ascii_case(tag) || p.name.eq_ignore_ascii_case(tag))
    }
}

/// Starter template definition.
#[derive(Debug, Clone, Serialize)]
pub struct StarterTemplate {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub presets: &'static [SectionPreset],
}

pub const TEMPLATE_GENERAL: StarterTemplate = StarterTemplate {
    id: "general",
    name: "General Task",
    description: "Well-structured foundation for general-purpose AI tasks.",
    presets: &[
        PRESET_ROLE,
        PRESET_CONTEXT,
        PRESET_TASK,
        PRESET_CONSTRAINTS,
        PRESET_OUTPUT_FORMAT,
    ],
};

pub const TEMPLATE_ENGINEERING: StarterTemplate = StarterTemplate {
    id: "engineering",
    name: "Software Engineering Task",
    description: "Specialized prompt structure for software architecture, coding, and debugging tasks.",
    presets: &[
        PRESET_ROLE,
        PRESET_CONTEXT,
        PRESET_OBJECTIVE,
        PRESET_TASK,
        PRESET_REQUIREMENTS,
        PRESET_CONSTRAINTS,
        PRESET_ACCEPTANCE_CRITERIA,
    ],
};

pub const TEMPLATE_RESEARCH: StarterTemplate = StarterTemplate {
    id: "research",
    name: "Research and Analysis",
    description: "Structured prompt for deep-dive investigation, synthesis, and report generation.",
    presets: &[
        PRESET_ROLE,
        PRESET_CONTEXT,
        PRESET_TASK,
        PRESET_INPUT,
        PRESET_CONSTRAINTS,
        PRESET_OUTPUT_FORMAT,
        PRESET_STYLE,
    ],
};

pub const ALL_TEMPLATES: &[StarterTemplate] = &[
    TEMPLATE_GENERAL,
    TEMPLATE_ENGINEERING,
    TEMPLATE_RESEARCH,
];

impl StarterTemplate {
    /// Instantiates a new `PromptDocument` populated with the sections of this template.
    pub fn instantiate(&self, title: Option<&str>) -> PromptDocument {
        let title = title.unwrap_or(self.name);
        let mut doc = PromptDocument::new(title, self.description);
        for preset in self.presets {
            let section = preset.to_section();
            doc.add_section(section).expect("preset section is valid");
        }
        doc
    }

    /// Finds a starter template by ID or name.
    pub fn find(id_or_name: &str) -> Option<&'static StarterTemplate> {
        ALL_TEMPLATES.iter().find(|t| {
            t.id.eq_ignore_ascii_case(id_or_name) || t.name.eq_ignore_ascii_case(id_or_name)
        })
    }
}

/// A manageable Section Type representing a reusable prompt section component with folder and tags.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SectionType {
    pub id: String,
    pub name: String,
    pub tag: String,
    pub description: String,
    pub default_brief: String,
    #[serde(default = "default_type_folder")]
    pub folder: String,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub is_builtin: bool,
}

pub fn default_type_folder() -> String {
    "General".to_string()
}

pub fn get_builtin_section_types() -> Vec<SectionType> {
    vec![
        SectionType {
            id: "builtin-role".to_string(),
            name: "Role".to_string(),
            tag: "role".to_string(),
            description: "The persona, seniority, or specialized perspective the LLM should adopt.".to_string(),
            default_brief: "Expert software engineer and systems architect.".to_string(),
            folder: "Core Structure".to_string(),
            tags: vec!["persona".to_string(), "architecture".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-context".to_string(),
            name: "Context".to_string(),
            tag: "context".to_string(),
            description: "Background information, environment, domain knowledge, or existing system setup.".to_string(),
            default_brief: "Current system background, architecture, and technology stack.".to_string(),
            folder: "Core Structure".to_string(),
            tags: vec!["background".to_string(), "domain".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-objective".to_string(),
            name: "Objective".to_string(),
            tag: "objective".to_string(),
            description: "High-level goal or business outcome to achieve.".to_string(),
            default_brief: "Primary goal and intended impact of this request.".to_string(),
            folder: "Core Structure".to_string(),
            tags: vec!["goals".to_string(), "business".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-task".to_string(),
            name: "Task".to_string(),
            tag: "task".to_string(),
            description: "Specific, actionable instructions for what the LLM must execute or produce.".to_string(),
            default_brief: "Detailed step-by-step task instructions.".to_string(),
            folder: "Core Structure".to_string(),
            tags: vec!["instructions".to_string(), "core".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-requirements".to_string(),
            name: "Requirements".to_string(),
            tag: "requirements".to_string(),
            description: "Functional and technical requirements that must be met.".to_string(),
            default_brief: "- Requirement 1\n- Requirement 2\n- Requirement 3".to_string(),
            folder: "Rules & Boundaries".to_string(),
            tags: vec!["specs".to_string(), "criteria".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-constraints".to_string(),
            name: "Constraints".to_string(),
            tag: "constraints".to_string(),
            description: "Boundaries, limitations, things to avoid, or mandatory restrictions.".to_string(),
            default_brief: "- Do not break backward compatibility\n- No third-party network dependencies\n- Keep memory footprint minimal".to_string(),
            folder: "Rules & Boundaries".to_string(),
            tags: vec!["limits".to_string(), "strict".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-negative-rules".to_string(),
            name: "Negative Rules".to_string(),
            tag: "negative_rules".to_string(),
            description: "Explicit negative guardrails and prohibited patterns or outputs.".to_string(),
            default_brief: "- Never use deprecated APIs\n- Never produce markdown code without syntax language tags\n- Never assume unverified external inputs".to_string(),
            folder: "Rules & Boundaries".to_string(),
            tags: vec!["safety".to_string(), "prohibitions".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-input".to_string(),
            name: "Input".to_string(),
            tag: "input".to_string(),
            description: "Data, schema, code, or materials provided for the task.".to_string(),
            default_brief: "Input specifications, sample data, or source schema.".to_string(),
            folder: "Data & Schema".to_string(),
            tags: vec!["data".to_string(), "schema".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-output-format".to_string(),
            name: "Output Format".to_string(),
            tag: "output_format".to_string(),
            description: "Exact structure, schema, packaging, or envelope expected for output.".to_string(),
            default_brief: "Structured format requirements (e.g. JSON schema, Markdown RFC, unified diff).".to_string(),
            folder: "Data & Schema".to_string(),
            tags: vec!["format".to_string(), "schema".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-examples".to_string(),
            name: "Examples".to_string(),
            tag: "examples".to_string(),
            description: "Few-shot demonstrations or expected input/output pairs.".to_string(),
            default_brief: "Example 1:\nInput: ...\nOutput: ...".to_string(),
            folder: "Data & Schema".to_string(),
            tags: vec!["few-shot".to_string(), "patterns".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-chain-of-thought".to_string(),
            name: "Chain of Thought".to_string(),
            tag: "chain_of_thought".to_string(),
            description: "Reasoning methodology or explicit scratchpad instructions.".to_string(),
            default_brief: "Analyze architectural invariants first, evaluate trade-offs, then produce the output.".to_string(),
            folder: "Reasoning & Quality".to_string(),
            tags: vec!["reasoning".to_string(), "logic".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-evaluation".to_string(),
            name: "Evaluation".to_string(),
            tag: "evaluation".to_string(),
            description: "Verification rules, acceptance checks, or self-critique rubric.".to_string(),
            default_brief: "Self-check checklist:\n1. All edge cases handled\n2. Complexity bounds respected\n3. Memory safety confirmed".to_string(),
            folder: "Reasoning & Quality".to_string(),
            tags: vec!["testing".to_string(), "qa".to_string()],
            is_builtin: true,
        },
        SectionType {
            id: "builtin-style".to_string(),
            name: "Style".to_string(),
            tag: "style".to_string(),
            description: "Tone, voice, formatting, and prose style preferences.".to_string(),
            default_brief: "Concise, precise, and authoritative engineering tone without filler text.".to_string(),
            folder: "Reasoning & Quality".to_string(),
            tags: vec!["tone".to_string(), "guidelines".to_string()],
            is_builtin: true,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_presets_validity() {
        for preset in ALL_PRESETS {
            let sec = preset.to_section();
            assert_eq!(sec.tag, preset.tag);
            assert!(!sec.brief.is_empty());
        }
    }

    #[test]
    fn test_templates_instantiate() {
        for template in ALL_TEMPLATES {
            let doc = template.instantiate(None);
            assert_eq!(doc.sections.len(), template.presets.len());
            assert!(doc.validate().is_ok());
        }
    }

    #[test]
    fn test_builtin_section_types() {
        let types = get_builtin_section_types();
        assert_eq!(types.len(), 13);
        for t in &types {
            assert!(!t.id.is_empty());
            assert!(!t.name.is_empty());
            assert!(!t.tag.is_empty());
            assert!(!t.folder.is_empty());
            assert!(!t.tags.is_empty());
            assert!(t.is_builtin);
        }
    }
}
