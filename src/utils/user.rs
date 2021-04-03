use std::{env, path::PathBuf};

use directories::UserDirs;

/// Represents current user using the application
pub struct User {}

impl User {
    #[cfg(unix)]
    pub const HOME_VAR: &'static str = "HOME";
    #[cfg(windows)]
    pub const HOME_VAR: &'static str = "HOMEPATH";

    /// Finds user's home directory
    pub fn home() -> Option<PathBuf> { cached_home() }

    /// Retrieves path to user home dir
    fn user_home() -> Option<PathBuf> {
        env::var(Self::HOME_VAR).ok()
            .map(PathBuf::from)
            .or_else(|| UserDirs::new().map(|u| u.home_dir().to_path_buf()))
    }
}

cached! {
    USER_HOME;
    fn cached_home() -> Option<PathBuf> = { User::user_home() }
}

#[cfg(test)]
pub mod user_tests {
    use crate::mocks::MockFs;

    use super::*;

    #[test]
    fn should_find_home() {
        assert_eq!(User::home().unwrap(), MockFs::home());
    }
}