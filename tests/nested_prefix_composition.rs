// Test automatic prefix composition for nested structs
// Verifies that TRADESMITH + SERVER = TRADESMITH_SERVER_HOST

use gonfig::Gonfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig, Default)]
#[gonfig(env_prefix = "SERVER")] // Just "SERVER", not "TRADESMITH_SERVER"
#[serde(default)]
pub struct ServerConfig {
    #[gonfig(default = "127.0.0.1")]
    pub host: String,

    #[gonfig(default = "8080")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig, Default)]
#[gonfig(env_prefix = "DATABASE")] // Just "DATABASE", not "TRADESMITH_DATABASE"
#[serde(default)]
pub struct DatabaseConfig {
    #[gonfig(default = "sqlite:./data.db")]
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "TRADESMITH")] // Parent prefix
pub struct TradeSmithConfig {
    #[gonfig(nested)]
    #[serde(default)]
    pub server: ServerConfig,

    #[gonfig(nested)]
    #[serde(default)]
    pub database: DatabaseConfig,

    #[gonfig(default = "production")]
    pub environment: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prefix_composition() {
        // Set environment variables using COMPOSED prefixes
        // TRADESMITH (parent) + SERVER (nested) = TRADESMITH_SERVER
        std::env::set_var("TRADESMITH_SERVER_HOST", "0.0.0.0");
        std::env::set_var("TRADESMITH_SERVER_PORT", "9000");
        std::env::set_var("TRADESMITH_DATABASE_URL", "postgres://db");
        std::env::set_var("TRADESMITH_ENVIRONMENT", "staging");

        let config = TradeSmithConfig::from_gonfig();

        assert!(
            config.is_ok(),
            "Should load with composed prefixes: {:?}",
            config.err()
        );

        let config = config.unwrap();

        // Verify nested fields loaded from COMPOSED prefixes
        assert_eq!(
            config.server.host, "0.0.0.0",
            "Should use TRADESMITH_SERVER_HOST"
        );
        assert_eq!(
            config.server.port, 9000,
            "Should use TRADESMITH_SERVER_PORT"
        );
        assert_eq!(
            config.database.url, "postgres://db",
            "Should use TRADESMITH_DATABASE_URL"
        );
        assert_eq!(
            config.environment, "staging",
            "Should use TRADESMITH_ENVIRONMENT"
        );

        // Cleanup
        std::env::remove_var("TRADESMITH_SERVER_HOST");
        std::env::remove_var("TRADESMITH_SERVER_PORT");
        std::env::remove_var("TRADESMITH_DATABASE_URL");
        std::env::remove_var("TRADESMITH_ENVIRONMENT");
    }

    #[test]
    fn test_nested_without_parent_prefix_fails() {
        // Clean environment
        std::env::remove_var("TRADESMITH_SERVER_HOST");
        std::env::remove_var("SERVER_HOST"); // Just SERVER_ prefix won't work

        // Set using non-composed prefix (should NOT work)
        std::env::set_var("SERVER_HOST", "wrong.example.com");

        let config = TradeSmithConfig::from_gonfig().expect("Should load");

        // Should use default, NOT the SERVER_HOST value
        assert_eq!(
            config.server.host, "127.0.0.1",
            "Should ignore SERVER_HOST, require TRADESMITH_SERVER_HOST"
        );

        // Cleanup
        std::env::remove_var("SERVER_HOST");
    }

    #[test]
    fn test_composed_prefix_takes_precedence() {
        // Set both composed and non-composed
        std::env::set_var("TRADESMITH_SERVER_PORT", "9999");
        std::env::set_var("SERVER_PORT", "7777");

        let config = TradeSmithConfig::from_gonfig().expect("Should load");

        // Should use composed prefix (TRADESMITH_SERVER_PORT), not SERVER_PORT
        assert_eq!(
            config.server.port, 9999,
            "Composed prefix TRADESMITH_SERVER_PORT should be used"
        );

        // Cleanup
        std::env::remove_var("TRADESMITH_SERVER_PORT");
        std::env::remove_var("SERVER_PORT");
    }
}
