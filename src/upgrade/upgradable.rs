use std::path::PathBuf;

use colored::Colorize;

use crate::{config::hash::ConfigHash, OptConversion};
use crate::utils::zr::Zr;
use crate::utils::error::ErrorExt;
use crate::utils::file::PathExt;
use crate::config::global::Config;
use crate::git::Git;

pub trait Upgradable {
    const INSTALL_DIR: &'static str;
    const NAME: &'static str;

    fn upgrade(config: Config) {
        info!("Updating {}", Self::NAME.green());
        Self::urls(&config).iter()
            .for_each(|url| Self::update_one(url, url.config_hash()))
    }

    fn urls(config: &Config) -> Vec<String>;

    fn update_one(url: &str, hash: String) {
        Self::install_dir()
            .wrap("Could not find installation dir")
            .map(|from| (from.join(hash), from))
            .and_then(|(into, from)| {
                if into.exists() {
                    Self::pull_rebase(&into)
                } else {
                    Self::install(&from, &into, url)
                }
            })
            .warn(format!("Could not update from {}", url))
    }

    fn install(path: &PathBuf, into_dir: &PathBuf, url: &str) -> anyhow::Result<()> {
        debug!("Installing {} from {}", Self::NAME, url);
        into_dir.create_dir()
            .and_then(|_| into_dir.to_str().wrap("Failed acquiring target clone directory"))
            .and_then(|into| {
                Git::clone(path, url, into, None, false)
                    .wrap(format!("Failed cloning {} into {:?}", url, into))
                    .map(|_| ())
            })
    }

    fn pull_rebase(path: &PathBuf) -> anyhow::Result<()> {
        debug!("Updating {} at {:?}", Self::NAME, path);
        Git::pull_rebase(path)
    }

    fn find(hash: String) -> Option<PathBuf> {
        Zr::home()
            .map(|it| it.join(hash))
            .filter(|it| it.exists())
    }

    fn install_dir() -> Option<PathBuf> {
        Zr::home()
            .map(|it| it.join(Self::INSTALL_DIR))
            .map(|it| {
                if !it.exists() { it.create_dir_all_or_fail() }
                it
            })
    }
}