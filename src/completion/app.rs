use std::ops::Not;

use clap::App;
use itertools::Itertools;

use crate::cmd::InitializrSubCommand;
use crate::completion::PreInitializrArgs;

use super::dynamic_app::DynamicApp;
use crate::config::global::Config;

pub struct ZrApp {}

impl ZrApp {
    /// Creates a new App from the current one ; adding dynamic subcommands fetched from
    /// configuration or any other source
    pub fn app<'a>(app: App<'a>, config: &'a Config, initializr_args: &'a PreInitializrArgs) -> App<'a> {
        let mut dynamic_apps = Self::dynamic_apps(app.clone(), config, initializr_args);
        dynamic_apps.append(&mut Self::other_apps(app.clone(), &dynamic_apps));
        Self::copy(app, dynamic_apps)
    }

    /// Resolve all dynamic subcommands
    fn dynamic_apps<'a>(app: App<'a>, config: &'a Config, initializr_args: &'a PreInitializrArgs) -> Vec<App<'a>> {
        vec![
            InitializrSubCommand(initializr_args).dynamic_app(app, config),
        ].into_iter().filter_map(|it| it).collect_vec()
    }

    /// Subcommands which did not change
    fn other_apps<'a>(app: App<'a>, dynamic_apps: &Vec<App<'a>>) -> Vec<App<'a>> {
        app.get_subcommands()
            .filter(|cmd| dynamic_apps.iter().any(|it| it.get_name() == cmd.get_name()).not())
            .map(|it| it.to_owned())
            .collect_vec()
    }

    fn copy<'a>(app: App<'a>, subcommands: Vec<App<'a>>) -> App<'a> {
        let mut new_app = App::new(app.get_name())
            .args(app.get_arguments())
            .subcommands(subcommands);
        if let Some(bin_name) = app.get_bin_name() {
            new_app = new_app.bin_name(bin_name);
        }
        new_app
    }
}