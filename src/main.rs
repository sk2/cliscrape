mod cli;
mod output;
mod transcript;
mod tui;

use crate::cli::{Cli, Commands, ErrorFormat, TemplateFormat as CliTemplateFormat};
use anyhow::Context;
use clap::Parser;
use cliscrape::FsmParser;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::collections::HashSet;
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};

fn main() {
    // Pre-scan argv for --error-format to honor it even during clap parsing failures
    let error_format = detect_error_format_from_argv();

    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(e) => {
            use clap::error::ErrorKind;
            match e.kind() {
                ErrorKind::DisplayHelp | ErrorKind::DisplayVersion => {
                    // Help/version are success cases - print to stdout and exit 0
                    print!("{}", e);
                    std::process::exit(0);
                }
                _ => {
                    // Real errors - format according to --error-format and exit 1
                    print_error(&e.to_string(), error_format);
                    std::process::exit(1);
                }
            }
        }
    };

    let error_format = cli.error_format;
    if let Err(e) = run_command(cli) {
        print_error(&format!("{:#}", e), error_format);
        std::process::exit(1);
    }
}

fn detect_error_format_from_argv() -> ErrorFormat {
    let args: Vec<_> = std::env::args_os().collect();
    for (i, arg) in args.iter().enumerate() {
        if let Some(arg_str) = arg.to_str() {
            // Handle --error-format=json
            if arg_str.starts_with("--error-format=") {
                if arg_str == "--error-format=json" {
                    return ErrorFormat::Json;
                }
            }
            // Handle --error-format json
            else if arg_str == "--error-format" {
                if let Some(next_arg) = args.get(i + 1).and_then(|a| a.to_str()) {
                    if next_arg == "json" {
                        return ErrorFormat::Json;
                    }
                }
            }
        }
    }
    ErrorFormat::Human
}

fn print_error(message: &str, format: ErrorFormat) {
    match format {
        ErrorFormat::Human => {
            eprintln!("Error: {}", message);
        }
        ErrorFormat::Json => {
            let error_obj = serde_json::json!({
                "ok": false,
                "error": message,
            });
            if let Err(e) = writeln!(io::stderr(), "{}", error_obj) {
                // Fallback if JSON serialization somehow fails
                eprintln!("Error: {}", message);
                eprintln!("(JSON serialization failed: {})", e);
            }
        }
    }
}

