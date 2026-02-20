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
        /// Path to the template file (.textfsm, .yaml/.yml, .toml)
        #[arg(short, long)]
        template: PathBuf,

        /// Override template format selection (default: auto from extension)
        #[arg(long, value_enum, default_value_t = TemplateFormat::Auto)]
        template_format: TemplateFormat,

        /// Path to the raw input file (uses stdin if omitted)
        input: Option<PathBuf>,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Json)]
        format: OutputFormat,
    },
    /// Launch the TUI debugger
    Debug {
        /// Path to the template file to debug (optional)
        #[arg(short = 't', long)]
        template: Option<PathBuf>,

        /// Path to the input transcript/text file (optional)
        #[arg(short = 'i', long)]
        input: Option<PathBuf>,
    },

    /// Convert a legacy TextFSM template into a modern YAML/TOML template
    Convert {
        /// Path to the input template file (.textfsm)
        #[arg(short = 'i', long)]
        input: PathBuf,

        /// Path to write the converted template (if omitted, you'll be prompted)
        #[arg(short, long)]
        output: Option<PathBuf>,

        /// Output format for the converted template (if omitted, you'll be prompted)
        #[arg(long, value_enum)]
        format: Option<ConvertFormat>,

        /// Run without prompts (choose defaults for missing values)
        #[arg(long)]
        defaults: bool,
    },
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum TemplateFormat {
    /// Infer template format from file extension
    Auto,
    /// Legacy TextFSM template
    Textfsm,
    /// Modern YAML template
    Yaml,
    /// Modern TOML template
    Toml,
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

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ConvertFormat {
    /// Modern YAML template
    Yaml,
    /// Modern TOML template
    Toml,
}
