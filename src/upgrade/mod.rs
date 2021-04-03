use colored::Colorize;

use upgradable::Upgradable;

use crate::completion::CliCompletion;
use crate::config::global::Config;

pub mod upgradable;

pub struct ZrUpgrade {}

impl ZrUpgrade {
    pub fn upgrade() {
        Config::upgrade(Config::get());
        info!("Updating {}", "completion files".green());
        CliCompletion::apply(None);
    }
}