fn run_command(cli: Cli) -> anyhow::Result<()> {
    match cli.command {
        Commands::Parse {
            template,
            template_format,
            inputs,
            input,
            input_glob,
            stdin,
            format,
            quiet: _,
        } => {
            // Template resolution: path vs identifier
            let template_path = resolve_template_spec(&template, template_format)?;

            let (parser, warnings) = match template_format {
                CliTemplateFormat::Auto => {
                    FsmParser::from_file_with_warnings(&template_path)
                        .with_context(|| format!("Failed to load template from {}", template_path.display()))?
                }
                CliTemplateFormat::Textfsm => {
                    let p = FsmParser::from_file_with_format(
                        &template_path,
                        cliscrape::TemplateFormat::Textfsm,
                    )
                    .with_context(|| format!("Failed to load template from {}", template_path.display()))?;
                    (p, Vec::new())
                }
                CliTemplateFormat::Yaml => {
                    let p = FsmParser::from_file_with_format(
                        &template_path,
                        cliscrape::TemplateFormat::Yaml,
                    )
                    .with_context(|| format!("Failed to load template from {}", template_path.display()))?;
                    (p, Vec::new())
                }
                CliTemplateFormat::Toml => {
                    let p = FsmParser::from_file_with_format(
                        &template_path,
                        cliscrape::TemplateFormat::Toml,
                    )
                    .with_context(|| format!("Failed to load template from {}", template_path.display()))?;
                    (p, Vec::new())
                }
            };

            // Print template loader warnings to stderr
            for warning in &warnings {
                eprintln!("Warning ({}): {}", warning.kind, warning.message);
            }

            // Resolve multi-input: files + globs + stdin
            let input_sources = resolve_input_sources(&inputs, &input, &input_glob, stdin)?;

            // Fail-fast parsing: collect all records before writing to stdout
            let mut all_results = Vec::new();
            let mut all_transcript_warnings = Vec::new();

            for source in &input_sources {
                let content = match source {
                    InputSource::Stdin => {
                        let mut buffer = String::new();
                        io::stdin()
                            .read_to_string(&mut buffer)
                            .context("Failed to read input from stdin")?;
                        buffer
                    }
                    InputSource::File(path) => std::fs::read_to_string(path)
                        .with_context(|| format!("Failed to read input from {}", path.display()))?,
                };

                let (blocks, transcript_warnings) =
                    transcript::preprocess_ios_transcript_with_warnings(&content);
                all_transcript_warnings.extend(transcript_warnings);

                for (idx, block) in blocks.iter().enumerate() {
                    let mut parsed = parser.parse(block).with_context(|| {
                        format!(
                            "Failed to parse block {} from {}",
                            idx + 1,
                            source.display()
                        )
                    })?;
                    all_results.append(&mut parsed);
                }
            }

            // Print transcript warnings to stderr
            for warning in all_transcript_warnings {
                eprintln!("Warning: {}", warning);
            }

            let output = output::serialize(&all_results, format)?;
            println!("{}", output);
        }
        Commands::Debug { template, input } => tui::run_debugger(template, input)?,

        Commands::Convert {
            input,
            output,
            format,
            defaults,
        } => {
            let ext = input.extension().and_then(|s| s.to_str());
            let ext_display = ext.unwrap_or("<none>");
            if ext != Some("textfsm") {
                anyhow::bail!(
                    "Unsupported input template extension '{ext_display}'. Supported: .textfsm"
                );
            }

            let input_content = std::fs::read_to_string(&input)
                .with_context(|| format!("Failed to read input template from {:?}", input))?;
            let ir = cliscrape::template::loader::TextFsmLoader::parse_str(&input_content)
                .with_context(|| format!("Failed to parse TextFSM template from {:?}", input))?;

            let theme = ColorfulTheme::default();

            let chosen_format = match format {
                Some(f) => f,
                None if defaults => crate::cli::ConvertFormat::Yaml,
                None => {
                    let idx = Select::with_theme(&theme)
                        .with_prompt("Output format")
                        .items(&["yaml", "toml"])
                        .default(0)
                        .interact()?;
                    match idx {
                        0 => crate::cli::ConvertFormat::Yaml,
                        _ => crate::cli::ConvertFormat::Toml,
                    }
                }
            };

            let default_out = default_output_path(&input, chosen_format);
            let output_provided = output.is_some();
            let out_path = match output {
                Some(p) => p,
                None if defaults => default_out.clone(),
                None => {
                    let entered: String = Input::with_theme(&theme)
                        .with_prompt("Output path")
                        .default(default_out.to_string_lossy().to_string())
                        .interact_text()?;
                    PathBuf::from(entered)
                }
            };

            if out_path.exists() {
                if defaults {
                    anyhow::bail!(
                        "Output file already exists: {} (choose a different --output or omit --defaults)",
                        out_path.display()
                    );
                }

                if output_provided {
                    anyhow::bail!(
                        "Output file already exists: {} (choose a different --output)",
                        out_path.display()
                    );
                }

                let overwrite = Confirm::with_theme(&theme)
                    .with_prompt(format!(
                        "Output file exists. Overwrite {}?",
                        out_path.display()
                    ))
                    .default(false)
                    .interact()?;
                if !overwrite {
                    anyhow::bail!("Refused to overwrite existing file: {}", out_path.display());
                }
            }

            if let Some(parent) = out_path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent).with_context(|| {
                        format!("Failed to create output directory {}", parent.display())
                    })?;
                }
            }

            let doc = cliscrape::template::convert::template_ir_to_modern_doc(&ir);
            let rendered = match chosen_format {
                crate::cli::ConvertFormat::Yaml => {
                    cliscrape::template::modern::to_yaml_string(&doc)
                        .context("Failed to serialize modern template to YAML")?
                }
                crate::cli::ConvertFormat::Toml => {
                    cliscrape::template::modern::to_toml_string(&doc)
                        .context("Failed to serialize modern template to TOML")?
                }
            };

            std::fs::write(&out_path, rendered)
                .with_context(|| format!("Failed to write output to {}", out_path.display()))?;

            // Sanity-check: converted template loads through modern loader selection.
            let _ = FsmParser::from_file(&out_path).with_context(|| {
                format!(
                    "Converted template did not load successfully from {}",
                    out_path.display()
                )
            })?;

            println!("Wrote converted template to {}", out_path.display());
        }
    }

    Ok(())
}

