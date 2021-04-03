use clap::{Arg, ArgSettings};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use LocalArgKind::{ARG, CMD, FLAG, MULTI};

use super::{arg_kind::LocalArgKind, super::data::arg_cmd::ArgCmd};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(default)]
pub struct LocalInitializrArg {
    pub about: Option<String>,
    pub short: Option<char>,
    pub long: Option<String>,
    pub kind: Option<LocalArgKind>,
}

impl LocalInitializrArg {
    pub fn is_default(&self) -> bool {
        let is_default_cmd = self.maybe_cmd().map(|it| it.is_default).unwrap_or_default();
        let is_default_flag = self.is_negate_flag();
        is_default_cmd || is_default_flag
    }

    pub fn is_multi(&self) -> bool {
        if let Some(MULTI { .. }) = self.kind.as_ref() {
            return true;
        }
        false
    }

    pub fn maybe_cmd(&self) -> Option<ArgCmd> {
        if let Some(CMD { order, default, cmd, .. }) = self.kind.as_ref() {
            return Some(ArgCmd {
                cmd: cmd.to_string(),
                order: order.unwrap_or_default(),
                is_default: default.unwrap_or_default(),
            });
        }
        None
    }

    pub fn maybe_flag(&self) -> Option<bool> {
        if let Some(FLAG { .. }) = self.kind.as_ref() {
            return Some(true);
        }
        None
    }

    fn is_negate_flag(&self) -> bool {
        if let Some(FLAG { negate }) = self.kind.as_ref() {
            return negate.unwrap_or_default();
        }
        false
    }
}

impl Default for LocalInitializrArg {
    fn default() -> Self {
        Self {
            about: None,
            short: None,
            long: None,
            kind: Some(LocalArgKind::default()),
        }
    }
}

pub(crate) struct NamedArg<'a>(pub(crate) &'a str, pub(crate) &'a LocalInitializrArg);

impl NamedArg<'_> {
    const CMD_HELP_HEADING: &'static str = "COMMANDS";

    fn flatten(of: &Option<Vec<String>>) -> Option<Vec<&str>> {
        of.as_ref().map(|it| it.iter().map(|i| i.as_str()).collect_vec())
    }

    fn map_kind<'a>(mut arg: Arg<'a>, kind: &'a LocalArgKind) -> Arg<'a> {
        match kind {
            FLAG { .. } => arg = arg.required(false).takes_value(false),
            ARG { default, possible_values } => {
                arg = arg.takes_value(true);
                if let Some(default) = default {
                    arg = arg.default_value(default.as_str())
                }
                if let Some(possible_values) = NamedArg::flatten(possible_values) {
                    arg = arg.possible_values(possible_values.as_slice())
                }
            }
            MULTI { default, possible_values } => {
                arg = arg
                    .takes_value(true)
                    .multiple(true)
                    .multiple_occurrences(true)
                    .setting(ArgSettings::RequireDelimiter);
                if let Some(default) = NamedArg::flatten(default) {
                    arg = arg.default_values(default.as_slice())
                }
                if let Some(possible_values) = NamedArg::flatten(possible_values) {
                    arg = arg.possible_values(possible_values.as_slice())
                }
            }
            CMD { .. } => {
                arg = arg
                    .required(false)
                    .takes_value(false)
                    .help_heading(Some(NamedArg::CMD_HELP_HEADING))
            }
        }
        arg
    }
}

impl<'a> From<NamedArg<'a>> for Arg<'a> {
    fn from(NamedArg(name, flag): NamedArg<'a>) -> Self {
        let mut arg = Arg::new(name);
        if let Some(about) = flag.about.as_ref() {
            arg = arg.about(about.as_str());
        }
        if let Some(&short) = flag.short.as_ref() {
            arg = arg.short(short);
        }
        if let Some(long) = flag.long.as_ref() {
            arg = arg.long(long.as_str());
        } else {
            arg = arg.long(name);
        }
        if let Some(kind) = flag.kind.as_ref() {
            arg = NamedArg::map_kind(arg, kind)
        }
        arg
    }
}

