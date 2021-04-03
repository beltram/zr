use std::{fmt::Debug, ops::BitXor};

use clap::{ArgMatches, IntoApp};
use itertools::Itertools;
use serde::Serialize;
use serde_json::{Map, Value};

use crate::{
    completion::{CliCompletion, PreInitializrArgs},
};
use crate::cli::Cli;
use crate::data::data_arg::DataArg;
use crate::template::local_arg::LocalInitializrArg;

use super::{
    cmd::{InitializrStdArgs, InitializrStdArgsFieldName},
};
use crate::config::global::Config;

pub mod data_arg;
mod arg_path;
mod arg_case;
pub mod arg_cmd;

#[derive(Serialize, Clone, Debug)]
#[serde(rename_all = "kebab-case")]
pub struct InitializrData {
    #[serde(flatten)]
    pub args: Map<String, Value>,
    #[serde(skip)]
    pub commands: Vec<String>,
}

impl InitializrData {
    const NEW_CMD_NAME: &'static str = "new";
    pub(crate) const PROJ_KEY: &'static str = "proj";

    fn matched_args(new_matches: &ArgMatches) -> Option<&ArgMatches> {
        new_matches
            .subcommand_matches(Self::NEW_CMD_NAME)
            .and_then(|it| it.subcommand())
            .and_then(|(_, it)| it.subcommand())
            .map(|(_, it)| it)
    }

    fn parse_project_name(matched_args: Option<&ArgMatches>) -> DataArg {
        let proj_name = matched_args
            .and_then(|it| it.value_of(InitializrStdArgs::PROJECT_NAME_ARG_NAME))
            .map(Value::from);
        DataArg::from((String::from(Self::PROJ_KEY), proj_name, None))
    }

    fn parse_args<'a>(matched_args: Option<&'a ArgMatches>, local_args: &'a PreInitializrArgs) -> impl Iterator<Item=DataArg> + 'a {
        Self::local_template_args(&local_args)
            .merge_by(InitializrStdArgs::variants().into_iter().map(|it| (it, None)), |_, _| true)
            .flat_map(move |(key, maybe_arg)| {
                matched_args
                    .filter(|a| {
                        let is_present = a.is_present(key);
                        let is_default = maybe_arg.as_ref().map(|it| it.is_default()).unwrap_or_default();
                        is_present.bitxor(is_default)
                    })
                    .map(|a| {
                        let is_multi = maybe_arg.as_ref().map(|it| it.is_multi()).unwrap_or_default();
                        if is_multi {
                            a.values_of(key)
                                .map(|it| it.into_iter().collect_vec())
                                .map(Value::from)
                        } else { a.value_of(key).map(Value::from) }
                    })
                    .map(|value| DataArg::from((Self::trim_arg_key(key), value, maybe_arg)))
            })
    }

    fn trim_arg_key(key: &str) -> String {
        key.trim_start_matches(|c: char| c == '-').to_string()
    }

    fn local_template_args(local_args: &PreInitializrArgs) -> impl Iterator<Item=(&str, Option<LocalInitializrArg>)> {
        local_args.iter()
            .filter(|(_, maybe_args)| maybe_args.is_some())
            .map(|(_, maybe_args)| maybe_args.as_ref().unwrap())
            .flat_map(|it| it.0.iter().map(|(key, arg)| (key.as_str(), Some(arg.to_owned()))))
    }

    pub fn multi_args(&self) -> Vec<(&String, &Vec<Value>)> {
        self.args.iter()
            .filter_map(|(k, v)| v.as_array().map(|it| (k, it)))
            .collect_vec()
    }
}

impl From<PreInitializrArgs> for InitializrData {
    fn from(local_args: PreInitializrArgs) -> Self {
        let config = Config::get();
        let dyn_app = CliCompletion::app(Cli::into_app(), &config, &local_args);
        let new_matches = dyn_app.get_matches();
        let matched_args = Self::matched_args(&new_matches);
        let proj = Self::parse_project_name(matched_args);
        let (args, commands): (Vec<DataArg>, Vec<DataArg>) = Self::parse_args(matched_args, &local_args)
            .merge_by(vec![proj].into_iter(), |_, _| true)
            .partition(|it| it.is_not_cmd());
        let cmds = commands.iter()
            .filter_map(|it| it.cmd.to_owned())
            .sorted()
            .map(|it| it.cmd)
            .collect_vec();
        Self::from((
            args.into_iter().collect_vec(),
            cmds,
        ))
    }
}

