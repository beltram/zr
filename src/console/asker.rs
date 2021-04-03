use std::io::{stdin, stdout, Write};

use colored::Colorize;

use crate::utils::error::ErrorExt;

/// Wraps utilities asking user for a choice
pub struct Asker {}

impl Asker {
    const YES: &'static str = "y";

    /// Asks a yes/no question.
    /// Captures user answer from stdin.
    pub fn ask<F>(question: &str, if_yes: F) -> bool where F: Fn() {
        let mut yes_no = String::new();
        print!("{} {}", question, "(Y/n): ".bold().yellow());
        let _ = stdout().flush();
        stdin().read_line(&mut yes_no).fail("Invalid answer");
        let user_answered_yes = yes_no.trim().eq_ignore_ascii_case(Self::YES);
        if user_answered_yes { if_yes(); }
        user_answered_yes
    }
}