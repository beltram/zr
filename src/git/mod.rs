use std::{path::PathBuf, process::Command, str};

use git2::Repository;

use crate::console::command::CommandExt;
use crate::utils::anyhow_err::{ErrConversion, OptConversion};
use crate::utils::error::ErrorExt;
use crate::utils::file::PathExt;

/// Git commands wrapper
#[derive(new)]
pub struct Git { pub repo: Repository }

impl<'a> From<&'a PathBuf> for Git {
    fn from(path: &'a PathBuf) -> Self {
        let repo = Repository::open(path).fail(format!("Failed opening git repository {:?}", path));
        Self { repo }
    }
}

impl From<Repository> for Git {
    fn from(repo: Repository) -> Self { Self { repo } }
}

impl Git {
    /// Clones a remote git repository
    /// * `from_dir` - Performs git clone into this directory
    /// * `uri` - Remote git uri to clone
    /// * `into_dir` - Name of the created folder
    /// * `branch` - Branch to checkout after clone
    pub fn clone(from_dir: &PathBuf, uri: &str, into_dir: &str, branch: Option<&str>, with_history: bool) -> Option<PathBuf> {
        let into_dir = &from_dir.join(into_dir);
        Self::clone_cmd(from_dir, uri, branch, with_history, into_dir)
            .map(|repository| Self::root_path(&Self::new(repository)))
    }

    /// Clones with external git command if credentials not found locally
    fn clone_cmd(from_dir: &PathBuf, uri: &str, branch: Option<&str>, with_history: bool, into_dir: &PathBuf) -> Option<Repository> {
        let dir_name = into_dir.path_str();
        if with_history {
            branch.and_then(|b| Self::cmd(&["clone", uri, "--branch", b, dir_name], from_dir))
                .or_else(|| Self::cmd(&["clone", uri, dir_name], from_dir))
        } else {
            branch.and_then(|b| Self::cmd(&["clone", "--depth", "1", uri, "--branch", b, dir_name], from_dir))
                .or_else(|| Self::cmd(&["clone", "--depth", "1", uri, dir_name], from_dir))
        }
            .and_then(|_| Repository::open(into_dir).ok())
    }

    pub fn pull_rebase(from: &PathBuf) -> anyhow::Result<()> {
        Self::cmd(&["pull", "--rebase"], from)
            .wrap(format!("'git pull --rebase' failed in {:?}", from))
    }

    /// 'git init'
    pub fn init(path: &PathBuf) -> anyhow::Result<Repository> {
        Repository::init(path).wrap()
    }

    pub fn add(&self, file: &PathBuf) {
        self.repo.index().ok()
            .and_then(|mut idx| idx.add_path(file).ok())
            .or_else(|| Self::cmd(&["add", file.path_str()], &self.root_path()));
    }

    fn cmd(args: &[&str], from_dir: &PathBuf) -> Option<()> {
        Command::new("git")
            .current_dir(from_dir)
            .args(args)
            .no_output()
            .spawn_and_wait()
            .map(|_| ())
            .ok()
    }

    /// Helper to execute raw git command in a repository
    /// because implem from git2 returns path to .git and we want parent
    fn root_path(&self) -> PathBuf {
        self.repo.path().parent()
            .fail(format!("Failed getting parent directory for git project {:?}", self.repo.path()))
            .to_path_buf()
    }
}
