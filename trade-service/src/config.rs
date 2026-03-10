use crate::error::ConfigError;
use serde::{Deserialize, Serialize};

// General Config
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    // Port for service to bind to
    pub port: u16,
    // Preferred log format
    pub log_format: String,
    // Selected publishing backend
    pub publish_backend: TradeServiceBackend,
    // Health/Ready endpoint port
    pub health_endpoint_port: u16,
    // Interval for readiness monitor
    pub monitor_interval_secs: u64,
}

// Backend Options
#[derive(Debug, Clone, Deserialize, Serialize)]
pub enum TradeServiceBackend {
    #[serde(rename = "stub")]
    Stub,
    #[serde(rename = "kafka")]
    Kafka,
}

// Create App Config
impl AppConfig {
    pub fn from_env() -> Self {
        // Load vars in given struct from env
        // Replaces values set in the .env file
        envy::from_env::<AppConfig>().expect("Failed to parse AppConfig from env")
    }
}

// Kafka Config
#[derive(Debug, Clone, Deserialize)]
pub struct KafkaConfig {
    // Kafka Server
    pub broker: String,
    // Destination Topic
    pub topic: String,
}

// Create Kafka Config
impl KafkaConfig {
    pub fn from_env() -> Result<Self, ConfigError> {
        // Load vars in given struct from env where env vars are KAFKA_(var name)
        // Replaces values set in the .env file
        Ok(envy::prefixed("KAFKA_").from_env::<KafkaConfig>()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use temp_env::with_vars;

    #[test]
    fn app_config_from_env_success() {
        with_vars(
            [
                ("PORT", Some("8080")),
                ("LOG_FORMAT", Some("json")),
                ("PUBLISH_BACKEND", Some("stub")),
                ("HEALTH_ENDPOINT_PORT", Some("9000")),
                ("MONITOR_INTERVAL_SECS", Some("30")),
            ],
            || {
                let cfg = AppConfig::from_env();
                assert_eq!(cfg.port, 8080);
                assert_eq!(cfg.log_format, "json");
                matches!(cfg.publish_backend, TradeServiceBackend::Stub);
                assert_eq!(cfg.health_endpoint_port, 9000);
                assert_eq!(cfg.monitor_interval_secs, 30);
            },
        );
    }

    #[test]
    #[should_panic]
    fn app_config_from_env_missing_var_panics() {
        // No env vars, expect panic
        let _ = AppConfig::from_env();
    }
}
