use std::fmt::Display;

use colored::Colorize;

use crate::console::emoji;

pub trait ErrorExt<T> {
    const UNEXPECTED_ERROR: &'static str = "An unexpected error occurred.";

    /// Fails with supplied error message
    fn fail<M: AsRef<str>>(self, msg: M) -> T;
    /// Does not fail but logs a warning
    fn warn<M: AsRef<str>>(self, msg: M);
    /// Does not fail but logs a warning and continue
    fn else_warn<M: AsRef<str>>(self, msg: M) -> Self;
    /// Does not fail but logs the error in debug
    fn debug<M: AsRef<str>>(self, msg: M);
    /// Fails with a default message when error is very strange.
    /// Error has to be investigated with a high priority.
    fn unexpected_failure(self) -> T;
    /// Execute fun if Ok or Some then continue
    fn then<F>(self, fun: F) -> Self where F: Fn(&T);
}

impl<T, E: Display> ErrorExt<T> for Result<T, E> {
    fn fail<M: AsRef<str>>(self, msg: M) -> T {
        self.unwrap_or_else(|e| panic!("{} {} because {}", emoji::FAILURE, msg.as_ref().red(), e))
    }

    fn warn<M: AsRef<str>>(self, msg: M) {
        if let Some(e) = self.err() {
            warn!("{} {} because {}", emoji::WARNING, msg.as_ref().yellow(), e)
        }
    }

    fn else_warn<M: AsRef<str>>(self, msg: M) -> Self {
        if let Some(e) = self.as_ref().err() {
            warn!("{} {} because {}", emoji::WARNING, msg.as_ref().yellow(), e)
        }
        self
    }

    fn debug<M: AsRef<str>>(self, msg: M) {
        if let Some(e) = self.err() {
            debug!("{} {} because {}", emoji::WARNING, msg.as_ref().dimmed(), e)
        }
    }

    fn unexpected_failure(self) -> T {
        self.unwrap_or_else(|e| panic!("{} {} because {}", emoji::FAILURE, Self::UNEXPECTED_ERROR, e))
    }

    fn then<F>(self, fun: F) -> Self where F: Fn(&T) {
        self.map(|it| {
            fun(&it);
            it
        })
    }
}

impl<T> ErrorExt<T> for Option<T> {
    fn fail<M: AsRef<str>>(self, msg: M) -> T {
        self.unwrap_or_else(|| panic!("{} {}", emoji::FAILURE, msg.as_ref().red()))
    }

    fn warn<M: AsRef<str>>(self, msg: M) {
        self.or_else(|| {
            warn!("{} {}", emoji::WARNING, msg.as_ref().yellow());
            None
        });
    }

    fn else_warn<M: AsRef<str>>(self, msg: M) -> Self {
        if self.is_none() { warn!("{} {}", emoji::WARNING, msg.as_ref().yellow()) }
        self
    }

    fn debug<M: AsRef<str>>(self, msg: M) {
        self.or_else(|| {
            debug!("{} {}", emoji::WARNING, msg.as_ref().dimmed());
            None
        });
    }

    fn unexpected_failure(self) -> T {
        self.unwrap_or_else(|| panic!("{} {}", emoji::FAILURE, Self::UNEXPECTED_ERROR))
    }

    fn then<F>(self, fun: F) -> Self where F: Fn(&T) {
        self.map(|it| {
            fun(&it);
            it
        })
    }
}