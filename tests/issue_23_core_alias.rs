// Regression test for issue #23: https://github.com/0xvasanth/gonfig/issues/23
// Tests that the gonfig derive macro works correctly when core crate is aliased

#![allow(unused_imports)]

use core as tradesmith_core; // Alias core to simulate user's environment
use gonfig::Gonfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "CORE_ALIAS_TEST", allow_config)]
pub struct ConfigWithCoreAlias {
    #[gonfig(default = "127.0.0.1")]
    pub host: String,

    #[gonfig(default = "8080")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "SERVER", allow_config, allow_cli)]
pub struct ServerConfigWithCoreAlias {
    #[gonfig(default = "localhost")]
    pub address: String,

    #[gonfig(default = "3000")]
    pub server_port: u16,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_alias_basic_with_config() {
        // This test verifies that the macro-generated code compiles
        // when core is aliased and allow_config is enabled
        let config = ConfigWithCoreAlias::from_gonfig();
        assert!(config.is_ok(), "Should compile with core alias");

        let config = config.unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, 8080);
    }

    #[test]
    fn test_core_alias_with_all_features() {
        // Test with both allow_config and allow_cli enabled
        let config = ServerConfigWithCoreAlias::from_gonfig();
        assert!(
            config.is_ok(),
            "Should compile with core alias and all features"
        );

        let config = config.unwrap();
        assert_eq!(config.address, "localhost");
        assert_eq!(config.server_port, 3000);
    }
}
