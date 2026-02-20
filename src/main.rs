mod cli;
mod output;
mod transcript;
mod tui;

use crate::cli::{Cli, Commands, TemplateFormat as CliTemplateFormat};
use anyhow::Context;
use clap::Parser;
use cliscrape::FsmParser;
use dialoguer::{theme::ColorfulTheme, Confirm, Input, Select};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse {
            template,
            template_format,
            input,
            format,
        } => {
            let parser = match template_format {
                CliTemplateFormat::Auto => FsmParser::from_file(&template),
                CliTemplateFormat::Textfsm => {
                    FsmParser::from_file_with_format(&template, cliscrape::TemplateFormat::Textfsm)
                }
                CliTemplateFormat::Yaml => {
                    FsmParser::from_file_with_format(&template, cliscrape::TemplateFormat::Yaml)
                }
                CliTemplateFormat::Toml => {
                    FsmParser::from_file_with_format(&template, cliscrape::TemplateFormat::Toml)
                }
            }
            .with_context(|| format!("Failed to load template from {:?}", template))?;

            let input_content = match input {
                Some(path) => std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read input from {:?}", path))?,
                None => {
                    let mut buffer = String::new();
                    io::stdin()
                        .read_to_string(&mut buffer)
                        .context("Failed to read input from stdin")?;
                    buffer
                }
            };

            let blocks = transcript::preprocess_ios_transcript(&input_content);
            let mut results = Vec::new();
            for (idx, block) in blocks.iter().enumerate() {
                let mut parsed = parser
                    .parse(block)
                    .with_context(|| format!("Failed to parse input block {}", idx + 1))?;
                results.append(&mut parsed);
            }

            let output = output::serialize(&results, format)?;
            println!("{}", output);
        }
        Commands::Debug { template, input: _ } => {
            if let Some(t) = template {
                println!("Starting TUI debugger with template: {:?}", t);
            } else {
                println!("Starting TUI debugger (no template loaded)");
            }
            // TUI logic will go here
        }

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
