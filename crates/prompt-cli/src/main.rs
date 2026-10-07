use clap::{Parser, Subcommand};
use prompt_core::{template::StarterTemplate, RenderOptions, RenderStage};
use prompt_persistence::{export_text, load_document, save_document, AppConfig};
use prompt_refinement::{
    generate_manual_request, OpenAiCompatibleProvider, RefinementEngine, RefinementMode,
};
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

const SKILL_MD_CONTENT: &str = include_str!("../../../skills/prompt-refinement/SKILL.md");

#[derive(Parser)]
#[command(
    name = "promptforge",
    author = "PromptForge Contributors",
    version = "0.1.0",
    about = "PromptForge — A Native Rust Prompt Engineering Workbench",
    long_about = "A local-first, native Rust application for building, refining, and reusing high-quality structured LLM prompts."
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the native desktop graphical workbench (Tauri 2)
    Gui {
        /// Optional prompt document file to open (.prompt.json)
        file: Option<PathBuf>,
    },

    /// Launch the interactive terminal user interface (Ratatui)
    Tui {
        /// Optional prompt document file to open (.prompt.json)
        file: Option<PathBuf>,
    },

    /// Create a new prompt document from a starter template
    New {
        /// Template to instantiate: 'general', 'engineering', or 'research'
        #[arg(short, long, default_value = "engineering")]
        template: String,

        /// Title of the new prompt document
        #[arg(long)]
        title: Option<String>,

        /// Output file path (.prompt.json)
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Validate a prompt document file for structural and XML tag integrity
    Validate {
        /// Path to the .prompt.json document to validate
        file: PathBuf,
    },

    /// Render a prompt document into XML
    Render {
        /// Path to the .prompt.json document
        file: PathBuf,

        /// Output format (default: xml)
        #[arg(short, long, default_value = "xml")]
        format: String,

        /// Render stage: 'draft' (original briefs) or 'final' (accepted refined text)
        #[arg(short, long, default_value = "draft")]
        stage: String,

        /// Clean export: omit internal section UUID attributes from XML
        #[arg(long)]
        clean: bool,

        /// Write rendered output to a file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Refine a prompt document using Manual mode or an AI provider
    Refine {
        /// Path to the .prompt.json document
        file: PathBuf,

        /// Refinement mode: 'conservative', 'expand', or 'critique'
        #[arg(short, long, default_value = "conservative")]
        mode: String,

        /// Output manual refinement instructions suitable for copy/pasting
        #[arg(long)]
        manual: bool,

        /// Provider to use: 'manual' or 'openai-compatible'
        #[arg(short, long, default_value = "manual")]
        provider: String,

        /// Model name for provider (e.g. gpt-4o)
        #[arg(long)]
        model: Option<String>,

        /// Base URL for OpenAI-compatible endpoint
        #[arg(long)]
        base_url: Option<String>,

        /// Environment variable containing the API key
        #[arg(long)]
        api_key_env: Option<String>,

        /// Apply an external response file (XML or JSON) instead of requesting refinement
        #[arg(long)]
        apply: Option<PathBuf>,

        /// In-place update of the prompt document when applying changes
        #[arg(short, long)]
        in_place: bool,

        /// Write refined output or changeset to a file instead of stdout
        #[arg(short, long)]
        output: Option<PathBuf>,
    },

    /// Manage Agent Skills for PromptForge
    Skill {
        #[command(subcommand)]
        action: SkillAction,
    },
}

#[derive(Subcommand)]
enum SkillAction {
    /// Install the PromptForge refinement skill into an Agent Skills directory
    Install {
        /// Destination directory (e.g. ~/.config/claude/skills or .agents/skills)
        #[arg(short, long)]
        target: Option<PathBuf>,

        /// Overwrite destination file if it already exists
        #[arg(short, long)]
        force: bool,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        None if std::env::var("DISPLAY").is_ok() || std::env::var("WAYLAND_DISPLAY").is_ok() => {
            if let Err(e) = prompt_gui::run_gui(None) {
                eprintln!("Error launching GUI: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Gui { file }) => {
            if let Err(e) = prompt_gui::run_gui(file) {
                eprintln!("Error launching GUI: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        _ => {
            tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .expect("Failed to initialize async runtime")
                .block_on(run_cli(cli))
        }
    }
}

async fn run_cli(cli: Cli) -> ExitCode {
    match cli.command {
        None => {
            if let Err(e) = prompt_tui::run_tui(None).await {
                eprintln!("Error launching TUI: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Some(Commands::Gui { .. }) => unreachable!(),
        Some(Commands::Tui { file }) => {
            if let Err(e) = prompt_tui::run_tui(file).await {
                eprintln!("Error launching TUI: {e}");
                return ExitCode::FAILURE;
            }
            ExitCode::SUCCESS
        }
        Some(Commands::New {
            template,
            title,
            output,
        }) => {
            let tmpl = match StarterTemplate::find(&template) {
                Some(t) => t,
                None => {
                    eprintln!(
                        "Error: Unknown template '{template}'. Available templates: general, engineering, research"
                    );
                    return ExitCode::FAILURE;
                }
            };

            let doc = tmpl.instantiate(title.as_deref());
            let out_path = output.unwrap_or_else(|| {
                PathBuf::from(format!(
                    "{}.prompt.json",
                    doc.title.to_lowercase().replace(' ', "_")
                ))
            });

            if let Err(e) = save_document(&out_path, &doc) {
                eprintln!("Error saving new document to '{}': {e}", out_path.display());
                return ExitCode::FAILURE;
            }

            println!("Created new prompt document at '{}'", out_path.display());
            ExitCode::SUCCESS
        }
        Some(Commands::Validate { file }) => {
            match load_document(&file) {
                Ok(doc) => {
                    println!(
                        "✔ Document '{}' is valid (schema v{}, {} sections)",
                        file.display(),
                        doc.schema_version,
                        doc.sections.len()
                    );
                    ExitCode::SUCCESS
                }
                Err(e) => {
                    eprintln!("✘ Validation failed for '{}': {e}", file.display());
                    ExitCode::FAILURE
                }
            }
        }
        Some(Commands::Render {
            file,
            format,
            stage,
            clean,
            output,
        }) => {
            if !format.eq_ignore_ascii_case("xml") {
                eprintln!("Error: Unsupported format '{format}'. Supported formats: xml");
                return ExitCode::FAILURE;
            }

            let doc = match load_document(&file) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error loading document '{}': {e}", file.display());
                    return ExitCode::FAILURE;
                }
            };

            let render_stage = match stage.to_lowercase().as_str() {
                "draft" => RenderStage::Draft,
                "final" => RenderStage::Final,
                other => {
                    eprintln!("Error: Unknown stage '{other}'. Use 'draft' or 'final'");
                    return ExitCode::FAILURE;
                }
            };

            let options = RenderOptions {
                stage: render_stage,
                include_ids: !clean,
                pretty: true,
            };

            let xml = match prompt_core::render_xml(&doc, options) {
                Ok(x) => x,
                Err(e) => {
                    eprintln!("Error rendering XML: {e}");
                    return ExitCode::FAILURE;
                }
            };

            if let Some(out_path) = output {
                if let Err(e) = export_text(&out_path, &xml) {
                    eprintln!("Error writing output to '{}': {e}", out_path.display());
                    return ExitCode::FAILURE;
                }
            } else {
                print!("{xml}");
            }

            ExitCode::SUCCESS
        }
        Some(Commands::Refine {
            file,
            mode,
            manual,
            provider,
            model,
            base_url,
            api_key_env,
            apply,
            in_place,
            output,
        }) => {
            let doc = match load_document(&file) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("Error loading document '{}': {e}", file.display());
                    return ExitCode::FAILURE;
                }
            };

            if let Some(apply_path) = apply {
                let raw_text = match std::fs::read_to_string(&apply_path) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Error reading response file '{}': {e}", apply_path.display());
                        return ExitCode::FAILURE;
                    }
                };

                let changeset = match prompt_refinement::parse_manual_response(&raw_text, &doc) {
                    Ok(c) => c,
                    Err(e) => {
                        eprintln!("Error parsing refinement response: {e}");
                        return ExitCode::FAILURE;
                    }
                };

                if let Err(e) = changeset.validate_against_document(&doc) {
                    eprintln!("Error validating changeset against document: {e}");
                    return ExitCode::FAILURE;
                }

                println!("✔ Validated refinement changeset with {} section change(s)", changeset.changes.len());
                for change in &changeset.changes {
                    if let Some(sec) = doc.section(change.section_id) {
                        println!("\n--- Section: <{}> (id: {}) ---", sec.tag, sec.id);
                        if !change.reason.is_empty() {
                            println!("Rationale: {}", change.reason);
                        }
                        let diff = prompt_refinement::compute_line_diff(&sec.brief, &change.refined);
                        for line in diff {
                            match line.tag {
                                prompt_refinement::DiffTag::Equal => println!("  {}", line.text),
                                prompt_refinement::DiffTag::Delete => println!("- {}", line.text),
                                prompt_refinement::DiffTag::Insert => println!("+ {}", line.text),
                            }
                        }
                    }
                }

                if in_place || output.is_some() {
                    let mut updated_doc = doc.clone();
                    for change in &changeset.changes {
                        if let Some(sec) = updated_doc.section_mut(change.section_id) {
                            sec.refined = Some(change.refined.clone());
                        }
                    }
                    let target_path = if let Some(out_p) = output { out_p } else { file };
                    if let Err(e) = save_document(&target_path, &updated_doc) {
                        eprintln!("Error saving updated document: {e}");
                        return ExitCode::FAILURE;
                    }
                    println!("\n✔ Applied refinements to '{}'", target_path.display());
                } else {
                    println!("\n(Dry run: pass --in-place or -o <FILE> to save refinements to document)");
                }

                return ExitCode::SUCCESS;
            }

            let refinement_mode = match mode.to_lowercase().as_str() {
                "conservative" => RefinementMode::Conservative,
                "expand" => RefinementMode::Expand,
                "critique" => RefinementMode::Critique,
                other => {
                    eprintln!("Error: Unknown mode '{other}'. Supported: conservative, expand, critique");
                    return ExitCode::FAILURE;
                }
            };

            let is_manual = manual || provider.eq_ignore_ascii_case("manual");

            if is_manual {
                let req_text = match generate_manual_request(&doc, refinement_mode) {
                    Ok(t) => t,
                    Err(e) => {
                        eprintln!("Error generating manual request: {e}");
                        return ExitCode::FAILURE;
                    }
                };

                if let Some(out_path) = output {
                    if let Err(e) = export_text(&out_path, &req_text) {
                        eprintln!("Error saving manual request: {e}");
                        return ExitCode::FAILURE;
                    }
                    println!("Exported manual refinement request to '{}'", out_path.display());
                } else {
                    print!("{req_text}");
                }

                ExitCode::SUCCESS
            } else if provider.eq_ignore_ascii_case("openai-compatible") {
                let config = AppConfig::load_or_default();
                let b_url = base_url.unwrap_or(config.openai_compatible.base_url);
                let m_name = model.unwrap_or(config.openai_compatible.model);
                let k_env = api_key_env.unwrap_or(config.openai_compatible.api_key_env_var);

                let prov = OpenAiCompatibleProvider::new(b_url, m_name, k_env, 60);
                let engine = RefinementEngine::new();

                eprintln!("Requesting refinement from OpenAI-compatible endpoint ({mode})...");
                match engine.execute(&prov, &doc, refinement_mode).await {
                    Ok(changeset) => {
                        let json = match serde_json::to_string_pretty(&changeset) {
                            Ok(j) => j,
                            Err(e) => {
                                eprintln!("Error serializing changeset: {e}");
                                return ExitCode::FAILURE;
                            }
                        };

                        if let Some(out_path) = output {
                            if let Err(e) = export_text(&out_path, &json) {
                                eprintln!("Error saving changeset to '{}': {e}", out_path.display());
                                return ExitCode::FAILURE;
                            }
                            println!("Saved refinement changeset to '{}'", out_path.display());
                        } else {
                            println!("{json}");
                        }

                        ExitCode::SUCCESS
                    }
                    Err(e) => {
                        eprintln!("Refinement error: {e}");
                        ExitCode::FAILURE
                    }
                }
            } else {
                eprintln!("Error: Unsupported provider '{provider}'. Use 'manual' or 'openai-compatible'");
                ExitCode::FAILURE
            }
        }
        Some(Commands::Skill {
            action: SkillAction::Install { target, force },
        }) => {
            let dest_dir = target.unwrap_or_else(|| {
                // Default to ~/.config/claude/skills or ./skills
                if let Some(home) = std::env::var_os("HOME") {
                    PathBuf::from(home)
                        .join(".config")
                        .join("claude")
                        .join("skills")
                } else {
                    PathBuf::from("skills")
                }
            });

            let skill_dir = if dest_dir.ends_with("prompt-refinement") {
                dest_dir
            } else {
                dest_dir.join("prompt-refinement")
            };

            let skill_file = skill_dir.join("SKILL.md");

            if skill_file.exists() && !force {
                eprintln!(
                    "Error: Skill already exists at '{}'. Use --force to overwrite.",
                    skill_file.display()
                );
                return ExitCode::FAILURE;
            }

            if let Err(e) = fs::create_dir_all(&skill_dir) {
                eprintln!("Error creating directory '{}': {e}", skill_dir.display());
                return ExitCode::FAILURE;
            }

            if let Err(e) = fs::write(&skill_file, SKILL_MD_CONTENT) {
                eprintln!("Error writing skill to '{}': {e}", skill_file.display());
                return ExitCode::FAILURE;
            }

            println!(
                "Successfully installed PromptForge refinement skill to '{}'",
                skill_file.display()
            );
            ExitCode::SUCCESS
        }
    }
}
