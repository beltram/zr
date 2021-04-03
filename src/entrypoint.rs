use clap::Clap;

use crate::{cli::{Cli, SubCommand::*}, cli_log::CliLog, completion::CliCompletion, get_config::GetConfig, Initializr, upgrade::ZrUpgrade};

/// Executable entrypoint
pub struct CliEntryPoint;

impl CliEntryPoint {
    pub fn run() {
        let cli = Cli::parse();
        CliLog::init(&cli.log);
        if let Some(cmd) = cli.cmd {
            match cmd {
                Upgrade {} => ZrUpgrade::upgrade(),
                Completion { shell } => { CliCompletion::apply(shell); }
                New { lang } => Initializr::bootstrap(lang),
                GetConfig => GetConfig::exec(),
            }
        } else {
            panic!("Not implemented yet !");
        }
    }
}