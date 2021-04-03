#[macro_use]
extern crate cached;
#[macro_use]
extern crate derive_new;
#[macro_use]
extern crate enum_display_derive;
extern crate handlebars;
#[cfg(test)]
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate log;
extern crate serde;
#[macro_use]
extern crate shells;

use std::{env, path::PathBuf};
use std::iter::FromIterator;

use colored::Colorize;
use handlebars::Handlebars;
use itertools::Itertools;
use serde::Serialize;
use serde_json::{Map, Value};

use cmd::InitializrLang;

use crate::{
    completion::CliCompletion,
    console::emoji,
    utils::anyhow_err::{ErrConversion, OptConversion},
};
use crate::config::global::Config;
use crate::console::asker::Asker;
use crate::data::InitializrData;
use crate::utils::error::ErrorExt;
use crate::utils::file::PathExt;

use self::{actions::ProjectActions};
use self::handlebars::Context;

mod actions;
pub mod config;
pub mod cmd;
pub mod template;
mod data;
pub mod utils;
pub mod completion;
pub mod console;
pub mod git;
pub mod cli;
pub mod cli_log;
pub mod entrypoint;
pub mod get_config;
pub mod upgrade;
#[cfg(test)]
pub mod mocks;

pub struct Initializr {}

impl Initializr {
    const TEMPLATE_EXTENSION: &'static str = ".hbs";
    const README: &'static str = "README.md";
    const HIDDEN_PATH_ESCAPER: char = '!';

    pub fn bootstrap(lang: InitializrLang) {
        let template_path = Self::find_template(Config::get(), lang.clone())
            .fail(format!("No template found for lang '{}' and kind '{}'", lang.lang.as_str().yellow(), lang.kind.kind.as_str().yellow()));
        let initializr_flags = CliCompletion::initializr_args(Config::get()).into_iter()
            .filter(|(path, _)| path.parent() == Some(template_path.as_path()))
            .collect_vec();
        Self::new_project(template_path, InitializrData::from(initializr_flags));
    }

    /// Bootstraps a new project based on flags provided
    fn new_project(template_path: PathBuf, data: InitializrData) {
        info!("Generating project {} from {}", data.project_name().green(), template_path.path_str().green());
        let mut handlebar: Handlebars = Handlebars::new();
        handlebar.register_templates_directory(Self::TEMPLATE_EXTENSION, &template_path)
            .fail(format!("No templates found in {:?}", template_path));
        let destination = env::current_dir().unexpected_failure().join(data.project_name());
        if Self::propose_overwrite(&destination, &data).is_ok() {
            handlebar.get_templates().iter()
                .map(|(name, _)| name)
                .filter(|name| !name.ends_with(Self::README) || data.should_render_readme())
                .flat_map(|name| Self::maybe_duplicate(&handlebar, name, &data))
                .for_each(|(name, is_duplicate)| Self::render_template(&handlebar, data.clone(), name.as_str(), &destination, is_duplicate));
            info!("Generated project in {}", format!("{:?}", destination).as_str().green());
            Self::then_initialize(destination, data);
        }
    }

    fn render_template<D>(handlebar: &Handlebars, data: D, template_name: &str, destination: &PathBuf, is_duplicate: bool) where D: Serialize {
        handlebar.render_template(template_name, &data).wrap()
            .map(|name| destination.join(&name))
            .and_then(|destination_file| {
                destination_file.parent()
                    .wrap(format!("Could not find parent of {:?}", destination_file))
                    .and_then(|it| it.create_dir_all())
                    .fail(format!("Failed creating parents of {:?}", destination_file));
                destination_file.open_new()
                    .and_then(|file| {
                        match handlebar.render_to_write(&template_name, &data, &file) {
                            Err(e) if e.desc.starts_with("Template not found") && is_duplicate => Ok(destination_file.clone()),
                            Err(e) => anyhow::Result::Err(anyhow::Error::msg(e)),
                            _ => Ok(destination_file.clone()),
                        }
                    })
                    .map(Self::escape_hidden_files)
            })
            .warn(format!("Handlebar failed rendering file '{}'", template_name.yellow()));
    }

    fn then_initialize(new_project_path: PathBuf, data: InitializrData) {
        info!("Now initializing it...");
        env::current_dir().wrap()
            .and_then(|current_dir| {
                ProjectActions::new(&new_project_path, &data).apply()
                    .and_then(|_| env::set_current_dir(current_dir).wrap())
            })
            .warn(format!("Failed initializing new project in {:?}", new_project_path));
        info!("{} {}", "Happy coding".bold().yellow(), emoji::PARTY)
    }

    /// If it sees one path with a placeholder and the latter is a multi arg it duplicates it
    fn maybe_duplicate(handlebar: &Handlebars, name: &str, data: &InitializrData) -> Vec<(String, bool)> {
        let duplicated = data.multi_args().iter()
            .flat_map(|(k, values)| {
                values.iter()
                    .map(move |v| vec![(k.to_string(), v.to_owned())])
                    .map(Map::from_iter)
                    .map(Value::from)
                    .filter_map(|it| Context::wraps(it).ok())
                    .filter_map(|ctx| handlebar.render_template_with_context(name, &ctx).ok())
            })
            .map(|it| (it, true))
            .collect_vec();
        if duplicated.is_empty() { vec![(name.to_string(), false)] } else { duplicated }
    }