impl From<(Vec<DataArg>, Vec<String>)> for InitializrData {
    fn from((args, commands): (Vec<DataArg>, Vec<String>)) -> Self {
        Self {
            args: args.into_iter().flat_map(|it| it.variants).collect(),
            commands,
        }
    }
}

impl InitializrData {
    const DEFAULT_PROJECT_NAME: &'static str = "sample";

    pub fn project_name(&self) -> &str {
        self.args.get(Self::PROJ_KEY)
            .and_then(|it| it.as_str())
            .unwrap_or(Self::DEFAULT_PROJECT_NAME)
    }
    pub fn is_force(&self) -> bool { self.contains(InitializrStdArgsFieldName::Force.name()) }
    pub fn is_idea(&self) -> bool { self.contains(InitializrStdArgsFieldName::Idea.name()) }
    pub fn is_vs_code(&self) -> bool { self.contains(InitializrStdArgsFieldName::Code.name()) }
    pub fn should_render_readme(&self) -> bool { self.contains(InitializrStdArgsFieldName::Readme.name()) }
    pub fn is_dry(&self) -> bool { self.contains(InitializrStdArgsFieldName::Dry.name()) }
    fn contains(&self, key: &str) -> bool {
        self.args.iter().any(|(k, _)| k == key)
    }
}

#[cfg(test)]
mod data_tests {
    use std::{env, ops::Not};

    use crate::{mocks::{cmd::*, MockFs}, PathExt};

