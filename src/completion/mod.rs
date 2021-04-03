use std::{ops::Not, path::PathBuf};

use clap::{App, IntoApp};
use clap_generate::{generate_to, Generator};
#[cfg(unix)]
use clap_generate::generators::{Bash, Elvish, Fish, Zsh};
#[cfg(windows)]
use clap_generate::generators::{Bash, PowerShell};
use clap_generate::generators::PowerShell;
use colored::Colorize;

use crate::{
    cli::Cli,
    completion::{app::ZrApp, model::Shell},
};
use crate::config::global::Config;
use crate::template::InitializrTemplate;
use crate::template::local_args::LocalInitializrArgs;
use crate::utils::error::ErrorExt;
use crate::utils::file::PathExt;
use crate::utils::user::User;

pub mod model;
mod app;
pub mod dynamic_app;

pub(crate) type PreInitializrArgs = Vec<(PathBuf, Option<LocalInitializrArgs>)>;

/// Generates and applies shell completion files
pub struct CliCompletion {}

impl CliCompletion {
    const ZSH_DIR: &'static str = "/usr/local/share/zsh/site-functions";
    const BASH_DIR: &'static str = ".bash_completion.d";

    pub fn apply(desired_shell: Option<Shell>) {
        if let Some(shell) = desired_shell.or_else(Shell::current) {
            info!("Will generate completion files for {}", shell.as_ref().green());
            let config = Config::get();
            let initializr_args = Self::initializr_args(config.clone());
            let app = Self::app(Cli::into_app(), &config, &initializr_args);
            Self::create_completion_for(app, shell);
        } else {
            warn!("Could not determine current shell. Consider passing it explicitly e.g. 'zr completion zsh'")
        }
    }

    pub fn app<'a>(app: App<'a>, config: &'a Config, initializr_args: &'a PreInitializrArgs) -> App<'a> {
        ZrApp::app(app, config, initializr_args)
    }

    // we need to initialize it here because those initializr flags require a longer lifetime than App<'help>
    pub fn initializr_args(config: Config) -> Vec<(PathBuf, Option<LocalInitializrArgs>)> {
        config.all_templates().iter()
            .map(|it| InitializrTemplate::new(it.to_path_buf()))
            .flat_map(|it| it.all_args())
            .collect()
    }

    fn create_completion_for(mut app: App, shell: Shell) {
        match shell {
            Shell::Bash => Self::create_completion::<Bash>(&mut app, shell),
            Shell::Zsh => Self::create_completion::<Zsh>(&mut app, shell),
            Shell::Elvish => Self::create_completion::<Elvish>(&mut app, shell),
            Shell::Fish => Self::create_completion::<Fish>(&mut app, shell),
            Shell::Powershell => Self::create_completion::<PowerShell>(&mut app, shell),
        }
    }

    fn create_completion<G: Generator>(app: &mut App, shell: Shell) {
        let bin_name = app.get_name().to_string();
        let dir = Self::completion_dir(shell);
        generate_to::<G, _, _>(app, &bin_name, &dir);
        info!("Generated completion files for {} in {}", shell.as_ref().green(), format!("{:?}", dir).as_str().green());
    }

    fn completion_dir(shell: Shell) -> PathBuf {
        let dir = match shell {
            Shell::Zsh => PathBuf::from(Self::ZSH_DIR),
            Shell::Bash => User::home().map(|it| it.join(Self::BASH_DIR)).fail("Could not find home dir"),
            _ => panic!("Not supported yet"),
        };
        if dir.exists().not() {
            dir.create_dir()
                .fail(format!("Failed creating completion scripts folder {:?}", dir));
        }
        dir
    }
}

#[cfg(test)]
mod completion_test {
    use super::*;

    #[test]
    fn app_should_have_bin_name() {
        assert!(CliCompletion::app(Cli::into_app(), &Config::default(), &vec![]).get_bin_name().is_some())
    }
}