// Regression test for issue #23: nested attribute with core alias
// Tests that the macro accepts nested attribute without compilation errors

#![allow(unused_imports)]

use core as tradesmith_core; // Alias core like in the original issue
use gonfig::Gonfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "APP")]
pub struct Config {
    #[gonfig(nested)]
    #[serde(default)] // Required for automatic nested loading
    pub server: ServerConfig,

    #[gonfig(default = "production")]
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig, Default)]
#[gonfig(env_prefix = "SERVER")]
#[serde(default)] // Allows automatic nested loading
pub struct ServerConfig {
    #[gonfig(default = "127.0.0.1")]
    pub host: String,

    #[gonfig(default = "3000")]
    pub port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nested_attribute_compiles_with_core_alias() {
        // **Main fix**: This test verifies the compilation error is fixed
        // Original error: "failed to resolve: could not find `compile_error` in `core`"
        // This happened because darling used unqualified core::compile_error!()
        // when it encountered unknown attribute `nested`

        // By adding `nested` field to GonfigField, darling accepts it and
        // the code compiles successfully even with core aliased

        // Test that nested structs load independently
        let server = ServerConfig::from_gonfig();
        assert!(server.is_ok(), "Nested struct should load successfully");

        let server = server.unwrap();
        assert_eq!(server.host, "127.0.0.1");
        assert_eq!(server.port, 3000);
    }

    #[test]
    fn test_automatic_nested_loading_with_core_alias() {
        // **Enhancement**: As of v0.1.12, nested fields are automatically loaded!
        // No manual composition needed anymore

        let config = Config::from_gonfig();
        assert!(
            config.is_ok(),
            "Config with nested fields should load automatically: {:?}",
            config.err()
        );

        let config = config.unwrap();

        // Nested struct was loaded automatically via ServerConfig::from_gonfig()
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.environment, "production");
    }
}
