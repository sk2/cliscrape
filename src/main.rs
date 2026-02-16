mod cli;

use anyhow::Context;
use clap::Parser;
use cliscrape::FsmParser;
use std::io::{self, Read};
use crate::cli::{Cli, Commands};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse { template, input, format } => {
            let parser = FsmParser::from_file(&template)
                .with_context(|| format!("Failed to load template from {:?}", template))?;
            
            let input_content = match input {
                Some(path) => std::fs::read_to_string(&path)
                    .with_context(|| format!("Failed to read input from {:?}", path))?,
                None => {
                    let mut buffer = String::new();
                    io::stdin().read_to_string(&mut buffer)
                        .context("Failed to read input from stdin")?;
                    buffer
                }
            };
            
            let results = parser.parse(&input_content)
                .context("Failed to parse input")?;

            match format {
                cli::OutputFormat::Json => {
                    let json = serde_json::to_string_pretty(&results)
                        .context("Failed to serialize results to JSON")?;
                    println!("{}", json);
                }
                cli::OutputFormat::Csv => {
                    anyhow::bail!("CSV format not yet implemented");
                }
                cli::OutputFormat::Table => {
                    anyhow::bail!("Table format not yet implemented");
                }
            }
        }
        Commands::Debug { template } => {
            if let Some(t) = template {
                println!("Starting TUI debugger with template: {:?}", t);
            } else {
                println!("Starting TUI debugger (no template loaded)");
            }
            // TUI logic will go here
        }
    }

    Ok(())
}