    // because those tests are ok in standalone mode but mocked filesystem is altered
    // when executed with 'cargo test' for unknown reason ATM
    pub fn before_each() {
        env::set_var("BASH", "1");
        let config = MockFs::config();
        if config.read_to_string().is_empty() {
            config.write_to(r#"
            repositories = [
                'https://github.com/beltram/zr-tests.git',
                'https://github.com/beltram/zr-commands.git',
                'https://github.com/beltram/no-arg-initializr.git',
            ]
            "#);
        }
    }

    mod std_args {
        use super::*;

        #[test]
        fn should_just_generate_project() {
            before_each();
            zr(&["new", "simple", "project", "just-simple-proj"]);
            let generated = MockFs::home().join("just-simple-proj");
            assert!(generated.exists());
            assert!(generated.join("empty.txt").exists());
        }

        #[test]
        fn should_render_project_path() {
            before_each();
            zr(&["new", "just", "project-path", "a-b-c"]);
            let generated = MockFs::home().join("a-b-c");
            assert!(generated.exists());
            assert!(generated.join("a").join("b").join("c").exists());
        }

        #[test]
        fn should_consider_no_readme_by_default() {
            before_each();
            zr(&["new", "simple", "project", "no-readme-proj"]);
            let generated = MockFs::home().join("no-readme-proj");
            assert!(generated.exists());
            assert!(generated.join("README.md").exists().not());
        }

        #[test]
        fn should_consider_with_readme_std_arg() {
            before_each();
            zr(&["new", "simple", "project", "with-readme-proj", "--readme"]);
            let generated = MockFs::home().join("with-readme-proj");
            assert!(generated.exists());
            assert!(generated.join("README.md").exists());
        }

        #[test]
        fn should_consider_dry_std_arg() {
            before_each();
            zr(&["new", "simple", "project", "then-delete-proj", "--dry"]);
            let generated = MockFs::home().join("then-delete-proj");
            assert!(generated.exists().not());
        }

        #[test]
        fn should_consider_force_std_arg() {
            before_each();
            zr(&["new", "simple", "project", "then-delete-proj"]);
            let generated_file = MockFs::home().join("then-delete-proj").join("empty.txt");
            assert!(generated_file.exists());
            generated_file.write_to("new");
            zr(&["new", "simple", "project", "then-delete-proj", "--force"]);
            assert_ne!(generated_file.read_to_string().as_str(), "new");
        }

        #[test]
        fn should_consider_template_with_other_lang() {
            before_each();
            zr(&["new", "other", "lang", "other-proj", "-h"]);
        }

        #[test]
        fn should_not_consider_unknown_lang() {
            before_each();
            zr_fail(&["new", "unknown", "project", "unknown-lang", "-h"]);
        }

        #[test]
        fn should_not_consider_unknown_kind() {
            before_each();
            zr_fail(&["new", "simple", "unknown", "unknown-kind", "-h"]);
        }

        #[test]
        fn should_not_consider_missing_project_name() {
            before_each();
            zr_fail(&["new", "simple", "project"]);
        }

        #[test]
        fn should_consider_std_args() {
            before_each();
            zr(&["new", "just", "std-args", "std-proj", "--idea", "-h"]);
        }

        #[test]
        fn should_not_consider_unknown_std_args() {
            before_each();
            zr_fail(&["new", "just", "std-args", "std-proj", "--unknown", "-h"]);
        }
    }

    mod args {
        use super::*;

        #[test]
        fn should_render_local_string_arg() {
            before_each();
            let proj_name = "arg-str-without-default-value";
            zr(&["new", "just", "text-arg", proj_name, "--str-arg-no-default", "the-value"]);
            let file = MockFs::home().join(proj_name).join("no-default.txt");
            assert_eq!(file.read_to_string().as_str(), "the-value");
        }

        #[test]
        fn should_render_local_arg_with_equals() {
            before_each();
            let proj_name = "arg-with-equals";
            zr(&["new", "just", "text-arg", proj_name, "--str-arg-no-default=the-value"]);
            let file = MockFs::home().join(proj_name).join("no-default.txt");
            assert_eq!(file.read_to_string().as_str(), "the-value");
        }

        #[test]
        fn should_render_local_arg_default_when_absent() {
            before_each();
            let proj_name = "arg-with-default";
            zr(&["new", "just", "text-arg", proj_name]);
            let file = MockFs::home().join(proj_name).join("default.txt");
            assert_eq!(file.read_to_string().as_str(), "default");
        }

        #[test]
        fn should_supersede_arg_default() {
            before_each();
            let proj_name = "arg-with-default-superseded";
            zr(&["new", "just", "text-arg", proj_name, "--str-arg-with-default", "superseded"]);
            let file = MockFs::home().join(proj_name).join("default.txt");
            assert_eq!(file.read_to_string().as_str(), "superseded");
        }

        #[test]
        fn should_consider_global_args() {
            before_each();
            zr(&["new", "just", "global-args", "global-proj", "--one-global", "-h"]);
        }

        #[test]
        fn should_not_consider_unknown_global_args() {
            before_each();
            zr_fail(&["new", "just", "global-args", "global-proj", "--not-one-global", "-h"]);
        }

        #[test]
        fn should_consider_local_args() {
            before_each();
            zr(&["new", "just", "local-args", "local-proj", "--one-local", "-h"]);
        }

        #[test]
        fn should_not_consider_unknown_local_args() {
            before_each();
            zr_fail(&["new", "just", "local-args", "local-proj", "--not-one-local", "-h"]);
        }

        #[test]
        fn should_consider_template_without_local_args_file() {
            before_each();
            zr(&["new", "just", "no-local-args-file", "local-proj", "-h"]);
        }

        #[test]
        fn should_consider_template_without_global_args_file() {
            before_each();
            zr(&["new", "non", "global-args-simple", "a", "--idea", "-h"]);
            zr(&["new", "non", "global-args-simple", "a", "--code", "-h"]);
            zr(&["new", "non", "global-args-simple", "a", "--dry", "-h"]);
            zr(&["new", "non", "global-args-simple", "a", "--readme", "-h"]);
            zr(&["new", "non", "global-args-simple", "a", "--force", "-h"]);
            zr(&["new", "non", "global-args-simple", "a", "-f", "-h"]);
        }

        #[test]
        fn should_consider_template_without_global_args_file_but_with_local() {
            before_each();
            zr(&["new", "non", "global-args-with-local", "a", "--just-local", "-h"]);
        }
    }

    mod flags {
        use super::*;

        #[test]
        fn should_render_flag() {
            before_each();
            let proj_name = "just-simple-flag";
            zr(&["new", "just", "flag", proj_name, "--simple-flag"]);
            let file = MockFs::home().join(proj_name).join("flag.txt");
            assert_eq!(file.read_to_string().as_str(), "just a simple flag");
        }

        #[test]
        fn should_not_render_flag_when_absent() {
            before_each();
            let proj_name = "no-simple-flag";
            zr(&["new", "just", "flag", proj_name]);
            let file = MockFs::home().join(proj_name).join("flag.txt");
            assert!(file.read_to_string().is_empty());
        }

        #[test]
        fn should_render_negated_flag_when_absent() {
            before_each();
            let proj_name = "just-absent-negated-flag";
            zr(&["new", "just", "flag", proj_name]);
            let file = MockFs::home().join(proj_name).join("negate.txt");
            assert_eq!(file.read_to_string().as_str(), "just a negated flag");
        }

        #[test]
        fn should_not_render_negated_flag_when_present() {
            before_each();
            let proj_name = "just-present-negated-flag";
            zr(&["new", "just", "flag", proj_name, "--no-simple-flag"]);
            let file = MockFs::home().join(proj_name).join("negate.txt");
            assert!(file.read_to_string().is_empty());
        }

        #[test]
        fn should_render_explicit_not_negated_flag() {
            before_each();
            let proj_name = "just-simple-explicit-not-negated-flag";
            zr(&["new", "just", "flag", proj_name, "--explicit-simple-flag"]);
            let file = MockFs::home().join(proj_name).join("explicit-flag.txt");
            assert_eq!(file.read_to_string().as_str(), "just a simple explicit flag");
        }

        #[test]
        fn should_not_render_explicit_not_negated_flag_when_absent() {
            before_each();
            let proj_name = "no-simple-explicit-not-negated-flag";
            zr(&["new", "just", "flag", proj_name]);
            let file = MockFs::home().join(proj_name).join("explicit-flag.txt");
            assert!(file.read_to_string().is_empty());
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn should_render_multi_arg() {
            before_each();
            let proj_name = "just-multi-arg";
            zr(&["new", "just", "multi", proj_name, "--names", "john,bob,alice"]);
            let file = MockFs::home().join(proj_name).join("each.txt");
            assert_eq!(file.read_to_string().as_str(), "john,bob,alice,");
        }

        #[test]
        fn should_render_multi_arg_with_equals() {
            before_each();
            let proj_name = "just-multi-arg-equals";
            zr(&["new", "just", "multi", proj_name, "--names=john,bob,alice"]);
            let file = MockFs::home().join(proj_name).join("each.txt");
            assert_eq!(file.read_to_string().as_str(), "john,bob,alice,");
        }

        #[test]
        fn should_render_multi_arg_with_multiple_occurrences() {
            before_each();
            let proj_name = "just-multi-arg-multi-occurrences";
            zr(&["new", "just", "multi", proj_name, "--names", "john", "--names", "bob", "--names", "alice"]);
            let file = MockFs::home().join(proj_name).join("each.txt");
            assert_eq!(file.read_to_string().as_str(), "john,bob,alice,");
        }

        #[test]
        fn should_render_multi_arg_with_multiple_occurrences_and_equals() {
            before_each();
            let proj_name = "just-multi-arg-multi-occurrences-equals";
            zr(&["new", "just", "multi", proj_name, "--names=john", "--names=bob", "--names=alice"]);
            let file = MockFs::home().join(proj_name).join("each.txt");
            assert_eq!(file.read_to_string().as_str(), "john,bob,alice,");
        }

        #[test]
        fn should_render_multi_arg_with_short() {
            before_each();
            let proj_name = "just-multi-arg-short";
            zr(&["new", "just", "multi", proj_name, "-n", "john,bob,alice"]);
            let file = MockFs::home().join(proj_name).join("each.txt");
            assert_eq!(file.read_to_string().as_str(), "john,bob,alice,");
        }

        #[test]
        fn should_render_multi_arg_with_short_and_multiple_occurrences() {
            before_each();
            let proj_name = "just-multi-arg-short-multiple-occurrences";
            zr(&["new", "just", "multi", proj_name, "-n", "john", "-n", "bob", "-n", "alice"]);
            let file = MockFs::home().join(proj_name).join("each.txt");
            assert_eq!(file.read_to_string().as_str(), "john,bob,alice,");
        }

        #[test]
        fn should_not_render_multi_arg_when_absent() {
            before_each();
            let proj_name = "empty-multi-arg";
            zr(&["new", "just", "multi", proj_name]);
            let file = MockFs::home().join(proj_name).join("each.txt");
            assert!(file.read_to_string().is_empty());
        }

        #[test]
        fn should_unless_placeholder() {
            before_each();
            let proj_name = "just-multi-arg-unless";
            zr(&["new", "just", "multi", proj_name, "--names", "john,bob,alice"]);
            let file = MockFs::home().join(proj_name).join("unless.txt");
            assert_eq!(file.read_to_string().as_str(), "john,bob,alice");
        }

        #[test]
        fn should_duplicate_multi_when_on_folder() {
            before_each();
            let proj_name = "just-duplicate-folder";
            zr(&["new", "just", "multi-duplicate-folder", proj_name, "-m", "api", "-m", "error", "-m", "kafka"]);
            let proj = MockFs::home().join(proj_name);
            assert!(proj.join("api").exists());
            assert!(proj.join("api").join("a.txt").exists());
            assert!(proj.join("error").exists());
            assert!(proj.join("error").join("a.txt").exists());
            assert!(proj.join("kafka").exists());
            assert!(proj.join("kafka").join("a.txt").exists());
        }
    }

    mod cmd {
        use super::*;

        #[test]
        fn should_execute_arg_cmd() {
            before_each();
            let out = zr_output(&["new", "cmd", "std", "list-files", "--recap"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt"));
            assert!(out.lines().any(|it| it.trim() == "b.txt"));
            assert!(out.lines().any(|it| it.trim() == "c.txt"));
        }

        #[test]
        fn cmd_should_not_be_executed_by_default() {
            before_each();
            let out = zr_output(&["new", "cmd", "std", "does-not-list-files"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt").not());
            assert!(out.lines().any(|it| it.trim() == "b.txt").not());
            assert!(out.lines().any(|it| it.trim() == "c.txt").not());
        }

        #[test]
        fn should_execute_arg_cmd_by_default() {
            before_each();
            let out = zr_output(&["new", "cmd", "by-default", "list-files-by-default"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt"));
            assert!(out.lines().any(|it| it.trim() == "b.txt"));
            assert!(out.lines().any(|it| it.trim() == "c.txt"));
        }

        #[test]
        fn should_turn_off_default_cmd() {
            before_each();
            let out = zr_output(&["new", "cmd", "by-default", "does-not-list-files-by-default", "--no-recap"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt").not());
            assert!(out.lines().any(|it| it.trim() == "b.txt").not());
            assert!(out.lines().any(|it| it.trim() == "c.txt").not());
        }

        #[test]
        fn should_execute_explicit_not_default_arg_cmd() {
            before_each();
            let out = zr_output(&["new", "cmd", "not-default", "explicit-not-default", "--recap"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt"));
            assert!(out.lines().any(|it| it.trim() == "b.txt"));
            assert!(out.lines().any(|it| it.trim() == "c.txt"));
        }

        #[test]
        fn explicit_not_default_cmd_should_not_be_executed_by_default() {
            before_each();
            let out = zr_output(&["new", "cmd", "not-default", "explicit-not-default-absent"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt").not());
            assert!(out.lines().any(|it| it.trim() == "b.txt").not());
            assert!(out.lines().any(|it| it.trim() == "c.txt").not());
        }

        #[test]
        fn should_preserve_cmd_order() {
            before_each();
            let out = zr_output(&["new", "cmd", "order", "create-then-list", "--create", "--list"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt"));
            let out = zr_output(&["new", "cmd", "order", "also-create-then-list", "--list", "--create"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt"));
            let out = zr_output(&["new", "cmd", "order-reverse", "list-then-create", "--create", "--list"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt").not());
            let out = zr_output(&["new", "cmd", "order-reverse", "also-list-then-create", "--list", "--create"]);
            assert!(out.lines().any(|it| it.trim() == "a.txt").not());
        }
    }
}