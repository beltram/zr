use std::{env, str::FromStr};

use env_logger::Builder;
use log::LevelFilter;

use crate::cli::Log;

/// Sets log level for current process
pub struct CliLog;

impl CliLog {
    // sets default log level
    pub const LOG_LEVEL: &'static str = "LOG_LEVEL";
    // when "1", captures backtraces at runtime
    const RUST_LIB_BACKTRACE: &'static str = "RUST_LIB_BACKTRACE";
    // same as 'RUST_LIB_BACKTRACE' but with lower precedence
    const RUST_BACKTRACE: &'static str = "RUST_BACKTRACE";
    // allows setting log level globally for user
    const RUST_LOG: &'static str = "RUST_LOG";

    /// Inits log level globally
    /// If debug we set RUST_BACKTRACE for current process to get errors stacktrace
    /// Cli's log level can also be configured globally with RUST_LOG env var
    pub fn init(log: &Log) {
        let log_level = Self::match_log_level(log);
        env::set_var(Self::LOG_LEVEL, format!("{}", log_level));
        let is_log_level_debug = log_level == LevelFilter::Debug;
        if !is_log_level_debug { Self::backtraces(false); }
        Builder::from_default_env()
            .format_timestamp(None)
            .filter_level(log_level)
            .format_level(is_log_level_debug)
            .format_module_path(is_log_level_debug)
            .init()
    }

    fn match_log_level(log: &Log) -> LevelFilter {
        match log {
            ref l if l.debug => Self::if_log_level_debug(),
            ref l if l.info => LevelFilter::Info,
            ref l if l.warning => LevelFilter::Warn,
            ref l if l.error => LevelFilter::Error,
            _ => Self::default_log_level(),
        }
    }

    fn if_log_level_debug() -> LevelFilter {
        Self::backtraces(true);
        LevelFilter::Debug
    }

    fn backtraces(enabled: bool) {
        let value = if enabled { "1" } else { "0" };
        env::set_var(Self::RUST_BACKTRACE, value);
        env::set_var(Self::RUST_LIB_BACKTRACE, value);
    }

    fn default_log_level() -> LevelFilter {
        env::var(Self::RUST_LOG).ok()
            .and_then(|it| LevelFilter::from_str(&it).ok())
            .unwrap_or(LevelFilter::Info)
    }
}