    /// Hidden files are not supported by Handlebars. It is embarrassing for files or folder starting
    /// with dot e.g. `.gitlab-ci.yml`, `.github`, `.gitignore`.
    /// To overcome this those files have to be prefixed with '!' in templates.
    /// Then, here, after write, we simply rename them.
    fn escape_hidden_files(into_path: PathBuf) {
        if let Some(filename) = into_path.file_name_str() {
            filename.strip_prefix(Self::HIDDEN_PATH_ESCAPER)
                .map(|it| into_path.with_file_name(it))
                .map(|it| into_path.rename(&it));
        }
    }

    /// If project path already exists, proposes to overwrite it or just do it if -f flag passed
    fn propose_overwrite(proj_path: &PathBuf, data: &InitializrData) -> anyhow::Result<()> {
        if proj_path.exists() {
            if data.is_force() {
                proj_path.delete_dir().wrap()
            } else {
                let question = &format!("A project named '{}' already exists, do you want to overwrite it ?", data.project_name().green());
                let did_user_answered_yes = Asker::ask(question, || {
                    proj_path.delete_dir()
                        .fail(format!("Failed deleting existing project {:?}", proj_path));
                });
                if did_user_answered_yes { Ok(()) } else { Err(anyhow::Error::msg("")) }
            }
        } else { Ok(()) }
    }

    fn find_template(config: Config, lang: InitializrLang) -> Option<PathBuf> {
        let name = format!("{}-{}", lang.lang, lang.kind.kind);
        config.find_template(&name)
    }
}

#[cfg(test)]
mod initializr_tests {
    use std::env;
    use std::path::PathBuf;

    use itertools::Itertools;
    use serde_json::Value;

    use crate::config::global::Config;
    use crate::data::data_arg::DataArg;
    use crate::data::InitializrData;
    use crate::Initializr;
    use crate::mocks::MockFs;

    fn before_all() {
        env::set_current_dir(MockFs::dev()).unwrap();
    }

    fn default_config() -> Config {
        Config {
            repositories: Some(vec!["https://github.com/beltram/zr-test.git".to_string()]),
            ..Default::default()
        }
    }

    mod search {
        use crate::cmd::{InitializrKind, InitializrLang};
        use crate::Initializr;

        use super::*;

        #[test]
        fn should_find_existing_template() {
            before_all();
            let config = default_config();
            let existing = MockFs::template("java-app");
            assert!(existing.exists());
            let kind = InitializrKind { kind: String::from("app"), ..Default::default() };
            let flags = InitializrLang { lang: String::from("java"), kind };
            let java_app_template = Initializr::find_template(config, flags);
            assert_eq!(java_app_template, Some(existing));
        }

        #[test]
        fn should_not_find_any_existing_template_when_name_unknown() {
            before_all();
            let config = default_config();
            let kind = InitializrKind { kind: String::from("unknown"), ..Default::default() };
            let flags = InitializrLang { lang: String::from("unknown"), kind };
            assert!(Initializr::find_template(config, flags).is_none());
        }
    }

    mod project_name {
        use itertools::Itertools;

        use crate::utils::file::PathExt;

        use super::*;

        #[test]
        fn should_apply_project_name() {
            before_all();
            let data = data("some-java-proj", &[]);
            let generated = new(data, "java-app");
            assert!(generated.exists());
            assert_eq!(generated.file_name_str(), Some("some-java-proj"));
        }

        #[test]
        fn project_name_should_also_be_available_as_placeholder_with_variants() {
            before_all();
            let data = data("a-b-c", &[]);
            let lines = new(data, "arg-variants").join("proj.txt").lines();
            assert_eq!(lines, vec![
                "a-b-c",
                "a/b/c",
                "a.b.c",
                "A-B-C",
                "a-b-c",
                "a b c",
                "A B C",
                "aBC",
                "ABC",
                "a-b-c",
                "A-B-C",
                "a_b_c",
                "A_B_C",
            ].iter().map(|it| it.to_string()).collect_vec());
        }

        #[test]
        fn should_render_project_name_path_as_nested_folders() {
            before_all();
            let data = data("x-y-z", &[]);
            let project = new(data, "with-project-path");
            assert!(project.join("x").join("y").join("z").exists());
        }
    }

    mod multi {
        use itertools::Itertools;

        use crate::data::data_arg::DataArg;
        use crate::data::InitializrData;
        use crate::utils::file::PathExt;

        use super::*;

