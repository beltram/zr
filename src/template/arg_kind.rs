use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum LocalArgKind {
    /// A boolean arg, taking no value e.g. '--force'
    #[serde(rename_all = "kebab-case")]
    FLAG {
        negate: Option<bool>,
    },
    /// An arg, taking value e.g. '--gradle-version=7.0'
    #[serde(rename_all = "kebab-case")]
    ARG {
        default: Option<String>,
        possible_values: Option<Vec<String>>,
    },
    /// An arg having multiple occurrences e.g. '--mod=api --mod=error'
    #[serde(rename_all = "kebab-case")]
    MULTI {
        default: Option<Vec<String>>,
        possible_values: Option<Vec<String>>,
    },
    /// A command execute after project creation
    #[serde(rename_all = "kebab-case")]
    CMD {
        order: Option<u8>,
        default: Option<bool>,
        cmd: String,
    },
}

impl Default for LocalArgKind {
    fn default() -> Self {
        Self::ARG { default: None, possible_values: None }
    }
}