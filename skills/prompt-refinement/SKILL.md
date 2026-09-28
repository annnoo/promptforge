---
name: prompt-refinement
description: Refine, critique, or expand PromptForge prompt documents into structured LLM changesets preserving intent, constraints, and section identities.
---

# PromptForge Refinement Skill

This skill guides AI agents in refining, expanding, or critiquing structured prompt engineering documents managed by **PromptForge**.

## When to Activate

Activate this skill when:
- The user requests refinement, review, or optimization of a PromptForge prompt document (`.prompt.json` or rendered XML).
- The user asks for suggestions to improve a structured LLM prompt.
- Working with the PromptForge CLI to validate, render, or refine prompt documents.
- Generating structured refinement changesets that can be directly imported into PromptForge GUI or TUI.

## Core Directives

1. **NEVER Execute the Underlying Task**:
   If the prompt instructs an LLM to design an API, build a service, or write a poem, your role is to refine the *instructions of the prompt*, **NOT** to design the API, build the service, or write the poem.
2. **Preserve User Intent**:
   Retain the author's voice, scope, architectural choices, and domain goals.
3. **Preserve Explicit Constraints**:
   Never remove, weaken, or overlook negative constraints, performance bounds, or restrictions.
4. **Do Not Fabricate Unknown Facts**:
   Do not invent imaginary backend endpoints, database credentials, specific libraries, or company policies unless explicitly instructed.
5. **Respect Locked Sections**:
   Sections marked with `locked: true` (or noted as locked in refinement requests) must NOT have changes proposed.
6. **Preserve Section Identifiers**:
   Always refer to existing sections using their exact immutable UUIDs.
7. **Brevity is Clarity**:
   Do not equate length with quality. Tight, punchy instructions are often superior to verbose paragraphs.

## PromptForge Document Model

A PromptForge document contains:
- `schema_version`: Document schema version (currently `1`).
- `id`: Unique UUID of the document.
- `title`: Human-readable prompt title.
- `description`: Document overview or goal.
- `sections`: Ordered collection of `PromptSection`:
  - `id`: Immutable UUID.
  - `tag`: XML element tag name (e.g., `role`, `context`, `task`, `constraints`, `output_format`).
  - `brief`: User's original notes and raw instructions.
  - `refined`: Accepted refined version (optional).
  - `locked`: Boolean; if true, automated changes must not target this section.
  - `enabled`: Boolean; if false, omitted from rendered output.

## Refinement Modes

### 1. Conservative
- Focuses strictly on clarity, conciseness, and precision.
- Removes vague language and eliminates unnecessary filler.
- Preserves all requirements exactly as specified.
- Does not introduce new technical requirements.

### 2. Expand
- Improves clarity and specificity by elaborating on ambiguous directives.
- Suggests useful missing sections (e.g., `acceptance_criteria`, `constraints`, `non_goals`) in `suggested_sections`.
- Discloses assumptions clearly in `warnings` or `open_questions`.
- Never presents assumptions as established facts.

### 3. Critique
- Analyzes existing instructions for internal contradictions, missing prerequisites, ambiguities, or excessive complexity.
- Leaves `changes` empty.
- Provides deep analytical feedback in `critique`, `open_questions`, and `warnings`.

## Expected Changeset Schema

When refining a prompt document, the output must adhere strictly to this JSON structure:

```json
{
  "changes": [
    {
      "section_id": "00000000-0000-0000-0000-000000000000",
      "refined": "Precise, polished instructions replacing the original brief.",
      "reason": "Clarifies ambiguous edge cases and eliminates conversational fluff."
    }
  ],
  "suggested_sections": [
    {
      "tag": "acceptance_criteria",
      "content": "- All public functions documented with rustdoc.\n- Zero compiler warnings with strict clippy.",
      "reason": "Provides verifiable completion boundaries for the executing model."
    }
  ],
  "open_questions": [
    "Should backward compatibility be maintained with the legacy v1 format?"
  ],
  "warnings": [
    "No output format was specified; downstream parsers may encounter varied formatting."
  ],
  "critique": "Overall evaluation of prompt quality (populated in Critique mode or when notable structural feedback applies)."
}
```

## PromptForge CLI Integration

When PromptForge is installed locally, you can invoke the CLI directly:

```bash
# Validate document structure and XML tags
promptforge validate architecture.prompt.json

# Render draft XML (original briefs)
promptforge render architecture.prompt.json --stage draft

# Render final clean XML (accepted refinements, no internal IDs)
promptforge render architecture.prompt.json --stage final --clean

# Generate manual refinement request text
promptforge refine architecture.prompt.json --mode conservative --manual

# Refine with configured OpenAI-compatible endpoint
promptforge refine architecture.prompt.json --mode expand --provider openai-compatible --model gpt-4o

# Install this skill into Claude Code, Codex, or custom directory
promptforge skill install --target ~/.config/claude/skills
```
