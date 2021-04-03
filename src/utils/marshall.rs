use std::{fmt::Debug, path::{Path, PathBuf}};

use serde::{Deserialize, Serialize};

use crate::utils::anyhow_err::ErrConversion;
use crate::utils::error::ErrorExt;
use crate::utils::file::PathExt;

pub trait Tomlable where for<'de> Self: Deserialize<'de> + Serialize + Debug {
    fn from_toml<T: AsRef<str>>(content: T) -> Self {
        toml::from_str(content.as_ref())
            .fail(format!("Failed deserializing {:#?} from toml", content.as_ref()))
    }

    fn from_file_or_fail<T: AsRef<Path>>(path: T) -> Self {
        toml::from_slice(path.as_ref().read_binary().as_slice())
            .fail(format!("Failed reading toml from file {:?}", path.as_ref()))
    }

    fn from_file<T: AsRef<Path>>(path: T) -> anyhow::Result<Self> {
        toml::from_slice(path.as_ref().read_binary().as_slice()).wrap()
    }

    fn to_toml(&self) -> String {
        toml::to_string_pretty(self)
            .fail(format!("Failed serializing {:?} to toml", self))
    }

    fn to_file(&self, path: &PathBuf) { path.write_to(&self.to_toml()) }
}