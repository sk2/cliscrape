use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cliscrape")]
#[command(about = "High-performance CLI scraping and parsing tool", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Parse a raw text file using a template
    Parse {
        /// Path to the template file (.textfsm)
        #[arg(short, long)]
        template: PathBuf,

        /// Path to the raw input file (uses stdin if omitted)
        input: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Launch the TUI debugger
    Debug {
        /// Path to the template file to debug (optional)
        #[arg(short, long)]
        template: Option<PathBuf>,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum OutputFormat {
    /// JSON output
    Json,
    /// CSV output (placeholder)
    Csv,
    /// Table output (placeholder)
    Table,
}
