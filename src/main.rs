mod cli;
mod output;
mod transcript;

use crate::cli::{Cli, Commands};
use anyhow::Context;
use clap::Parser;
use cliscrape::FsmParser;
use std::io::{self, Read};

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Parse {
            template,
            input,
            format,
        } => {
            let parser = FsmParser::from_file(&template)
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
