use clap::App;

use crate::config::global::Config;

pub trait DynamicApp<'a> {
    const NEW_NAME: &'static str = "new";
    const NEW_ABOUT: &'static str = "Bootstraps a new project from template";

    const SUBCOMMAND_NAME: &'static str;

    fn subcommand_apps(&self, config: &'a Config) -> Vec<App<'a>>;

    fn alter(&self, app: App<'a>) -> App<'a> {
        let name = app.get_name();
        let mut new_app = App::new(name);
        if name == Self::NEW_NAME {
            new_app = new_app.about(Self::NEW_ABOUT);
        }
        new_app
    }

    fn dynamic_app(&self, app: App<'a>, config: &'a Config) -> Option<App<'a>> {
        app.find_subcommand(Self::SUBCOMMAND_NAME)
            .map(|it| it.to_owned())
            .map(|it| self.alter(it))
            .map(|it| it.subcommands(self.subcommand_apps(config)))
    }
}