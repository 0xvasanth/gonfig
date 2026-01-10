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
    pub server: ServerConfig,

    #[gonfig(default = "production")]
    pub environment: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "SERVER")]
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
    fn test_manual_nested_composition() {
        // Demonstrate the current recommended pattern for nested configs
        // (Automatic composition will be added in a future version)

        let server = ServerConfig::from_gonfig().expect("Server config should load");
        let config = Config {
            server,
            environment: "production".to_string(),
        };

        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.environment, "production");
    }
}
