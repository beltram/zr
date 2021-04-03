use sha2::{Digest, Sha256};

pub trait ConfigHash {
    /// Computes a SHA-256 hash of each git url in order to ensure uniqueness
    fn config_hash(&self) -> String;
}

impl ConfigHash for &String {
    fn config_hash(&self) -> String {
        let mut hasher = Sha256::new();
        hasher.update(self);
        hex::encode(hasher.finalize())
    }
}

#[cfg(test)]
mod config_hash_tests {
    use super::*;

    #[test]
    fn should_hash() {
        let github = &String::from("https://github.com/beltram/my-zr-config.git");
        assert_eq!(github.config_hash().as_str(), "e60c242d21eda9a56777374159b48917d6e89665f206cc4b630e19be540039de");
        let gitlab = &String::from("https://gitlab.com/beltram/my-zr-config.git");
        assert_eq!(gitlab.config_hash().as_str(), "8e447b758171188de3ce8cb808fee095cb68006ff4c381c1fca65ce27ee99ecf");
    }
}