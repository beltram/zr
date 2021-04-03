use std::{env, fmt::Display, ops::Not, path::PathBuf};

use clap::Clap;
use strum::AsRefStr;

use crate::utils::anyhow_err::{ErrConversion, OptConversion};

#[derive(Clap, Debug, Copy, Clone, Display, Eq, PartialEq, AsRefStr)]
#[clap(rename_all = "kebab-case")]
pub enum Shell {
    /// Bash
    Bash,
    /// Zsh
    Zsh,
    /// Fish
    Fish,
    /// Elvish
    Elvish,
    /// Powershell
    Powershell,
}

impl Shell {
    /// is always set in a bash shell
    const IS_BASH: &'static str = "BASH";
    /// is always set in a zsh shell but might be built-in
    const IS_ZSH: &'static str = "UPDATE_ZSH_DAYS";

    pub fn current() -> Option<Self> {
        Self::is_bash().or_else(Self::is_zsh)
    }

    fn is_bash() -> Option<Self> {
        env::var(Self::IS_BASH).ok()
            .filter(|it| it.is_empty().not())
            .map(|_| Self::Bash)
    }
    fn is_zsh() -> Option<Self> {
        env::var(Self::IS_ZSH)
            .ok()
            .filter(|it| it.is_empty().not())
            .map(|_| Self::Zsh)
    }

    /// Executes the given cmd in current shell on MacOs or Linux
    #[cfg(unix)]
    pub fn run(cmd: &str, into: &PathBuf) -> anyhow::Result<(i32, String, String)> {
        Self::current()
            .wrap("Could not determine current shell")
            .and_then(|shell| {
                env::set_current_dir(into).wrap()
                    .map(|_| {
                        match shell {
                            Shell::Bash => bash!("{}", cmd),
                            Shell::Zsh => zsh!("{}", cmd),
                            Shell::Fish => fish!("{}", cmd),
                            _ => sh!("{}", cmd)
                        }
                    })
            }).or_else(|_| Ok(sh!("{}", cmd)))
    }

    /// Executes the given cmd in current shell on Windows
    #[cfg(windows)]
    pub fn run(cmd: &str, into: &PathBuf) -> Option<()> {
        Self::current()
            .and_then(|shell| {
                env::set_current_dir(into).unexpected_failure();
                match shell {
                    Shell::Bash => wrap_bash!("{}",cmd).ok().map(|_| ()),
                    _ => Command::new("cmd").args(&["/C", cmd]).spawn()
                        .and_then(|mut s| s.wait().map(|_| ()))
                        .ok()
                }
            })
    }
}

#[cfg(test)]
mod shell_tests {
    use super::*;

    fn before_all() {
        env::remove_var(Shell::IS_BASH);
        env::remove_var(Shell::IS_ZSH);
    }

    #[test]
    fn should_detect_bash_shell() {
        before_all();
        env::set_var(Shell::IS_BASH, "/bin/bash");
        assert_eq!(Shell::current(), Some(Shell::Bash));
    }

    #[test]
    fn should_not_detect_bash_when_env_var_empty() {
        before_all();
        env::set_var(Shell::IS_BASH, "");
        assert!(Shell::current().is_none());
    }

    #[test]
    fn should_detect_zsh_shell() {
        before_all();
        env::set_var(Shell::IS_ZSH, "/bin/zsh");
        assert_eq!(Shell::current(), Some(Shell::Zsh));
    }

    #[test]
    fn should_not_detect_zsh_when_env_var_empty() {
        before_all();
        env::set_var(Shell::IS_ZSH, "");
        assert!(Shell::current().is_none());
    }

    #[test]
    fn should_not_detect_any_when_no_env_var_present() {
        before_all();
        assert!(Shell::current().is_none());
    }
}