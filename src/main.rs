use clap::{Parser, Subcommand};
use cliscrape::FsmParser;
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cliscrape")]
#[command(about = "High-performance CLI scraping and parsing tool", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Parse a raw text file using a template
    Parse {
        /// Path to the template file (TextFSM or YAML/TOML)
        #[arg(short, long)]
        template: PathBuf,

        /// Path to the raw input file
        input: PathBuf,
    },
    /// Launch the TUI debugger
    Debug {
        /// Path to the template file to debug (optional)
        #[arg(short, long)]
        template: Option<PathBuf>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Parse { template, input } => {
            println!("Loading template from: {:?}", template);
            let parser = FsmParser::from_file(template)?;
            
            println!("Reading input from: {:?}", input);
            let input_content = std::fs::read_to_string(input)?;
            
            println!("Parsing...");
            let _results = parser.parse(&input_content)?;
            println!("Done! (Output logic not implemented yet)");
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
