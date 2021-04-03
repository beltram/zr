use std::path::PathBuf;

use colored::Colorize;
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::config::hash::ConfigHash;
use crate::upgrade::upgradable::Upgradable;
use crate::utils::error::ErrorExt;
use crate::utils::file::PathExt;
use crate::utils::marshall::Tomlable;
use crate::utils::zr::Zr;

cached! {
    CONFIG;
    fn cached_config() -> Config = { Config::load_or_create() }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct Config {
    // pub repositories: Option<RepoConfig>,
    pub repositories: Option<Vec<String>>,
}

impl Config {
    const CONFIG_FILE_NAME: &'static str = "config.toml";

    /// Memoized config file retrieval
    pub fn get() -> Config { cached_config() }

    /// Loads config from local file system or creates it
    pub fn load_or_create() -> Self {
        confy::load::<Config>(Zr::NAME)
            .fail("Failed loading zr config file")
    }

    pub fn path() -> Option<PathBuf> {
        Zr::zr_home()
            .map(|it| it.config_dir().join(format!("{}.toml", Zr::NAME)))
    }

    /// Get remote configuration
    pub fn remote_configs(&self) -> Vec<Config> {
        Self::urls(self).iter()
            .filter_map(|url| {
                Self::find(url.config_hash())
                    .else_warn(format!("Missing local config for '{}'. Do 'zr upgrade' to update it", url.as_str().bold()))
            })
            .map(|it| it.join(Self::CONFIG_FILE_NAME))
            .filter(|it| it.exists())
            .map(Config::from_file_or_fail)
            .collect_vec()
    }

    pub fn find_template(&self, name: &str) -> Option<PathBuf> {
        let all_temp = self.all_templates();
        all_temp.into_iter()
            .filter_map(|it| it.find(name))
            .take(1).next()
    }

    pub fn all_templates(&self) -> Vec<PathBuf> {
        self.repositories.as_ref()
            .map(|repos| {
                repos.iter()
                    .filter_map(|url| Self::find(url.config_hash()))
                    .collect_vec()
            }).unwrap_or_default()
    }
}

impl Upgradable for Config {
    const INSTALL_DIR: &'static str = "zr";
    const NAME: &'static str = "zr remote configuration";

    fn urls(config: &Config) -> Vec<String> {
        config.to_owned().repositories.unwrap_or_default()
    }
}

impl Tomlable for Config {}


#[cfg(test)]
impl From<Vec<&str>> for Config {
    fn from(uris: Vec<&str>) -> Self {
        Self { repositories: Some(uris.iter().map(|it| it.to_string()).collect_vec()) }
    }
}


#[cfg(test)]
pub mod config_tests {
    use std::ops::Not;

    use crate::{mocks::MockFs, PathExt};

    use super::*;

    fn before_all() {
        MockFs::new();
    }

    #[test]
    fn should_find_config() {
        before_all();
        Config::load_or_create();
        assert_eq!(Config::path().unwrap(), MockFs::config());
    }

    #[test]
    fn should_create_config() {
        before_all();
        MockFs::config().delete().unwrap();
        assert!(MockFs::config().exists().not());
        Config::load_or_create();
        assert_eq!(Config::path().unwrap(), MockFs::config());
        assert!(MockFs::config().exists());
    }

    #[test]
    fn default_config_should_contain_only_none() {
        assert!(Config::default().to_toml().is_empty())
    }

    mod repositories_config_tests {
        use crate::mocks::MockFs;

        use super::*;

        static DEFAULT_REMOTE: &'static str = "https://github.com/beltram/zr-test.git";
        static UNKNOWN_REMOTE: &'static str = "https://github.com/beltram/not-zr-test.git";

        fn before_all() {
            MockFs::new();
        }

        #[test]
        fn all_initializrs_should_find_one() {
            before_all();
            let config = Config::from(vec![DEFAULT_REMOTE]);
            assert_eq!(config.all_templates().len(), 1);
        }

        #[test]
        fn find_template_should_find_one() {
            before_all();
            let config = Config::from(vec![DEFAULT_REMOTE]);
            assert_eq!(config.find_template("rust-app"), Some(MockFs::template("rust-app")));
        }

        #[test]
        fn find_template_should_not_find_any_when_unknown_template_name() {
            before_all();
            let config = Config::from(vec![DEFAULT_REMOTE]);
            assert!(config.find_template("unknown-unknown").is_none());
        }

        #[test]
        fn find_template_should_not_find_any_when_not_in_remote() {
            before_all();
            let config = Config::from(vec![UNKNOWN_REMOTE]);
            assert!(config.find_template("rust-app").is_none());
        }
    }
}