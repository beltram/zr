use std::path::PathBuf;

use clap::App;
use itertools::Itertools;

use kind::AppKind;
use local_args::LocalInitializrArgs;

use crate::{completion::PreInitializrArgs, utils::marshall::Tomlable};
use crate::utils::error::ErrorExt;
use crate::utils::file::PathExt;

pub mod local_args;
pub mod local_arg;
pub mod arg_kind;
mod kind;

struct AppLang<'a>(String, AppKind<'a>);

pub struct InitializrTemplate {
    pub root_path: PathBuf,
}

impl InitializrTemplate {
    const SEPARATOR: char = '-';
    const ABOUT_LANG: &'static str = "pick a language";
    const ABOUT_KIND: &'static str = "then a project kind";
    const CONFIG_FILE: &'static str = "zr.toml";

    pub fn new(root_path: PathBuf) -> Self {
        Self { root_path }
    }

    pub fn apps<'a>(&self, flags: &'a PreInitializrArgs) -> Vec<App<'a>> {
        self.all_templates().into_iter()
            .map(|(lang, kind)| {
                let root_config = self.root_path
                    .join(format!("{}-{}", lang, kind))
                    .join(Self::CONFIG_FILE);
                let my_flags = flags.iter()
                    .find(|(p, _)| p == &root_config)
                    .and_then(|(_, f)| f.as_ref());
                AppLang(lang, AppKind(kind, my_flags))
            })
            .into_group_map_by(|it| it.0.to_string()).into_iter()
            .map(|(lang, langs)| {
                let kinds = langs.into_iter().map(|it| it.1).map(App::from);
                App::new(lang.as_str())
                    .about(InitializrTemplate::ABOUT_LANG)
                    .subcommands(kinds)
            })
            .collect_vec()
    }

    pub fn all_args(&self) -> PreInitializrArgs {
        let root_args = self.root_config();
        self.all_template_paths()
            .map(|it| it.join(Self::CONFIG_FILE))
            .map(|local| {
                let args = if local.exists() {
                    if let Ok(mut local_args) = LocalInitializrArgs::from_file(&local) {
                        if let Some(root) = root_args.clone().as_mut() {
                            local_args.0.append(&mut root.0);
                            Some(local_args)
                        } else { Some(local_args) }
                    } else { root_args.clone() }
                } else { root_args.clone() };
                (local, args)
            })
            .collect_vec()
    }

    fn root_config(&self) -> Option<LocalInitializrArgs> {
        Some(self.root_path.join(Self::CONFIG_FILE))
            .filter(|it| it.exists())
            .and_then(|it| {
                LocalInitializrArgs::from_file(it)
                    .else_warn(format!("Invalid file format in {:?}/{}", self.root_path, Self::CONFIG_FILE))
                    .ok()
            })
    }

    pub fn flag_of(path: PathBuf) -> Option<LocalInitializrArgs> {
        Some(path.join(Self::CONFIG_FILE))
            .filter(|it| it.exists())
            .and_then(|it| LocalInitializrArgs::from_file(it).ok())
    }

    fn all_templates(&self) -> Vec<(String, String)> {
        self.root_path.children().as_slice().iter()
            .filter(|it| it.is_dir())
            .filter_map(|it| it.file_name_str())
            .filter_map(|it| Self::get_lang(it).zip(Self::get_kind(it)))
            .collect_vec()
    }

    fn all_template_paths(&self) -> impl Iterator<Item=PathBuf> {
        self.root_path.children().into_iter()
            .filter(|it| it.is_dir())
            .filter(|path| {
                path.file_name_str()
                    .map(|n| Self::get_lang(n).and(Self::get_kind(n)).is_some())
                    .unwrap_or_default()
            })
    }

    fn get_lang(name: &str) -> Option<String> {
        Some(name)
            .filter(|it| it.contains(Self::SEPARATOR))
            ?.split(Self::SEPARATOR).next()
            .map(|it| it.to_string())
    }

    fn get_kind(name: &str) -> Option<String> {
        Some(name)
            .filter(|it| it.contains(Self::SEPARATOR))
            .map(|it| it.split(Self::SEPARATOR))
            .map(|it| it.skip(1))
            .map(|mut it| it.join("-"))
    }
}

