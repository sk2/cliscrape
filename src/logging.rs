use clap::ValueEnum;
use tracing_subscriber::filter::{Directive, EnvFilter, LevelFilter};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, ValueEnum)]
pub enum LogFormat {
    Text,
    Json,
}

pub fn init_logging(verbosity: u8, format: LogFormat) {
    // Locked decision: RUST_LOG overrides -v defaults.
    let rust_log_is_set = std::env::var_os(EnvFilter::DEFAULT_ENV).is_some();

    let filter = if rust_log_is_set {
        EnvFilter::try_from_default_env().unwrap_or_else(|err| {
            // Subscriber not installed yet, so we can't emit a tracing event here.
            eprintln!(
                "Warning: invalid {} value (falling back to 'warn'): {}",
                EnvFilter::DEFAULT_ENV,
                err
            );
            EnvFilter::builder()
                .with_default_directive(LevelFilter::WARN.into())
                .parse_lossy("")
        })
    } else {
        let directive = default_directive_from_verbosity(verbosity);
        EnvFilter::builder()
            .with_default_directive(directive)
            .parse_lossy("")
    };

    let base = tracing_subscriber::fmt()
        .with_env_filter(filter)
        // Never write logs to stdout; keep command output safe for pipelines.
        .with_writer(std::io::stderr);

    if format == LogFormat::Json {
        let _ = base
            .json()
            .flatten_event(true)
            .with_current_span(true)
            .with_span_list(true)
            .try_init();
    } else {
        let _ = base.try_init();
    }
}

fn default_directive_from_verbosity(verbosity: u8) -> Directive {
    match verbosity {
        0 => LevelFilter::WARN.into(),
        1 => "cliscrape=info"
            .parse()
            .unwrap_or_else(|_| LevelFilter::INFO.into()),
        2 => "cliscrape=debug"
            .parse()
            .unwrap_or_else(|_| LevelFilter::DEBUG.into()),
        3 => "cliscrape=trace"
            .parse()
            .unwrap_or_else(|_| LevelFilter::TRACE.into()),
        _ => LevelFilter::TRACE.into(),
    }
}
