use clap::{App, Arg, IntoApp};
use itertools::Itertools;

use crate::cmd::InitializrStdArgs;

use super::{InitializrTemplate, LocalInitializrArgs};

pub struct AppKind<'a>(pub(crate) String, pub(crate) Option<&'a LocalInitializrArgs>);

impl<'a> From<AppKind<'a>> for App<'a> {
    fn from(kind: AppKind<'a>) -> Self {
        let mut app = App::new(kind.0.as_str())
            .about(InitializrTemplate::ABOUT_KIND);
        app = app.arg(InitializrStdArgs::project_name_arg());
        if let Some(f) = kind.1 {
            app = app.args(Vec::<Arg>::from(f))
        }
        let std_args = InitializrStdArgs::into_app();
        let std_args = std_args.get_arguments().collect_vec();
        app = app.args(std_args);
        app
    }
}