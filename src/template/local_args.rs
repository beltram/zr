//! # Initializr flags
//!
//! ## Sample
//! ```
//! # use zr::{utils::marshall::Tomlable, template::{local_args::LocalInitializrArgs, local_arg::LocalInitializrArg, arg_kind::LocalArgKind}};
//! # use std::{collections::BTreeMap, iter::FromIterator};
//! #
//! # let sample = r#"
//! [spring-boot-version]
//! about = 'Spring Boot version'
//! short = 's'
//! long = 'spring-boot-version'
//! kind = { ARG = { default = '2.4.0', possible-values = ['2.4.0', '2.4.1'] } }
//! # "#.to_string();
//! # let spring_boot_version = LocalInitializrArg {
//! #     about: Some(String::from("Spring Boot version")),
//! #     short: Some('s'),
//! #     long: Some(String::from("spring-boot-version")),
//! #     kind: Some(LocalArgKind::ARG {
//! #         default: Some(String::from("2.4.0")),
//! #         possible_values: Some(vec![String::from("2.4.0"), String::from("2.4.1")])
//! #     }),
//! # };
//! # let flags = BTreeMap::from_iter(vec![(String::from("spring-boot-version"), spring_boot_version)]);
//! # assert_eq!(LocalInitializrArgs(flags), LocalInitializrArgs::from_toml(sample));
//! ```
use std::collections::BTreeMap;

use clap::Arg;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::utils::marshall::Tomlable;

use super::{local_arg::LocalInitializrArg, local_arg::NamedArg};

#[derive(Clone, Debug, Serialize, Deserialize, Default, Eq, PartialEq)]
#[serde(transparent)]
pub struct LocalInitializrArgs(pub BTreeMap<String, LocalInitializrArg>);

impl<'a> From<&'a LocalInitializrArgs> for Vec<Arg<'a>> {
    fn from(args: &'a LocalInitializrArgs) -> Self {
        args.0.iter()
            .map(|(name, flag)| NamedArg(name, flag))
            .map(Arg::from)
            .collect_vec()
    }
}

impl Tomlable for LocalInitializrArgs {}

#[cfg(test)]
mod local_args_tests {
    use crate::template::arg_kind::LocalArgKind;

    use super::*;