#[cfg(test)]
mod local_arg_tests {
    use clap::App;
    use itertools::Itertools;

    use super::*;

    mod std {
        use super::*;

        #[test]
        fn should_apply_name() {
            let arg = LocalInitializrArg::default();
            let arg = Arg::from(NamedArg("the-name", &arg));
            assert_eq!(arg.get_name(), "the-name");
        }

        #[test]
        fn should_apply_about() {
            let arg = LocalInitializrArg { about: Some(String::from("about this")), ..Default::default() };
            let arg = Arg::from(NamedArg("", &arg));
            assert_eq!(arg.get_about(), Some("about this"));
        }

        #[test]
        fn should_apply_short() {
            let arg = LocalInitializrArg { short: Some('s'), ..Default::default() };
            let arg = Arg::from(NamedArg("", &arg));
            assert_eq!(arg.get_short(), Some('s'));
        }

        #[test]
        fn should_apply_long() {
            let arg = LocalInitializrArg { long: Some(String::from("bla-bla")), ..Default::default() };
            let arg = Arg::from(NamedArg("", &arg));
            assert_eq!(arg.get_long(), Some("bla-bla"));
        }

        #[test]
        fn long_should_default_to_name() {
            let arg = LocalInitializrArg { long: None, ..Default::default() };
            let arg = Arg::from(NamedArg("bla-bla", &arg));
            assert_eq!(arg.get_long(), Some("bla-bla"));
        }

        #[test]
        fn should_default_to_arg() {
            let arg = LocalInitializrArg::default();
            let arg = Arg::from(NamedArg("opt", &arg));
            let app = App::new("prog").arg(arg);
            assert!(app.try_get_matches_from(vec!["prog", "--opt", "value"]).is_ok());
        }
    }

    mod arg {
        use super::*;

        #[test]
        fn should_not_be_required_by_default() {
            let arg = LocalInitializrArg {
                kind: Some(ARG { default: None, possible_values: None }),
                ..Default::default()
            };
            let arg = Arg::from(NamedArg("opt", &arg));
            let app = App::new("prog").arg(arg);
            assert!(app.try_get_matches_from(vec!["prog"]).is_ok());
        }

        #[test]
        fn should_not_require_equals() {
            let arg = LocalInitializrArg {
                kind: Some(ARG { default: None, possible_values: None }),
                ..Default::default()
            };
            let arg = Arg::from(NamedArg("opt", &arg));
            let app = App::new("prog").arg(arg);
            assert!(app.try_get_matches_from(vec!["prog", "--opt", "val"]).is_ok());
        }

        #[test]
        fn should_apply_default_value() {
            let arg = LocalInitializrArg {
                kind: Some(ARG {
                    default: Some(String::from("abba")),
                    possible_values: None,
                }),
                ..Default::default()
            };
            let arg = Arg::from(NamedArg("opt", &arg));
            let app = App::new("prog").arg(arg);
            let matches = app.get_matches_from(vec!["prog"]);
            assert_eq!(matches.value_of("opt"), Some("abba"));
        }

        #[test]
        fn should_apply_possible_values() {
            let arg = LocalInitializrArg {
                kind: Some(ARG {
                    default: None,
                    possible_values: Some(vec![String::from("a"), String::from("b")]),
                }),
                ..Default::default()
            };
            let arg = Arg::from(NamedArg("opt", &arg));
            let app = App::new("prog").arg(arg);
            assert!(app.clone().try_get_matches_from(vec!["prog", "--opt", "a"]).is_ok());
            assert!(app.clone().try_get_matches_from(vec!["prog", "--opt", "b"]).is_ok());
            assert!(app.try_get_matches_from(vec!["prog", "--opt", "c"]).is_err());
        }
    }

    mod flag {
        use super::*;

        #[test]
        fn flag_should_not_be_required() {
            let flag = LocalInitializrArg { kind: Some(FLAG { negate: None }), ..Default::default() };
            let arg = Arg::from(NamedArg("f", &flag));
            let app = App::new("prog").arg(arg);
            assert!(app.try_get_matches_from(vec!["prog"]).is_ok());
        }

