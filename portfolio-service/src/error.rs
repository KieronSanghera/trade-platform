use envy;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Invalid config: {0}")]
    InvalidConfig(#[from] envy::Error),
}