    #[test]
    fn should_read_args_from_zr_toml() {
        let args = LocalInitializrArgs::from_toml(r#"
        [spring-boot-version]
        about = 'Spring Boot version'
        short = 's'
        long = 'sb-version'
        [spring-cloud-version]
        kind = { ARG = { possible-values = ['2020.0.0', '2020.0.1'] } }
        [kotlin-version]
        kind = { ARG = { default = '1.4.30' } }
        "#);
        assert_eq!(args.0.len(), 3);
        let spring_boot_version = args.0.get("spring-boot-version").unwrap();
        assert_eq!(spring_boot_version.about.as_ref().unwrap().as_str(), "Spring Boot version");
        assert_eq!(spring_boot_version.short.unwrap(), 's');
        assert_eq!(spring_boot_version.long.as_ref().unwrap().as_str(), "sb-version");
        assert_eq!(
            spring_boot_version.kind.as_ref().unwrap(),
            &LocalArgKind::ARG { default: None, possible_values: None }
        );
        let spring_cloud_version = args.0.get("spring-cloud-version").unwrap();
        assert_eq!(spring_cloud_version.kind.as_ref().unwrap(), &LocalArgKind::ARG {
            default: None,
            possible_values: Some(vec![String::from("2020.0.0"), String::from("2020.0.1")]),
        });
        let kotlin_version = args.0.get("kotlin-version").unwrap();
        assert_eq!(
            kotlin_version.kind.as_ref().unwrap(),
            &LocalArgKind::ARG { default: Some(String::from("1.4.30")), possible_values: None }
        );
    }

    #[test]
    fn should_read_multi_from_zr_toml() {
        let args = LocalInitializrArgs::from_toml(r#"
        [spring-boot-version]
        kind = { MULTI = {} }
        [spring-cloud-version]
        kind = { MULTI = { possible-values = ['2020.0.0', '2020.0.1'] } }
        [modules]
        kind = { MULTI = { default = ['api', 'error', 'kafka'] } }
        "#);
        assert_eq!(args.0.len(), 3);
        let spring_boot_version = args.0.get("spring-boot-version").unwrap();
        assert_eq!(spring_boot_version.kind.as_ref().unwrap(), &LocalArgKind::MULTI { default: None, possible_values: None });
        let spring_cloud_version = args.0.get("spring-cloud-version").unwrap();
        assert_eq!(spring_cloud_version.kind.as_ref().unwrap(), &LocalArgKind::MULTI {
            default: None,
            possible_values: Some(vec![String::from("2020.0.0"), String::from("2020.0.1")]),
        });
        let modules = args.0.get("modules").unwrap();
        assert_eq!(modules.kind.as_ref().unwrap(), &LocalArgKind::MULTI {
            default: Some(vec![String::from("api"), String::from("error"), String::from("kafka")]),
            possible_values: None,
        });
    }

    #[test]
    fn should_read_flag_from_zr_toml() {
        let args = LocalInitializrArgs::from_toml(r#"
        [with-kafka]
        about = 'use kafka'
        kind = { FLAG = { negate = true } }
        [just-flag]
        kind = { FLAG = {} }
        "#);
        assert_eq!(args.0.len(), 2);
        let with_kafka = args.0.get("with-kafka").unwrap();
        assert_eq!(with_kafka.about.as_ref().unwrap().as_str(), "use kafka");
        assert_eq!(with_kafka.kind.as_ref().unwrap(), &LocalArgKind::FLAG { negate: Some(true) });
        let just_flag = args.0.get("just-flag").unwrap();
        assert_eq!(just_flag.kind.as_ref().unwrap(), &LocalArgKind::FLAG { negate: None });
    }

    #[test]
    fn kind_should_default_to_arg() {
        let args = LocalInitializrArgs::from_toml(r#"
        [spring-boot-version]
        "#);
        assert_eq!(args.0.len(), 1);
        let spring_boot_version = args.0.get("spring-boot-version").unwrap();
        assert_eq!(spring_boot_version.kind.as_ref().unwrap(), &LocalArgKind::ARG { default: None, possible_values: None });
    }

    #[test]
    fn should_cmd_from_zr_toml() {
        let args = LocalInitializrArgs::from_toml(r#"
        [gradle-wrapper]
        about = 'Init a Gradle wrapper'
        kind = { CMD = { cmd = 'gradle wrapper' } }
        [by-default]
        kind = { CMD = { default = true, cmd = 'ls .' } }
        [not-default]
        kind = { CMD = { default = false, cmd = 'ls .' } }
        [with-order]
        kind = { CMD = { order = 1, cmd = 'ls .' } }
        "#);
        assert_eq!(args.0.len(), 4);
        let gradle_wrapper = args.0.get("gradle-wrapper").unwrap();
        assert_eq!(gradle_wrapper.about.as_ref().unwrap(), "Init a Gradle wrapper");
        assert_eq!(gradle_wrapper.kind.as_ref().unwrap(), &LocalArgKind::CMD { order: None, default: None, cmd: String::from("gradle wrapper") });
        let by_default = args.0.get("by-default").unwrap();
        assert_eq!(by_default.kind.as_ref().unwrap(), &LocalArgKind::CMD { order: None, default: Some(true), cmd: String::from("ls .") });
        let not_default = args.0.get("not-default").unwrap();
        assert_eq!(not_default.kind.as_ref().unwrap(), &LocalArgKind::CMD { order: None, default: Some(false), cmd: String::from("ls .") });
        let with_order = args.0.get("with-order").unwrap();
        assert_eq!(with_order.kind.as_ref().unwrap(), &LocalArgKind::CMD { order: Some(1), default: None, cmd: String::from("ls .") });
    }
}