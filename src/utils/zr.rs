use std::path::PathBuf;

use directories::ProjectDirs;

use crate::utils::file::PathExt;

pub struct Zr {}

impl Zr {
    pub const NAME: &'static str = "zr";
    const QUALIFIER: &'static str = "rs";

    /// Finds user's zr home directory
    pub fn home() -> Option<PathBuf> {
        cached_zr_home()
            .map(|it| it.data_dir().to_path_buf())
            .map(|data_dir| {
                if !data_dir.exists() { data_dir.create_dir_all_or_fail() }
                data_dir
            })
    }

    pub fn zr_home() -> Option<ProjectDirs> {
        ProjectDirs::from(Self::QUALIFIER, "", Self::NAME)
    }
}

cached! {
    ZR_HOME;
    fn cached_zr_home() -> Option<ProjectDirs> = { Zr::zr_home() }
}

#[cfg(test)]
pub mod app_tests {
    use crate::mocks::MockFs;

    use super::*;

    #[test]
    fn should_find_zr_home() {
        assert_eq!(Zr::home().unwrap(), MockFs::zr());
    }
}