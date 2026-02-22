use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cliscrape")]
#[command(about = "High-performance CLI scraping and parsing tool", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Error output format
    #[arg(long, value_enum, default_value_t = ErrorFormat::Human, global = true)]
    pub error_format: ErrorFormat,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Parse a raw text file using a template
    Parse {
        /// Template spec (path or identifier)
        #[arg(short, long, value_name = "TEMPLATE")]
        template: String,

        /// Override template format selection (default: auto from extension)
        #[arg(long, value_enum, default_value_t = TemplateFormat::Auto)]
        template_format: TemplateFormat,

        /// Input paths (0+)
        #[arg(value_name = "INPUTS", num_args = 0..)]
        inputs: Vec<PathBuf>,

        /// Add an input path (repeatable)
        #[arg(long, value_name = "PATH")]
        input: Vec<PathBuf>,

        /// Add an input glob pattern (repeatable; expanded by app code)
        #[arg(long, value_name = "PATTERN")]
        input_glob: Vec<String>,

        /// Include stdin as an input source (in addition to file inputs)
        #[arg(long)]
        stdin: bool,

        /// Output format
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Auto)]
        format: OutputFormat,

        /// Suppress the success status line (warnings still print)
        #[arg(long)]
        quiet: bool,
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

    /// List available templates
    #[command(name = "list-templates", alias = "templates")]
    ListTemplates {
        /// Filter templates by pattern (supports wildcards)
        #[arg(short, long)]
        filter: Option<String>,

        /// Output format for template listing
        #[arg(short = 'f', long, value_enum, default_value_t = OutputFormat::Table)]
        format: OutputFormat,
    },

    /// Show detailed information about a specific template
    #[command(name = "show-template")]
    ShowTemplate {
        /// Template name to display
        template: String,

        /// Show template source code
        #[arg(long)]
        source: bool,
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
    /// Auto-select output format
    Auto,
    /// JSON output
    Json,
    /// CSV output (placeholder)
    Csv,
    /// Table output (placeholder)
    Table,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ErrorFormat {
    /// Human-readable error messages
    Human,
    /// Machine-readable JSON error objects
    Json,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum ConvertFormat {
    /// Modern YAML template
    Yaml,
    /// Modern TOML template
    Toml,
}
