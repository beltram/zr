use std::{env, path::PathBuf, process::Command};

use colored::Colorize;
use git2::Repository;

use crate::{completion::model::Shell, ErrConversion};
use crate::console::command::CommandExt;
use crate::utils::error::ErrorExt;
use crate::utils::file::PathExt;

use super::data::InitializrData;
use crate::git::Git;

/// Represents actions that can be executed on a project after it has been bootstrapped.
/// Actions can be triggered by command line flags/args or by global configuration
#[derive(new)]
pub struct ProjectActions<'a> {
    pub path: &'a PathBuf,
    pub data: &'a InitializrData,
}

impl<'a> ProjectActions<'a> {
    const GITIGNORE: &'static str = ".gitignore";

    /// Do optional stuffs after project bootstrapped
    pub fn apply(&self) -> anyhow::Result<()> {
        env::set_current_dir(self.path)?;
        self.git_init()
            .map(|r| self.git_add_ignore(r))
            .warn(format!("Failed initializing git in {}", format!("{:?}", self.path).as_str().yellow()));
        self.then_execute_commands();
        self.idea_open();
        self.vs_code_open();
        self.erase_if_dry()
    }

    fn git_init(&self) -> anyhow::Result<Repository> {
        Git::init(self.path)
            .then(|_| info!("Git initialized in {}", format!("{:?}", self.path).as_str().cyan()))
            .wrap()
    }

    // Adds '.gitignore' if it exists. Helps when IDE adds all file when opening
    fn git_add_ignore(&self, repo: Repository) {
        let gitignore = self.path.join(Self::GITIGNORE);
        if gitignore.exists() {
            Git::from(repo).add(&gitignore);
            debug!("Added {} to git index", format!("{:?}", &gitignore).as_str().cyan());
        }
    }

    fn idea_open(&self) {
        if self.data.is_idea() {
            info!("Opening in IntelliJ with 'idea' cli in {}", format!("{:?}", self.path).as_str().cyan());
            Command::new("idea")
                .arg(self.path)
                .no_output()
                .spawn_and_wait()
                .warn(format!("Failed opening {} in IntelliJ", format!("{:?}", self.path).as_str().yellow()))
        }
    }

    fn vs_code_open(&self) {
        if self.data.is_vs_code() {
            debug!("Now opening {} in VsCode", format!("{:?}", self.path).as_str().cyan());
            Shell::run("code .", self.path)
                .warn(format!("Failed opening {} in visual studio code", format!("{:?}", self.path).as_str().yellow()));
        }
    }

    fn then_execute_commands(&self) {
        self.data.commands.iter().for_each(|cmd| {
            info!("Executing '{}' in {}", cmd.as_str().cyan(), format!("{:?}", self.path).as_str().cyan());
            Shell::run(cmd, self.path)
                .then(|(_, stdout, _)| println!("{}", stdout))
                .warn(format!("Failed executing {} in {}", cmd.as_str().yellow(), format!("{:?}", self.path).as_str().yellow()));
        })
    }

    fn erase_if_dry(&self) -> anyhow::Result<()> {
        if self.data.is_dry() {
            info!("Removing generated project {}", format!("{:?}", self.path).as_str().cyan());
            self.path.delete_dir()
                .then(|_| info!("{} deleted", format!("{:?}", self.path).as_str().cyan()))
                .else_warn(format!("Failed deleting {}", format!("{:?}", self.path).as_str().yellow()))
                .wrap()
        } else { Ok(()) }
    }
}