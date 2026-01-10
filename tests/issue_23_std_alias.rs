// Regression test for issue #23: https://github.com/0xvasanth/gonfig/issues/23
// Tests that the gonfig derive macro works correctly when std crate is aliased

#![allow(unused_imports)]

use gonfig::Gonfig;
use serde::{Deserialize, Serialize};
use std as my_std; // Alias std to trigger potential path resolution issues

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "STD_ALIAS_TEST", allow_config)]
pub struct ConfigWithStdAlias {
    #[gonfig(default = "test_value")]
    pub field: String,

    #[gonfig(default = "42")]
    pub number: u32,
}

// Test with optional fields that might reference std types
#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[gonfig(env_prefix = "OPT", allow_config)]
pub struct OptionalConfigWithStdAlias {
    #[gonfig(default = r#"null"#)]
    pub optional_field: Option<String>,

    #[gonfig(default = r#"["item1","item2"]"#)]
    pub list_field: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_std_alias_basic_with_config() {
        // This test verifies that the macro handles std aliasing correctly
        // The issue manifests when allow_config is enabled because the macro
        // generates "use std::path::Path;" which would resolve incorrectly
        let config = ConfigWithStdAlias::from_gonfig();
        assert!(config.is_ok(), "Should compile with std alias");

        let config = config.unwrap();
        assert_eq!(config.field, "test_value");
        assert_eq!(config.number, 42);
    }

    #[test]
    fn test_std_alias_with_option_types() {
        // Test with Option and Vec types to ensure std::option and std::vec work
        let config = OptionalConfigWithStdAlias::from_gonfig();
        assert!(
            config.is_ok(),
            "Should compile with std alias and Option/Vec types"
        );

        let config = config.unwrap();
        assert_eq!(config.optional_field, None);
        assert_eq!(config.list_field, vec!["item1", "item2"]);
    }
}
