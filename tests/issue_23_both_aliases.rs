// Regression test for issue #23: https://github.com/0xvasanth/gonfig/issues/23
// Comprehensive test with both core and std aliased and all features enabled

#![allow(unused_imports)]

use core as my_core;
use gonfig::Gonfig;
use serde::{Deserialize, Serialize};
use std as my_std;

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "BOTH_ALIAS", allow_config, allow_cli)]
pub struct FullFeaturedConfig {
    #[gonfig(default = "localhost")]
    pub hostname: String,

    #[gonfig(default = "8080")]
    pub port: u16,

    #[gonfig(default = "info")]
    #[gonfig(env_name = "LOG_LEVEL")]
    pub log_level: String,
}

// Test nested configuration
#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "DATABASE", allow_config)]
pub struct DatabaseConfig {
    #[gonfig(default = "postgres://localhost/mydb")]
    pub url: String,

    #[gonfig(default = "10")]
    pub max_connections: u32,

    #[gonfig(default = "30")]
    pub timeout_seconds: u64,
}

// Test with skip attribute
#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "APP", allow_config, allow_cli)]
pub struct AppConfigWithSkip {
    #[gonfig(default = "production")]
    pub environment: String,

    #[skip]
    #[serde(skip)]
    pub runtime_data: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_both_aliases_all_features() {
        // Most comprehensive test: both std and core aliased, all features enabled
        let config = FullFeaturedConfig::from_gonfig();
        assert!(
            config.is_ok(),
            "Should compile and run with both std and core aliased"
        );

        let config = config.unwrap();
        assert_eq!(config.hostname, "localhost");
        assert_eq!(config.port, 8080);
        assert_eq!(config.log_level, "info");
    }

    #[test]
    fn test_both_aliases_nested_config() {
        // Test with database configuration
        let config = DatabaseConfig::from_gonfig();
        assert!(
            config.is_ok(),
            "Nested config should work with both aliases"
        );

        let config = config.unwrap();
        assert_eq!(config.url, "postgres://localhost/mydb");
        assert_eq!(config.max_connections, 10);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_both_aliases_with_skip() {
        // Test with skip attribute
        let config = AppConfigWithSkip::from_gonfig();
        assert!(config.is_ok(), "Config with skip should work with aliases");

        let config = config.unwrap();
        assert_eq!(config.environment, "production");
        assert_eq!(config.runtime_data, None);
    }

    #[test]
    fn test_builder_pattern_with_aliases() {
        // Test the builder pattern also works with aliases
        let builder = FullFeaturedConfig::gonfig_builder();
        let config = FullFeaturedConfig::from_gonfig_with_builder(builder);
        assert!(config.is_ok(), "Builder pattern should work with aliases");
    }
}
