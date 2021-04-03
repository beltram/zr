use std::{env, path::PathBuf};

use tempfile::tempdir;

use data::Data;

use crate::utils::error::ErrorExt;
use crate::utils::file::PathExt;
use crate::utils::user::User;

pub mod data;
pub mod cmd;

lazy_static! {
    static ref TEMP_DIR: PathBuf = tempdir().unwrap().into_path();
    pub static ref MOCK_HOME: MockFs = MockFs::init();
}

/// Mocks a user local user filesystem
pub struct MockFs { pub home: PathBuf }

impl<'a> MockFs {
    #[cfg(target_os = "macos")]
    const ZR_PATH: &'static str = "Library/Application Support/rs.zr";
    #[cfg(target_os = "linux")]
    const ZR_PATH: &'static str = ".local/share/zr";
    #[cfg(target_os = "windows")]
    const ZR_PATH: &'static str = "AppData/Roaming/zr/data";

    #[cfg(target_os = "macos")]
    const CONFIG_PATH: &'static str = "Library/Preferences/rs.zr/zr.toml";
    #[cfg(target_os = "linux")]
    const CONFIG_PATH: &'static str = ".config/zr/zr.toml";
    #[cfg(target_os = "windows")]
    const CONFIG_PATH: &'static str = "AppData/Roaming/zr/zr.toml";

    const INITIALIZR_INSTALL_DIR: &'static str = "a875e39e74a89420d95c67576a5969e7fd4007cf296c0d267d624b4582f5ac8e";

    pub fn new() -> &'static Self { &MOCK_HOME }

    pub fn home() -> PathBuf { Self::new().home.clone() }

    pub fn zr() -> PathBuf {
        Self::home().join(PathBuf::from(Self::ZR_PATH))
    }

    pub fn dev() -> PathBuf {
        Self::home()
    }

    pub fn template(name: &str) -> PathBuf {
        Self::zr().join(Self::INITIALIZR_INSTALL_DIR).join(name)
    }

    pub fn config() -> PathBuf {
        Self::home().join(PathBuf::from(Self::CONFIG_PATH))
    }

    fn init() -> Self {
        let temp_home: &PathBuf = &TEMP_DIR;
        Self::copy_mocks(&["common", env::consts::OS], &temp_home);
        Self::copy_zr(&temp_home);
        Self::vars(&temp_home);
        Self { home: temp_home.to_path_buf() }
    }

    /// Copies mock files
    fn copy_mocks(paths: &[&str], temp_home: &PathBuf) {
        paths.iter()
            .flat_map(|it| Self::mocked_home().join(it).read_dir().unwrap())
            .filter(|it| it.is_ok())
            .for_each(|it| it.unwrap().path().copy_all(&temp_home));
    }

    /// Copies all files under /data/home/zr
    fn copy_zr(temp_home: &PathBuf) {
        Self::mocked_home().join("zr")
            .copy_all(&temp_home.join(Self::ZR_PATH));
    }

    fn mocked_home() -> PathBuf { Data::new("home").path() }

    fn vars(temp_home: &PathBuf) {
        env::set_var(User::HOME_VAR, temp_home.path_str());
    }
}

impl Drop for MockFs {
    fn drop(&mut self) {
        self.home.read_dir()
            .fail("Failed reading file under home")
            .filter_map(|it| it.ok())
            .for_each(|it| {
                if it.path().is_dir() {
                    it.path().delete_dir().unwrap()
                } else {
                    it.path().delete().unwrap()
                };
            });
    }
}