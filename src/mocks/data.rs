use std::{fs::File, path::PathBuf};

use serde::de::DeserializeOwned;
use serde_json::from_reader;

use crate::{ErrorExt, PathExt};

#[derive(new)]
pub struct Data<'a> { path: &'a str }

impl Data<'_> {
    /// Loads and deserialize test data
    pub fn data<T: DeserializeOwned>(&self) -> T {
        self.path()
            .open_read()
            .and_then(|it| from_reader::<File, T>(it).ok())
            .unexpected_failure()
    }

    /// Locates test data at given path
    pub fn path(&self) -> PathBuf {
        PathBuf::from(Self::env())
            .join(PathBuf::from("tests/data"))
            .join(self.path)
    }

    pub fn env() -> &'static str { env!("CARGO_MANIFEST_DIR") }
}