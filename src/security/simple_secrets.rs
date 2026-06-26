use anyhow::Result;
use std::env;

/// Interface for sharing secrets via stdin/stdout or ENV variables
pub struct SimpleSecretsProvider;

impl SimpleSecretsProvider {
    pub fn get_secret_from_env(key: &str) -> Result<String> {
        env::var(key).map_err(|_| anyhow::anyhow!("Secret not found in ENV"))
    }

    pub fn read_json_secret_from_stdin() -> Result<String> {
        Err(anyhow::anyhow!(
            "Reading json secret from stdin not implemented"
        ))
    }
}