        #[test]
        fn multi_variants_should_be_available_as_placeholder() {
            before_all();
            let data = multi_data("all-multi-variants", &[
                ("group", &["a.corp", "b.corp"]),
                ("whisper", &["hello", "world"]),
                ("shout", &["HELLO", "WORLD"]),
                ("stuck", &["helLo", "WoRld"]),
                ("free", &["hello world"]),
            ]);
            let lines = new(data, "multi-variants").join("multi.txt").lines();
            assert_eq!(lines, vec![
                "a.corp,b.corp,",
                "a/corp,b/corp,",
                "a.corp,b.corp,",
                "HELLO,WORLD,",
                "hello,world,",
                "hel lo,wo rld,",
                "Hello World,",
                "helloWorld,",
                "HelloWorld,",
                "hello-world,",
                "Hello-World,",
                "hello_world,",
                "HELLO_WORLD,",
            ].iter().map(|it| it.to_string()).collect_vec());
        }

        fn multi_data(name: &str, args: &[(&str, &[&str])]) -> InitializrData {
            let proj_key = String::from(InitializrData::PROJ_KEY);
            let proj = DataArg::from((proj_key, Some(Value::from(name)), None));
            let args = args.into_iter()
                .map(move |(k, v)| (k.to_string(), Some(Value::from(&v[..]))))
                .map(|(k, v)| DataArg::from((k, v, None)))
                .merge_by(vec![proj].into_iter(), |_, _| true);
            InitializrData::from((args.collect_vec(), vec![]))
        }
    }

    mod args {
        use itertools::Itertools;

        use crate::utils::file::PathExt;

        use super::*;

        #[test]
        fn arg_variants_should_be_available_as_placeholder() {
            before_all();
            let data = data("all-variants", &[
                ("group", "io.corp.comp"),
                ("whisper", "helloworld"),
                ("shout", "HELLOWORLD"),
                ("stuck", "helloWorld"),
                ("free", "hello world"),
            ]);
            let lines = new(data, "arg-variants").join("arg.txt").lines();
            assert_eq!(lines, vec![
                "io/corp/comp",
                "io.corp.comp",
                "HELLOWORLD",
                "helloworld",
                "hello world",
                "Hello World",
                "helloWorld",
                "HelloWorld",
                "hello-world",
                "Hello-World",
                "hello_world",
                "HELLO_WORLD",
            ].iter().map(|it| it.to_string()).collect_vec());
        }

        #[test]
        fn should_pass_local_arg_to_handlebar_renderer() {
            before_all();
            let data = data("with-arg-no-default", &[("arg", "the-value")]);
            let file = new(data, "with-arg").join("empty.txt");
            assert_eq!(file.read_to_string(), "the-value");
        }
    }

    mod standard {
        use std::ops::Not;

        use crate::utils::file::PathExt;

        use super::*;

        #[test]
        fn should_exclude_readme_when_requested() {
            before_all();
            let data = data("no-readme-proj", &[]);
            let project = new(data, "no-readme");
            assert!(project.join("README.md").exists().not());
            assert!(project.join("folder").join("README.md").exists().not());
        }

        #[test]
        fn should_not_exclude_readme_when_not_requested() {
            before_all();
            let data = data("with-readme-proj", &[("readme", "")]);
            let project = new(data, "no-readme");
            assert!(project.join("README.md").exists());
            assert!(project.join("folder").join("README.md").exists());
        }

        #[test]
        fn should_support_hidden_files_by_escaping_exclamation_mark() {
            before_all();
            let gitlab = data("gitlab", &[]);
            let gitlab_ci = new(gitlab, "gitlab").join(".gitlab-ci.yml");
            assert!(gitlab_ci.exists());
            let github = data("github", &[]);
            let github_action = new(github, "github").join(".github/workflows/dev.yml");
            assert!(github_action.exists());
            let gitignore = data("gitignore", &[]);
            let gitignore_file = new(gitignore, "gitignore").join(".gitignore");
            assert!(gitignore_file.exists());
        }

        #[test]
        fn should_delete_generated_project_after_created_when_dry() {
            before_all();
            let data = data("dry-proj", &[("dry", "")]);
            assert!(new(data, "burn-after-reading").exists().not());
        }

        #[test]
        fn should_erase_existing_project_when_force() {
            before_all();
            let existing = data("any-proj", &[]);
            let existing_file = new(existing, "force").join("empty.txt");
            existing_file.write_to("existing");
            assert_eq!(existing_file.read_to_string(), "existing");
            let replacer = data("any-proj", &[("force", "")]);
            let readme = new(replacer, "force").join("empty.txt");
            assert_ne!(readme.read_to_string(), "existing");
        }
    }

    fn data(name: &str, args: &[(&str, &str)]) -> InitializrData {
        let proj_key = String::from(InitializrData::PROJ_KEY);
        let proj = DataArg::from((proj_key, Some(Value::from(name)), None));
        let args = args.into_iter()
            .map(move |(k, v)| (k.to_string(), Some(Value::from(*v))))
            .map(|(k, v)| DataArg::from((k, v, None)))
            .merge_by(vec![proj].into_iter(), |_, _| true);
        InitializrData::from((args.collect_vec(), vec![]))
    }

    pub fn new(data: InitializrData, name: &str) -> PathBuf {
        Initializr::new_project(MockFs::template(name), data.clone());
        env::current_dir().unwrap().join(data.project_name())
    }
}