        #[test]
        fn flag_should_take_no_value() {
            let flag = LocalInitializrArg { kind: Some(FLAG { negate: None }), ..Default::default() };
            let arg = Arg::from(NamedArg("f", &flag));
            let app = App::new("prog").arg(arg);
            assert!(app.try_get_matches_from(vec!["prog", "--f", "null"]).is_err());
        }
    }

    mod cmd {
        use super::*;

        #[test]
        fn should_not_be_required_and_take_no_value_when_kind_cmd() {
            let cmd = LocalInitializrArg { kind: Some(CMD { order: None, default: None, cmd: String::new() }), ..Default::default() };
            let arg = Arg::from(NamedArg("gradle", &cmd));
            let app = App::new("prog").arg(arg);
            // for required
            assert!(app.clone().try_get_matches_from(vec!["prog"]).is_ok());
            // for takes no value
            assert!(app.try_get_matches_from(vec!["prog", "--gradle", "null"]).is_err());
        }

        #[test]
        fn cmd_should_have_help_heading() {
            let cmd = LocalInitializrArg { kind: Some(CMD { order: None, default: None, cmd: String::new() }), ..Default::default() };
            let arg = Arg::from(NamedArg("gradle", &cmd));
            assert_eq!(arg.get_help_heading(), Some(NamedArg::CMD_HELP_HEADING))
        }
    }

    mod multi {
        use super::*;

        #[test]
        fn multi_should_not_be_required() {
            let multi = LocalInitializrArg { kind: Some(MULTI { default: None, possible_values: None }), ..Default::default() };
            let arg = Arg::from(NamedArg("m", &multi));
            let app = App::new("prog").arg(arg);
            assert!(app.try_get_matches_from(vec!["prog"]).is_ok());
        }

        #[test]
        fn multi_should_be_repeatable() {
            let multi = LocalInitializrArg { kind: Some(MULTI { default: None, possible_values: None }), ..Default::default() };
            let arg = Arg::from(NamedArg("m", &multi));
            let app = App::new("prog").arg(arg);
            assert_eq!(
                app.clone().get_matches_from(&["prog", "--m", "a", "--m", "b", "--m", "c"]).values_of("m").unwrap().collect_vec(),
                vec!["a", "b", "c"]
            );
            assert_eq!(
                app.get_matches_from(&["prog", "--m", "a,b,c"]).values_of("m").unwrap().collect_vec(),
                vec!["a", "b", "c"]
            );
        }

        #[test]
        fn multi_should_apply_possible_values() {
            let multi = LocalInitializrArg {
                kind: Some(MULTI {
                    default: None,
                    possible_values: Some(vec![String::from("a"), String::from("b")]),
                }),
                ..Default::default()
            };
            let arg = Arg::from(NamedArg("m", &multi));
            let app = App::new("prog").arg(arg);
            assert!(app.clone().try_get_matches_from(vec!["prog", "--m", "a", "--m", "b"]).is_ok());
            assert!(app.clone().try_get_matches_from(vec!["prog", "--m", "a"]).is_ok());
            assert!(app.clone().try_get_matches_from(vec!["prog", "--m", "b"]).is_ok());
            assert!(app.clone().try_get_matches_from(vec!["prog", "--m", "a", "--m", "c"]).is_err());
            assert!(app.clone().try_get_matches_from(vec!["prog", "--m", "b", "--m", "c"]).is_err());
            assert!(app.try_get_matches_from(vec!["prog", "--m", "c"]).is_err());
        }

        #[test]
        fn multi_should_apply_default_values() {
            let multi = LocalInitializrArg {
                kind: Some(MULTI {
                    default: Some(vec![String::from("api"), String::from("error")]),
                    possible_values: None,
                }),
                ..Default::default()
            };
            let arg = Arg::from(NamedArg("m", &multi));
            let app = App::new("prog").arg(arg);
            assert_eq!(
                app.get_matches_from(vec!["prog"]).values_of("m").unwrap().into_iter().collect_vec(),
                vec!["api", "error"]
            );
        }
    }
}