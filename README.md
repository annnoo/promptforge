# PromptForge 🔨⚡

> **A local-first, native Rust prompt engineering workbench for building, refining, and reusing high-quality structured LLM prompts.**

PromptForge treats prompts not as disposable scratchpad strings, but as structured, modular engineering artifacts. It provides visual section assembly, deterministic XML compilation, non-destructive refinement workflows, diff inspection, and multi-interface support (Desktop GUI, Terminal TUI, and headless CLI).

---

## Table of Contents

- [Vision & Architecture](#vision--architecture)
- [Crate Structure](#crate-structure)
- [Installation & Build](#installation--build)
- [Quick Start](#quick-start)
- [Interfaces](#interfaces)
  - [Graphical Interface (GUI)](#graphical-interface-gui)
  - [Terminal Interface (TUI)](#terminal-interface-tui)
  - [Headless CLI](#headless-cli)
- [Refinement Workflow](#refinement-workflow)
  - [The 3 Refinement Modes](#the-3-refinement-modes)
  - [Manual Refinement (Zero API Keys)](#manual-refinement-zero-api-keys)
  - [Automated Provider (OpenAI-Compatible)](#automated-provider-openai-compatible)
- [Agent Skill Integration](#agent-skill-integration)
- [File Format Specification](#file-format-specification)
- [Verification & Testing](#verification--testing)
- [License](#license)

---

## Vision & Architecture

Modern Large Language Models (such as Claude 3.5 Sonnet, GPT-4o, and Gemini 1.5 Pro) adhere best to prompts structured into distinct XML tags (`<role>`, `<context>`, `<task>`, `<constraints>`, `<output_format>`). 

PromptForge provides a complete workbench designed around this philosophy:

1. **Local-First & Private:** Pure native Rust with zero telemetry, zero forced cloud dependencies, and zero mandatory API accounts.
2. **Non-Destructive Refinement:** Your original thoughts and briefs are preserved alongside refined drafts. You can compare changes side-by-side using unified line diffs before accepting or rejecting them.
3. **Deterministic XML Compilation:** XML rendering is strictly ordered, properly escaped, and guaranteed identical across GUI, TUI, and CLI.
4. **Three Native Frontends:** Use the rich desktop GUI (`egui`), the high-speed terminal UI (`ratatui`), or shell automation (`clap`).

```
┌────────────────────────────────────────────────────────┐
│                      PromptForge                       │
├───────────────────┬───────────────────┬────────────────┤
│  prompt-gui (egui)│ prompt-tui (ratatui)│prompt-cli (clap)│
└─────────┬─────────┴─────────┬─────────┴────────┬───────┘
          │                   │                  │
          ▼                   ▼                  ▼
┌────────────────────────────────────────────────────────┐
│             prompt-application (Engine)                │
│    (ApplicationService, Command Dispatcher, Undo/Redo) │
├───────────────────────┬────────────────────────────────┤
│   prompt-refinement   │       prompt-persistence       │
│  (Diff, Manual/HTTP)  │      (Atomic Save, Config)     │
└───────────┬───────────┴────────────────┬───────────────┘
            │                            │
            ▼                            ▼
┌────────────────────────────────────────────────────────┐
│                  prompt-core (Domain)                  │
│  (PromptDocument, PromptSection, XML Rendering/Parser) │
└────────────────────────────────────────────────────────┘
```

---

## Crate Structure

PromptForge is organized as a Cargo workspace with strict layer isolation:

| Crate | Purpose | Core Dependencies |
| :--- | :--- | :--- |
| `crates/prompt-core` | Pure domain model, schema validation, presets, starter templates, deterministic XML serializer/parser | `uuid`, `serde`, `quick-xml` |
| `crates/prompt-application` | Command pattern, history stack (undo/redo), section live typing, application service | `prompt-core` |
| `crates/prompt-persistence` | Atomic file persistence (`.prompt.json`), safe reads, text exports, app configuration | `directories`, `serde_json`, `prompt-core` |
| `crates/prompt-refinement` | Refinement modes, changeset validation, unified diff generator, manual provider, OpenAI-compatible HTTP client | `similar`, `reqwest`, `prompt-core` |
| `crates/prompt-tui` | Interactive Terminal UI with multiline editing, modals, and diff review | `ratatui`, `crossterm`, `prompt-application` |
| `crates/prompt-gui` | Native Graphical Workbench with 3-column layout, drag-and-drop cards, and diff dialog | `egui`, `eframe`, `arboard`, `prompt-application` |
| `crates/prompt-cli` | Unified binary command line interface | `clap`, `tokio` |

---

## Installation & Build

### Prerequisites
- Modern Rust toolchain (Rust 1.80+ or 2021 edition recommended).
- Standard C/C++ development tools for native windowing on Linux (e.g. `libxkbcommon-dev`, `libxcb-shape0-dev` if building GUI on Linux).

### Compiling
Clone the repository and build with Cargo:

```bash
git clone https://github.com/your-org/promptforge.git
cd promptforge

# Build workspace in release mode
cargo build --release

# The compiled binary is available at:
./target/release/promptforge
```

---

## Quick Start

Create and inspect an engineering prompt in 10 seconds:

```bash
# 1. Create a document from the built-in engineering template
promptforge new --template engineering --output my_task.prompt.json

# 2. Validate structural integrity
promptforge validate my_task.prompt.json

# 3. Render draft XML with element IDs
promptforge render my_task.prompt.json --stage draft

# 4. Open in the interactive terminal UI
promptforge tui my_task.prompt.json

# 5. Or launch the graphical desktop workbench
promptforge gui my_task.prompt.json
```

---

## Interfaces

PromptForge ships with three first-class native interfaces in a single binary.

### Graphical Interface (GUI)
Launch with:
```bash
promptforge gui [FILE]
```

- **Left Panel (Presets & Templates):** Quick-add buttons for 13 standard prompt sections (`role`, `context`, `task`, `constraints`, `examples`, etc.) and starter templates.
- **Center Canvas (Visual Section Cards):**
  - Drag-and-drop handle (≡) to dynamically reorder sections.
  - Inline tag editor with XML name validation.
  - Multiline brief editor with real-time word counting.
  - Lock toggle (🔒) to protect sections against deletion or automated refinement.
  - Enable toggle (👁) to exclude sections from compilation without deleting them.
  - Duplicate (📋) and Remove (🗑) buttons.
- **Right Panel (Live XML Compilation):** Real-time, deterministic XML preview. Toggle between **Draft** (original briefs) and **Final** (accepted refined text). Includes a **Copy to Clipboard** button.
- **Top Bar:** File open/save/export, undo/redo (`Ctrl+Z`, `Ctrl+Y`), and the **Refine Prompt** modal.

### Terminal Interface (TUI)
Launch with:
```bash
promptforge tui [FILE]
```

A clean, full-screen Ratatui terminal dashboard featuring:
- **Left Panel:** Reorderable section list with status badges (`[L]` locked, `[R]` refined, `[D]` disabled).
- **Right Panel:** Tabbed preview showing the Section Brief Editor, Diff View, or compiled XML output.
- **Keybindings:**
  - `j` / `k` or `Down` / `Up`: Navigate section list.
  - `J` / `K`: Move section down / up.
  - `Enter` / `e`: Edit selected section in multiline buffer.
  - `a`: Add new section modal.
  - `r`: Rename section XML tag.
  - `d`: Duplicate section.
  - `x`: Delete section (locked sections are protected).
  - `l`: Toggle section lock.
  - `Space`: Toggle section enabled/disabled.
  - `Tab`: Switch preview tabs (Brief / Diff / XML).
  - `Ctrl+Z` / `Ctrl+Y`: Undo / Redo.
  - `s` / `o`: Save document / Open document.
  - `m`: Refine prompt (Manual or OpenAI-compatible).
  - `q`: Quit.

### Headless CLI
The `promptforge` CLI is designed for shell pipelines, Git pre-commit hooks, and CI/CD validation.

```bash
# Create a new document
promptforge new --template [general|engineering|research] --output <PATH>

# Validate a document
promptforge validate <PATH>

# Render XML (Draft vs Final, with or without section IDs)
promptforge render <PATH> --stage draft
promptforge render <PATH> --stage final --clean --output prompt.xml

# Export manual refinement instructions to terminal or clipboard
promptforge refine <PATH> --mode conservative --manual

# Apply manual XML or JSON responses from an external LLM
promptforge refine <PATH> --apply response.json --in-place

# Automated refinement using OpenAI-compatible endpoint
promptforge refine <PATH> --provider openai-compatible --model gpt-4o

# Install the coding agent skill
promptforge skill install --target ~/.config/claude/skills
```

---

## Refinement Workflow

PromptForge introduces a disciplined refinement loop that prevents the model from wandering or overwriting user constraints.

### The 3 Refinement Modes

1. **Conservative:**
   - Improves clarity, precision, and phrasing.
   - Preserves all requirements, technologies, and constraints exactly.
   - Prohibits introducing unsolicited dependencies or architectural assumptions.
2. **Expand:**
   - Elaborates on vague instructions with justified details.
   - Identifies implicit requirements and edge cases.
   - Suggests missing sections (such as error handling, testing boundaries, or non-goals).
3. **Critique:**
   - Evaluates the prompt against clarity, completeness, constraint tightness, and ambiguity.
   - Returns a structured diagnostic report without modifying existing section text.

### Manual Refinement (Zero API Keys)

For users who prefer using web interfaces (ChatGPT Plus, Claude Pro) or air-gapped environments:

1. In GUI, TUI, or CLI, trigger **Manual Refinement**:
   ```bash
   promptforge refine my_prompt.prompt.json --manual > request.txt
   ```
2. PromptForge exports a formatted prompt containing the document XML and instructions for the external LLM.
3. Paste the request into ChatGPT or Claude. The model returns a structured JSON or XML snippet referencing sections by their stable UUIDs.
4. Import the response back:
   - In GUI: Click **Import Refinement**, paste the response, and inspect the side-by-side diff.
   - In TUI: Press `m`, select **Import Response**, and accept/reject diffs per section.
   - In CLI: Run `promptforge refine my_prompt.prompt.json --apply response.json --in-place`.

### Automated Provider (OpenAI-Compatible)

Configure your endpoint in `~/.config/promptforge/config.toml`:

```toml
active_provider = "openai_compatible"
theme = "dark"

[openai_compatible]
base_url = "https://api.openai.com/v1"
model = "gpt-4o"
api_key_env_var = "OPENAI_API_KEY"
timeout_seconds = 60
```

When triggered, PromptForge dispatches the prompt, validates that the returned changeset respects all locked sections, and loads the diffs into the review buffer.

---

## Agent Skill Integration

PromptForge implements the [Agent Skills standard](skills/prompt-refinement/SKILL.md), allowing autonomous coding assistants (such as Claude Code, Cursor, and Antigravity) to act as specialized prompt engineers.

### Installing the Skill
```bash
# Install to Claude or custom agent directory
promptforge skill install --target ~/.config/claude/skills
```

Once installed, coding agents automatically detect the skill and can create, inspect, validate, and refine prompts programmatically.

---

## File Format Specification

PromptForge stores prompt documents in a clean, versioned JSON format (`.prompt.json`):

```json
{
  "schema_version": 1,
  "id": "c4afc344-05cf-4161-968f-61e2ff1bad0e",
  "title": "Software Engineering Task",
  "description": "Specialized prompt structure for software architecture tasks.",
  "sections": [
    {
      "id": "efb3b79f-c60e-46eb-bb22-a810c69e94e3",
      "tag": "role",
      "brief": "Senior Systems Architect.",
      "refined": "Principal Systems Architect specializing in low-latency distributed systems.",
      "locked": false,
      "enabled": true
    },
    {
      "id": "8d95089f-6eb3-492a-a0d3-cfa91f904ff3",
      "tag": "constraints",
      "brief": "- Zero allocation on hot paths\n- Strict backward compatibility",
      "locked": true,
      "enabled": true
    }
  ]
}
```

### Compiled XML Output
When compiled with `--stage final --clean`, PromptForge produces deterministic, well-formed XML:

```xml
<prompt>
  <role>
    Principal Systems Architect specializing in low-latency distributed systems.
  </role>
  <constraints>
    - Zero allocation on hot paths
    - Strict backward compatibility
  </constraints>
</prompt>
```

---

## Verification & Testing

PromptForge maintains an exhaustive automated test suite covering all domain logic, history transitions, XML roundtripping, refinement changeset validation, diff generation, CLI operations, and error recovery:

```bash
# Run all tests across the workspace
cargo test --workspace

# Run integration tests specifically
cargo test --test integration_tests
```

All 47 unit and integration tests execute in `< 0.05s` with 0 failures and 0 warnings.

---

## License

PromptForge is dual-licensed under either:
- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE))
- MIT license ([LICENSE-MIT](LICENSE-MIT))

at your option.
