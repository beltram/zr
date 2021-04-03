use itertools::Itertools;
use serde_json::Value;

use crate::data::arg_cmd::ArgCmd;
use crate::template::local_arg::LocalInitializrArg;

use super::{arg_case::ArgCase, arg_path::ArgSeparator};

/// Proposes variants of a given arg e.g. as path, as kebab-case, as snake_case
#[derive(Clone, Debug, Default)]
pub struct DataArg {
    pub variants: Vec<(String, Value)>,
    pub cmd: Option<ArgCmd>,
}

impl DataArg {
    fn original((k, v): (String, Option<Value>)) -> (String, Value) {
        (k, v.unwrap_or_default())
    }

    pub fn is_not_cmd(&self) -> bool {
        self.cmd.is_none()
    }
}

impl From<(String, Option<Value>, Option<LocalInitializrArg>)> for DataArg {
    fn from((k, v, maybe_arg): (String, Option<Value>, Option<LocalInitializrArg>)) -> Self {
        if let Some(cmd) = maybe_arg.as_ref().and_then(|it| it.maybe_cmd()) {
            return Self { cmd: Some(cmd), ..Default::default() };
        } else if let Some(value) = maybe_arg.as_ref().and_then(|it| it.maybe_flag()) {
            return Self { variants: vec![(k, Value::from(value))], ..Default::default() };
        }
        let kv = (k, v);
        let arg_cases = ArgCase::from(kv.clone());
        let original_arg = Self::original(kv.clone());
        let arg_separator = ArgSeparator::from(kv);
        let variants = arg_cases.variants()
            .merge_by(arg_separator.variants().into_iter(), |_, _| true)
            .merge_by(vec![original_arg].into_iter(), |_, _| true)
            .collect_vec();
        Self { variants, ..Default::default() }
    }
}

#[cfg(test)]
mod data_arg_tests {
    use std::collections::BTreeMap;
    use std::iter::FromIterator;

    use super::*;

    #[test]
    fn original_should_not_alter() {
        let input = (String::from("key"), Some(Value::from("value")));
        let (key, value) = DataArg::original(input);
        assert_eq!(key.as_str(), "key");
        assert!(value.is_string());
        assert_eq!(value.as_str(), Some("value"));
    }

    #[test]
    fn original_should_convert_none_into_value_null() {
        let input = (String::new(), None);
        let (_, value) = DataArg::original(input);
        assert!(value.is_null());
    }

    #[test]
    fn should_add_path_variant() {
        let input = (String::from("some-key"), Some(Value::from("a-b-c")), None);
        let variants = BTreeMap::from_iter(DataArg::from(input).variants);
        assert!(variants.contains_key("some-key"));
        assert_eq!(variants.get("some-key").and_then(|it| it.as_str()), Some("a-b-c"));
        assert!(variants.contains_key("some-key-path"));
        assert_eq!(variants.get("some-key-path").and_then(|it| it.as_str()), Some("a/b/c"));
    }
}