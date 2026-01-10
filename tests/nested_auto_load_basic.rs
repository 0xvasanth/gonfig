// Test automatic nested struct loading - basic scenario
// Uses unique env vars to avoid test interference

use gonfig::Gonfig;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig, Default)]
#[gonfig(env_prefix = "BASIC_SERVER")]
#[serde(default)]
pub struct BasicServerConfig {
    #[gonfig(default = "127.0.0.1")]
    pub host: String,

    #[gonfig(default = "3000")]
    pub port: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "BASIC_APP")]
pub struct BasicAppConfig {
    #[gonfig(nested)]
    #[serde(default)]
    pub server: BasicServerConfig,

    #[gonfig(default = "production")]
    pub environment: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_automatic_nested_loading() {
        let config = BasicAppConfig::from_gonfig();

        assert!(
            config.is_ok(),
            "Should automatically load nested struct: {:?}",
            config.err()
        );

        let config = config.unwrap();

        // Verify nested struct was loaded automatically
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 3000);
        assert_eq!(config.environment, "production");
    }
}