fn default_output_path(input: &Path, format: crate::cli::ConvertFormat) -> PathBuf {
    let mut out = input.to_path_buf();
    match format {
        crate::cli::ConvertFormat::Yaml => {
            out.set_extension("yaml");
        }
        crate::cli::ConvertFormat::Toml => {
            out.set_extension("toml");
        }
    }
    out
}

/// Resolve template spec: if it's a path, use it; otherwise search CWD for identifier
fn resolve_template_spec(
    spec: &str,
    format_filter: CliTemplateFormat,
) -> anyhow::Result<PathBuf> {
    let spec_path = PathBuf::from(spec);

    // If spec points to an existing path, use it directly
    if spec_path.exists() {
        return Ok(spec_path);
    }

    // Otherwise treat as identifier and search CWD
    let extensions = match format_filter {
        CliTemplateFormat::Auto => vec!["textfsm", "yaml", "yml", "toml"],
        CliTemplateFormat::Textfsm => vec!["textfsm"],
        CliTemplateFormat::Yaml => vec!["yaml", "yml"],
        CliTemplateFormat::Toml => vec!["toml"],
    };

    let mut candidates = Vec::new();
    for ext in extensions {
        let candidate = PathBuf::from(format!("{}.{}", spec, ext));
        if candidate.exists() {
            candidates.push(candidate);
        }
    }

    match candidates.len() {
        0 => anyhow::bail!(
            "Template '{}' not found (tried {} and identifier search in CWD)",
            spec,
            spec_path.display()
        ),
        1 => Ok(candidates.into_iter().next().unwrap()),
        _ => {
            let names: Vec<_> = candidates.iter().map(|p| p.display().to_string()).collect();
            anyhow::bail!(
                "Ambiguous template identifier '{}': found multiple matches: {}. Use an explicit path or --template-format to disambiguate.",
                spec,
                names.join(", ")
            )
        }
    }
}

#[derive(Debug, Clone)]
enum InputSource {
    Stdin,
    File(PathBuf),
}

impl InputSource {
    fn display(&self) -> String {
        match self {
            InputSource::Stdin => "<stdin>".to_string(),
            InputSource::File(p) => p.display().to_string(),
        }
    }
}

/// Resolve final input sources: combine positional inputs, --input, --input-glob, and stdin
fn resolve_input_sources(
    positional: &[PathBuf],
    repeatable_inputs: &[PathBuf],
    globs: &[String],
    explicit_stdin: bool,
) -> anyhow::Result<Vec<InputSource>> {
    let mut file_paths = HashSet::new();

    // Add positional inputs
    for p in positional {
        file_paths.insert(p.clone());
    }

    // Add repeatable --input
    for p in repeatable_inputs {
        file_paths.insert(p.clone());
    }

    // Expand --input-glob patterns
    for pattern in globs {
        let matches: Vec<_> = glob::glob(pattern)
            .with_context(|| format!("Invalid glob pattern: {}", pattern))?
            .collect::<Result<Vec<_>, _>>()
            .with_context(|| format!("Failed to expand glob pattern: {}", pattern))?;

        if matches.is_empty() {
            anyhow::bail!("Glob pattern matched no files: {}", pattern);
        }

        for path in matches {
            file_paths.insert(path);
        }
    }

    let mut sources: Vec<InputSource> = file_paths
        .into_iter()
        .map(InputSource::File)
        .collect();

    // Sort file sources deterministically
    sources.sort_by(|a, b| match (a, b) {
        (InputSource::File(p1), InputSource::File(p2)) => p1.cmp(p2),
        _ => std::cmp::Ordering::Equal,
    });

    // Determine if stdin should be included
    let include_stdin = if explicit_stdin {
        true
    } else if sources.is_empty() {
        // No explicit inputs: include stdin if it's not a TTY
        use std::io::IsTerminal;
        !io::stdin().is_terminal()
    } else {
        false
    };

    if include_stdin {
        // Process stdin last
        sources.push(InputSource::Stdin);
    }

    // Error if final input set is empty
    if sources.is_empty() {
        anyhow::bail!("No input sources specified (use files, --stdin, or pipe input)");
    }

    Ok(sources)
}
