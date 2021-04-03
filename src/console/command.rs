use std::io::Error;
use std::process::{Command, ExitStatus, Stdio};

use crate::utils::error::ErrorExt;

/// Extensions for external commands
pub trait CommandExt {
    /// Prevent command from writing to stdout or stderr
    fn no_output(&mut self) -> &mut Self;

    /// Spawns the command then waits until fulfillment and return result
    fn spawn_and_wait(&mut self) -> Result<ExitStatus, Error>;

    /// Spawns the command then waits until fulfillment then on failure panics with message
    fn spawn_and_wait_or_fail(&mut self, msg: &str) -> ExitStatus;
}

impl CommandExt for Command {
    fn no_output(&mut self) -> &mut Self {
        self.stdout(Stdio::null())
            .stderr(Stdio::null())
    }

    fn spawn_and_wait(&mut self) -> Result<ExitStatus, Error> {
        self.spawn().and_then(|mut it| it.wait())
    }

    fn spawn_and_wait_or_fail(&mut self, msg: &str) -> ExitStatus {
        self.spawn_and_wait().fail(msg)
    }
}