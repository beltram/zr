use clap::Clap;

use crate::{completion::model::Shell};
use crate::cmd::InitializrLang;

/// initializr with completion
#[derive(Clap, Debug)]
#[clap(
name = "zr",
bin_name = "zr",
about,
version,
rename_all = "kebab-case",
)]
pub struct Cli {
    #[clap(subcommand)]
    pub cmd: Option<SubCommand>,
    #[clap(flatten)]
    pub log: Log,
}

#[derive(Clap, Debug)]
#[clap(rename_all = "kebab-case")]
pub enum SubCommand {
    /// Updates cli itself
    ///
    /// and all its remote configuration, remote initializr templates.
    Upgrade,
    /// Generates & installs completion scripts
    ///
    /// By default, zr will try to infer your current shell and generate a completion file
    /// for it. If it cannot supply the shell you want a completion file to be generated for
    /// e.g. `zr completion bash`
    Completion {
        #[clap(subcommand)]
        shell: Option<Shell>,
    },
    New {
        #[clap(flatten)]
        lang: InitializrLang,
    },
    /// Prints zr configuration file location
    ///
    /// Use it like `code $(zr get-config)`
    GetConfig,
}

#[derive(Clap, Debug)]
#[clap(rename_all = "kebab-case")]
pub struct Log {
    /// Debug log level (with backtraces)
    #[clap(short, long)]
    pub debug: bool,
    /// Info log level.
    #[clap(short, long)]
    pub info: bool,
    /// Warn log level.
    #[clap(short, long)]
    pub warning: bool,
    /// Error log level.
    #[clap(short, long)]
    pub error: bool,
}

#[cfg(test)]
mod cli_tests {
    use crate::mocks::cmd::{zr, zr_fail};

    #[test]
    fn basic() {
        zr(&["-h"]);
        zr(&["--help"]);
        zr(&["help"]);
        zr(&["-V"]);
        zr(&["--version"]);
    }

    #[test]
    fn log() {
        zr(&["-d", "help"]);
        zr(&["--debug", "help"]);
        zr(&["-i", "help"]);
        zr(&["--info", "help"]);
        zr(&["-w", "help"]);
        zr(&["--warning", "help"]);
    }

    #[test]
    fn commands() {
        zr(&["completion", "-h"]);
        zr(&["new", "-h"]);
        zr(&["get-config", "-h"]);
        zr(&["upgrade", "-h"]);
        zr_fail(&["unknown", "-h"]);
    }

    #[test]
    fn should_match_completion() {
        zr(&["completion", "-h"]);
        zr(&["completion", "bash", "-h"]);
        zr(&["completion", "zsh", "-h"]);
        zr(&["completion", "fish", "-h"]);
        zr(&["completion", "elvish", "-h"]);
        zr_fail(&["completion", "unknown", "-h"]);
    }
}