#[cfg(test)]
mod initializr_template_tests {
    use crate::{mocks::MockFs, PathExt};

    use super::*;

    mod into_app {
        use std::collections::BTreeMap;
        use std::iter::FromIterator;

        use crate::cmd::InitializrStdArgs;
        use crate::template::local_arg::LocalInitializrArg;

        use super::*;

        #[test]
        fn should_convert_single_lang() {
            let templates = MockFs::home().join("single-lang");
            templates.create_dir_all_or_fail();
            templates.join("java-web-app").create_dir_all_or_fail();
            let all_flags = vec![];
            let apps = InitializrTemplate::new(templates).apps(&all_flags);
            assert_eq!(apps.len(), 1);
            let java_app = apps.iter().find(|it| it.get_name() == "java").unwrap();
            assert_eq!(java_app.get_name(), "java");
            assert_eq!(java_app.get_about(), Some(InitializrTemplate::ABOUT_LANG));
            assert_eq!(java_app.get_subcommands().count(), 1);
            let java_web_app = java_app.get_subcommands().find(|it| it.get_name() == "web-app").unwrap();
            assert_eq!(java_web_app.get_name(), "web-app");
            assert_eq!(java_web_app.get_about(), Some(InitializrTemplate::ABOUT_KIND));
        }

        #[test]
        fn should_convert_multi_lang() {
            let templates = MockFs::home().join("multi-lang");
            templates.create_dir_all_or_fail();
            templates.join("rust-web-app").create_dir_all_or_fail();
            templates.join("rust-cli").create_dir_all_or_fail();
            let all_flags = vec![];
            let apps = InitializrTemplate::new(templates).apps(&all_flags);
            assert_eq!(apps.len(), 1);
            let rust_app = apps.iter().find(|it| it.get_name() == "rust").unwrap();
            assert_eq!(rust_app.get_name(), "rust");
            assert_eq!(rust_app.get_about(), Some(InitializrTemplate::ABOUT_LANG));
            assert_eq!(rust_app.get_subcommands().count(), 2);
            let rust_web_app = rust_app.get_subcommands().find(|it| it.get_name() == "web-app").unwrap();
            assert_eq!(rust_web_app.get_name(), "web-app");
            assert_eq!(rust_web_app.get_about(), Some(InitializrTemplate::ABOUT_KIND));
            let rust_cli = rust_app.get_subcommands().find(|it| it.get_name() == "cli").unwrap();
            assert_eq!(rust_cli.get_name(), "cli");
            assert_eq!(rust_cli.get_about(), Some(InitializrTemplate::ABOUT_KIND));
        }

        #[test]
        fn should_convert_config_into_args() {
            let root_dir = MockFs::home().join("with-args");
            let rust_cli = root_dir.join("rust-any");
            rust_cli.create_dir_all_or_fail();
            let template = InitializrTemplate::new(root_dir);
            let pre_fetched_flags = vec![(
                rust_cli.join(InitializrTemplate::CONFIG_FILE),
                Some(LocalInitializrArgs(BTreeMap::from_iter(vec![
                    (String::from("with-cache"), LocalInitializrArg::default())
                ])))
            )];
            let apps = template.apps(&pre_fetched_flags);
            assert_eq!(apps.len(), 1);
            let args = apps.iter()
                .find(|it| it.get_name() == "rust").unwrap()
                .get_subcommands()
                .find(|it| it.get_name() == "any").unwrap()
                .get_arguments()
                .collect_vec();
            // add std args + project name
            assert_eq!(args.len(), 1 + InitializrStdArgs::variants().len() + 1);
            let with_cache = args.get(1).unwrap();
            assert_eq!(with_cache.get_name(), "with-cache");
        }
    }

