use crate::logging::LogFormat;
use clap::{Parser, Subcommand, ValueEnum};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "cliscrape")]
#[command(about = "High-performance CLI scraping and parsing tool", long_about = None)]
#[command(version)]
pub struct Cli {
    /// Increase logging verbosity (-v info, -vv debug, -vvv trace, -vvvv trace all targets)
    #[arg(short, long, action = clap::ArgAction::Count, global = true)]
    pub verbose: u8,

    /// Log output format (stderr)
    #[arg(long, value_enum, default_value_t = LogFormat::Text, global = true)]
    pub log_format: LogFormat,

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

        /// Abort on first parsing error or if match threshold is not met
        #[arg(long)]
        strict: bool,

        /// Minimum required field capture percentage (default: 80.0)
        #[arg(long, default_value_t = 80.0)]
        threshold: f64,

        /// Maximum parsing time in milliseconds
        #[arg(long)]
        timeout: Option<u64>,

        /// Use common schema mapping for keys
        #[arg(long)]
        common: bool,

        /// Verify template integrity via round-trip (Parse -> Generate)
        #[arg(long)]
        verify: bool,
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
        #[arg(long)]
        filter: Option<String>,

        /// Output format for template listing
        #[arg(short, long, value_enum, default_value_t = OutputFormat::Table)]
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

    /// Generate synthetic CLI output from JSON records using a template
    Generate {
        /// Template spec (path or identifier)
        #[arg(short, long, value_name = "TEMPLATE")]
        template: String,

        /// Path to JSON file containing records (if omitted, reads from stdin)
        #[arg(short, long, value_name = "JSON")]
        input: Option<PathBuf>,
    },

    /// Compare operational state between two CLI outputs
    Diff {
        /// "Before" CLI output file
        before: PathBuf,

        /// "After" CLI output file
        after: PathBuf,

        /// Template spec (path or identifier)
        #[arg(short, long, value_name = "TEMPLATE")]
        template: String,
    },

    /// Infer a candidate template from raw CLI samples
    Infer {
        /// Input sample files
        #[arg(value_name = "SAMPLES", required = true)]
        samples: Vec<PathBuf>,
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
