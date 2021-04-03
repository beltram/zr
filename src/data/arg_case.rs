use std::convert::TryInto;

use inflections::Inflect;
use itertools::Itertools;
use serde_json::Value;

/// Given an arg, translates its value to many case (kebab, camel etc..) variants
/// Its key will be suffixed with case name
pub struct ArgCase(String, Option<Value>);

impl ArgCase {
    const UPPER_SUFFIX: &'static str = "upper";
    const LOWER_SUFFIX: &'static str = "lower";
    const SENTENCE_SUFFIX: &'static str = "sentence";
    const TITLE_SUFFIX: &'static str = "title";
    const CAMEL_SUFFIX: &'static str = "camel";
    const PASCAL_SUFFIX: &'static str = "pascal";
    const KEBAB_SUFFIX: &'static str = "kebab";
    const TRAIN_SUFFIX: &'static str = "train";
    const SNAKE_SUFFIX: &'static str = "snake";
    const CONSTANT_SUFFIX: &'static str = "constant";

    pub fn variants(self) -> impl Iterator<Item=(String, Value)> {
        vec![
            (self.key(Self::UPPER_SUFFIX), self.value(Inflect::to_upper_case)),
            (self.key(Self::LOWER_SUFFIX), self.value(Inflect::to_lower_case)),
            (self.key(Self::SENTENCE_SUFFIX), self.value(Inflect::to_sentence_case)),
            (self.key(Self::TITLE_SUFFIX), self.value(Inflect::to_title_case)),
            (self.key(Self::CAMEL_SUFFIX), self.value(Inflect::to_camel_case)),
            (self.key(Self::PASCAL_SUFFIX), self.value(Inflect::to_pascal_case)),
            (self.key(Self::KEBAB_SUFFIX), self.value(Inflect::to_kebab_case)),
            (self.key(Self::TRAIN_SUFFIX), self.value(Inflect::to_train_case)),
            (self.key(Self::SNAKE_SUFFIX), self.value(Inflect::to_snake_case)),
            (self.key(Self::CONSTANT_SUFFIX), self.value(Inflect::to_constant_case)),
        ].into_iter()
    }

    fn key(&self, suffix: &str) -> String {
        format!("{}-{}", self.0, suffix)
    }

    fn value(&self, case: fn(&str) -> String) -> Value {
        self.value_str(case)
            .or_else(|| self.value_array(case))
            .unwrap_or_default()
    }

    fn value_str(&self, case: fn(&str) -> String) -> Option<Value> {
        self.1.as_ref()
            .and_then(|it| it.as_str())
            .map(case)
            .map(Value::String)
    }

    fn value_array(&self, case: fn(&str) -> String) -> Option<Value> {
        self.1.as_ref()
            .and_then(|it| it.as_array())
            .and_then(|v| {
                v.iter()
                    .filter_map(|it| it.as_str())
                    .map(case)
                    .collect_vec()
                    .try_into().ok()
            })
    }
}

impl From<(String, Option<Value>)> for ArgCase {
    fn from((k, v): (String, Option<Value>)) -> Self { Self(k, v) }
}

#[cfg(test)]
mod arg_case_tests {
    use std::{collections::HashMap, iter::FromIterator};

    use super::*;

    mod simple {
        use super::*;

        #[test]
        fn should_contain_upper_case_variant() {
            assert_eq!(variants("helloworld")["key-upper"], Value::from("HELLOWORLD"))
        }

        #[test]
        fn should_contain_lower_case_variant() {
            assert_eq!(variants("HELLOWORLD")["key-lower"], Value::from("helloworld"))
        }

        #[test]
        fn should_contain_sentence_case_variant() {
            assert_eq!(variants("helloWorld")["key-sentence"], Value::from("hello world"))
        }

        #[test]
        fn should_contain_title_case_variant() {
            assert_eq!(variants("hello world")["key-title"], Value::from("Hello World"))
        }

        #[test]
        fn should_contain_camel_case_variant() {
            assert_eq!(variants("hello world")["key-camel"], Value::from("helloWorld"))
        }

        #[test]
        fn should_contain_pascal_case_variant() {
            assert_eq!(variants("hello world")["key-pascal"], Value::from("HelloWorld"))
        }

        #[test]
        fn should_contain_kebab_case_variant() {
            assert_eq!(variants("hello world")["key-kebab"], Value::from("hello-world"))
        }

        #[test]
        fn should_contain_train_case_variant() {
            assert_eq!(variants("hello world")["key-train"], Value::from("Hello-World"))
        }

        #[test]
        fn should_contain_snake_case_variant() {
            assert_eq!(variants("hello world")["key-snake"], Value::from("hello_world"))
        }

        #[test]
        fn should_contain_constant_case_variant() {
            assert_eq!(variants("hello world")["key-constant"], Value::from("HELLO_WORLD"))
        }

        fn variants(value: &str) -> HashMap<String, Value> {
            let input = (String::from("key"), Some(Value::from(value)));
            HashMap::from_iter(ArgCase::from(input).variants())
        }
    }

    mod array {
        use super::*;

        #[test]
        fn should_contain_upper_case_variant() {
            assert_eq!(variants(&["helloworld"])["key-upper"], Value::from(&["HELLOWORLD"][..]))
        }

        #[test]
        fn should_contain_lower_case_variant() {
            assert_eq!(variants(&["HELLOWORLD"])["key-lower"], Value::from(&["helloworld"][..]))
        }

        #[test]
        fn should_contain_sentence_case_variant() {
            assert_eq!(variants(&["helloWorld"])["key-sentence"], Value::from(&["hello world"][..]))
        }

        #[test]
        fn should_contain_title_case_variant() {
            assert_eq!(variants(&["hello world"])["key-title"], Value::from(&["Hello World"][..]))
        }

        #[test]
        fn should_contain_camel_case_variant() {
            assert_eq!(variants(&["hello world"])["key-camel"], Value::from(&["helloWorld"][..]))
        }

        #[test]
        fn should_contain_pascal_case_variant() {
            assert_eq!(variants(&["hello world"])["key-pascal"], Value::from(&["HelloWorld"][..]))
        }

        #[test]
        fn should_contain_kebab_case_variant() {
            assert_eq!(variants(&["hello world"])["key-kebab"], Value::from(&["hello-world"][..]))
        }

        #[test]
        fn should_contain_train_case_variant() {
            assert_eq!(variants(&["hello world"])["key-train"], Value::from(&["Hello-World"][..]))
        }

        #[test]
        fn should_contain_snake_case_variant() {
            assert_eq!(variants(&["hello world"])["key-snake"], Value::from(&["hello_world"][..]))
        }

        #[test]
        fn should_contain_constant_case_variant() {
            assert_eq!(variants(&["hello world"])["key-constant"], Value::from(&["HELLO_WORLD"][..]))
        }

        fn variants(values: &[&str]) -> HashMap<String, Value> {
            let input = (String::from("key"), Some(Value::from(&values[..])));
            HashMap::from_iter(ArgCase::from(input).variants())
        }
    }
}