    mod all_templates {
        use super::*;

        #[test]
        fn should_not_find_any_when_empty() {
            let templates = MockFs::home().join("empty-templates");
            templates.create_dir_all_or_fail();
            assert!(templates.exists());
            assert!(templates.children().is_empty());
            assert!(InitializrTemplate::new(templates).all_templates().is_empty());
        }

        #[test]
        fn should_ignore_files() {
            let templates = MockFs::home().join("file-templates");
            templates.create_dir_all_or_fail();
            templates.join("file.txt").create().unwrap();
            assert_eq!(templates.children().len(), 1);
            assert!(InitializrTemplate::new(templates).all_templates().is_empty());
        }

        #[test]
        fn should_find_all() {
            let templates = MockFs::home().join("all-templates");
            templates.create_dir_all_or_fail();
            templates.join("file.txt").create().unwrap();
            templates.join("java-app").create_dir_all_or_fail();
            templates.join("rust-app").create_dir_all_or_fail();
            assert_eq!(templates.children().len(), 3);
            assert_eq!(InitializrTemplate::new(templates).all_templates().len(), 2);
        }

        #[test]
        fn should_find_only_valid_templates() {
            let templates = MockFs::home().join("valid-templates");
            templates.create_dir_all_or_fail();
            templates.join("rust").create_dir_all_or_fail();
            templates.join("java_web_app").create_dir_all_or_fail();
            assert_eq!(templates.children().len(), 2);
            assert!(InitializrTemplate::new(templates).all_templates().is_empty());
        }
    }

    mod all_args {
        use crate::mocks::MockFs;

        use super::*;

        fn get_all_args_for_app(name: &str) -> LocalInitializrArgs {
            let root = MockFs::template(name);
            InitializrTemplate::new(root.clone()).all_args().into_iter()
                .find(|(path, _)| path.parent() == Some(&root.join("one-app")))
                .unwrap().1.clone().unwrap()
        }

        #[test]
        fn should_find_all_args() {
            let all_flags = get_all_args_for_app("args-find-all");
            // contains at least root flags
            assert!(all_flags.0.len().gt(&2));
            assert!(all_flags.0.get("spring-boot-version").is_some());
            assert!(all_flags.0.get("kotlin-version").is_some());
        }

        #[test]
        fn should_find_root_ones_when_local_absent() {
            let all_flags = get_all_args_for_app("args-just-root");
            assert_eq!(all_flags.0.len(), 2);
            assert!(all_flags.0.get("spring-boot-version").is_some());
            assert!(all_flags.0.get("spring-cloud-version").is_some());
        }

        #[test]
        fn should_find_root_ones_when_local_invalid() {
            let all_flags = get_all_args_for_app("args-invalid-local");
            assert_eq!(all_flags.0.len(), 2);
            assert!(all_flags.0.get("spring-boot-version").is_some());
            assert!(all_flags.0.get("spring-cloud-version").is_some());
        }

        #[test]
        fn should_find_local_ones_when_root_absent() {
            let all_flags = get_all_args_for_app("args-just-local");
            assert_eq!(all_flags.0.len(), 1);
            assert!(all_flags.0.get("kotlin-version").is_some());
        }

        #[test]
        fn should_find_local_ones_when_root_invalid() {
            let all_flags = get_all_args_for_app("args-invalid-root");
            assert_eq!(all_flags.0.len(), 1);
            assert!(all_flags.0.get("kotlin-version").is_some());
        }

        #[test]
        fn should_not_fail_when_no_zr_toml() {
            let root = MockFs::template("args-none");
            let all_flags = InitializrTemplate::new(root.clone()).all_args();
            assert!(all_flags.get(0).unwrap().1.is_none());
        }
    }
}