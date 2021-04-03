use std::convert::{TryFrom, TryInto};

use itertools::Itertools;
use serde_json::Value;

/// Given an arg, proposes it with different separators e.g. slash or dot
pub struct ArgSeparator(String, Option<Value>);

impl ArgSeparator {
    const PATH_SUFFIX: &'static str = "path";
    const DOT_SUFFIX: &'static str = "dot";

    pub fn variants(self) -> impl Iterator<Item=(String, Value)> {
        vec![
            (self.key(Self::PATH_SUFFIX), self.value("/", '/')),
            (self.key(Self::DOT_SUFFIX), self.value(".", '.')),
        ].into_iter()
    }

    fn key(&self, suffix: &str) -> String {
        format!("{}-{}", self.0, suffix)
    }

    fn value(&self, separator: &str, sep: char) -> Value {
        self.value_str(separator, sep)
            .or_else(|| self.value_array(separator, sep))
            .unwrap_or_default()
    }

    fn value_str(&self, separator: &str, sep: char) -> Option<Value> {
        self.1.as_ref()
            .and_then(|it| it.as_str())
            .map(|it| Self::transform(it, separator, sep))
            .and_then(|it| Value::try_from(it).ok())
    }

    fn value_array(&self, separator: &str, sep: char) -> Option<Value> {
        self.1.as_ref()
            .and_then(|it| it.as_array())
            .and_then(|values| values.iter()
                .filter_map(|v| v.as_str())
                .map(|s| Self::transform(s, separator, sep))
                .filter_map(|it| Value::try_from(it).ok())
                .collect_vec()
                .try_into().ok())
    }

    fn transform(from: &str, separator: &str, sep: char) -> String {
        from.replace(|c: char| !c.is_ascii_alphanumeric(), separator)
            .chars()
            .coalesce(|a, b| if a == sep && a == b { Ok(sep) } else { Err((a, b)) })
            .join("")
    }
}

impl From<(String, Option<Value>)> for ArgSeparator {
    fn from((k, v): (String, Option<Value>)) -> Self { Self(k, v) }
}

#[cfg(test)]
mod arg_separator_tests {
    use std::{collections::HashMap, iter::FromIterator};

    use super::*;

    mod simple {
        use super::*;

        #[test]
        fn should_contain_path_variant() {
            assert_eq!(variants("a-b_c$d!e'f4g h")["key-path"], Value::from("a/b/c/d/e/f4g/h"));
        }

        #[test]
        fn should_coalesce_path_variant() {
            assert_eq!(variants("a--b__c---d")["key-path"], Value::from("a/b/c/d"));
        }

        #[test]
        fn should_contain_dot_variant() {
            assert_eq!(variants("a-b_c$d!e'f4g h")["key-dot"], Value::from("a.b.c.d.e.f4g.h"));
        }

        #[test]
        fn should_coalesce_dot_variant() {
            assert_eq!(variants("a--b__c---d")["key-dot"], Value::from("a.b.c.d"));
        }

        fn variants(value: &str) -> HashMap<String, Value> {
            let input = (String::from("key"), Some(Value::from(value)));
            HashMap::from_iter(ArgSeparator::from(input).variants())
        }
    }

    mod array {
        use super::*;

        #[test]
        fn multi_should_contain_path_variant() {
            assert_eq!(
                variants(&["a-b_c$d", "d!e'f4g h"])["key-path"],
                Value::from(&["a/b/c/d", "d/e/f4g/h"][..])
            );
        }

        #[test]
        fn multi_should_coalesce_path_variant() {
            assert_eq!(
                variants(&["a--b__c", "b__c---d"])["key-path"],
                Value::from(&["a/b/c", "b/c/d"][..])
            );
        }

        #[test]
        fn multi_should_contain_dot_variant() {
            assert_eq!(
                variants(&["a-b_c$d", "d!e'f4g h"])["key-dot"],
                Value::from(&["a.b.c.d", "d.e.f4g.h"][..])
            );
        }

        #[test]
        fn multi_should_coalesce_dot_variant() {
            assert_eq!(
                variants(&["a--b__c", "b__c---d"])["key-dot"],
                Value::from(&["a.b.c", "b.c.d"][..])
            );
        }

        fn variants(values: &[&str]) -> HashMap<String, Value> {
            let input = (String::from("key"), Some(Value::from(&values[..])));
            HashMap::from_iter(ArgSeparator::from(input).variants())
        }
    }
}