use crate::config::global::Config;

/// Enables printing current config file location.
/// Pretty standard feature for any cli.
pub struct GetConfig {}

impl GetConfig {
    /// prints current config file location
    pub fn exec() {
        if let Some(path) = Config::path().as_ref().and_then(|it| it.to_str()) {
            println!("{}", path);
        }
    }
}