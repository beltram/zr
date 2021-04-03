use clap::{App, AppSettings, Arg, Clap};
use field_types::FieldName;
use itertools::Itertools;
use serde::Serialize;

use crate::{completion::{dynamic_app::DynamicApp, PreInitializrArgs}};
use crate::config::global::Config;

use super::template::InitializrTemplate;

#[derive(Clap, Debug, Clone, Default)]
pub struct InitializrLang {
    #[clap(required = true)]
    pub lang: String,
    #[clap(flatten)]
    pub kind: InitializrKind,
}

#[derive(Clap, Debug, Clone, Default)]
pub struct InitializrKind {
    #[clap(required = true)]
    pub kind: String,
    #[clap(flatten)]
    pub std_args: InitializrStdArgs,
}

#[derive(Clap, Debug, Clone, Serialize, Default, FieldName)]
#[clap(setting = AppSettings::AllowExternalSubcommands)]
pub struct InitializrStdArgs {
    /// Opens project in IntelliJ
    #[clap(long)]
    pub idea: bool,
    /// Opens project in Visual Studio Code
    #[clap(long)]
    pub code: bool,
    /// Deletes the project just before exiting
    ///
    /// Useful when commands applied on generated project (most of the time test tasks)
    /// and we do not want to keep the project
    #[clap(long)]
    pub dry: bool,
    /// Also render any 'README.md' in generated project
    ///
    /// Such files are convenient to place in Handlebar templates to have empty directories
    /// or just to explain what the directory is intended to contain
    #[clap(long, parse(from_flag = std::ops::Not::not))]
    pub readme: bool,
    /// Erases existing project with same name if any
    ///
    /// If not present, user permission is asked
    #[clap(short, long)]
    pub force: bool,
}

impl InitializrStdArgs {
    pub(crate) const PROJECT_NAME_ARG_NAME: &'static str = "project-name";
    const PROJECT_NAME_ABOUT: &'static str = "name of generated project";

    pub fn project_name_arg<'a>() -> Arg<'a> {
        Arg::new(Self::PROJECT_NAME_ARG_NAME)
            .about(Self::PROJECT_NAME_ABOUT)
            .required(true)
            .takes_value(true)
    }

    pub fn variants() -> Vec<&'static str> {
        Self::as_field_name_array().iter()
            .map(|it| it.name())
            .collect_vec()
    }
}

pub struct InitializrSubCommand<'a>(pub &'a PreInitializrArgs);

impl<'a> DynamicApp<'a> for InitializrSubCommand<'a> {
    const SUBCOMMAND_NAME: &'static str = "new";

    fn subcommand_apps(&self, config: &'a Config) -> Vec<App<'a>> {
        config.all_templates().iter()
            .map(|it| InitializrTemplate::new(it.to_path_buf()))
            .flat_map(|it| it.apps(self.0))
            .collect_vec()
    }
}

#[cfg(test)]
mod cmd_initializr_tests {
    use clap::IntoApp;

    use super::*;

    #[test]
    fn simple() {
        let app: App = InitializrStdArgs::into_app();
        let matches = app.get_matches_from(vec!["zr", "my-proj", "--else"]);
        match matches.subcommand() {
            Some((external, ext_m)) => {
                let ext_args: Vec<&str> = ext_m.values_of("").unwrap().collect();
                assert_eq!(external, "my-proj");
                assert_eq!(ext_args, ["--else"]);
            }
            _ => {}
        }
    }
}