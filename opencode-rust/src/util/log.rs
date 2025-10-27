use std::fmt;

use anyhow::Result;
use tracing_subscriber::{
    FmtSubscriber,
    filter::{EnvFilter, LevelFilter},
    fmt::writer::BoxMakeWriter,
};

pub struct LogConfig {
    pub level: LevelFilter,
    pub print_logs: bool,
}

impl LogConfig {
    pub fn new(level: LevelFilter, print_logs: bool) -> Self {
        Self { level, print_logs }
    }
}

impl fmt::Debug for LogConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("LogConfig")
            .field("level", &self.level)
            .field("print_logs", &self.print_logs)
            .finish()
    }
}

pub fn init(config: LogConfig) -> Result<()> {
    let level_str = match config.level {
        LevelFilter::OFF => "off",
        LevelFilter::ERROR => "error",
        LevelFilter::WARN => "warn",
        LevelFilter::INFO => "info",
        LevelFilter::DEBUG => "debug",
        LevelFilter::TRACE => "trace",
    };

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(level_str));

    let base = FmtSubscriber::builder()
        .with_max_level(config.level)
        .with_env_filter(filter);

    let make_writer: BoxMakeWriter = match config.print_logs {
        true => BoxMakeWriter::new(std::io::stderr),
        false => BoxMakeWriter::new(|| std::io::sink()),
    };

    let subscriber = base.with_writer(make_writer).finish();

    tracing::subscriber::set_global_default(subscriber)?;
    Ok(())
}
