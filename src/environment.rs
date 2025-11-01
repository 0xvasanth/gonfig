use crate::{
    error::Result,
    source::{ConfigSource, Source},
    Prefix,
};
use serde_json::{json, Map, Value};
use std::any::Any;
use std::collections::HashMap;
use std::env;

/// Environment variable configuration source.
///
/// The `Environment` struct provides a flexible way to read configuration values
/// from environment variables. It supports prefixes, custom separators, case sensitivity
/// control, and field-specific mappings.
///
/// # Examples
///
/// ## Basic Usage
///
/// ```rust
/// use gonfig::{Environment, ConfigBuilder};
/// use serde::Deserialize;
///
/// #[derive(Deserialize)]
/// struct Config {
///     database_url: String,
///     port: u16,
/// }
///
/// std::env::set_var("APP_DATABASE_URL", "postgres://localhost/db");
/// std::env::set_var("APP_PORT", "5432");
///
/// let config: Config = ConfigBuilder::new()
///     .add_source(Box::new(Environment::new().with_prefix("APP")))
///     .build()
///     .unwrap();
/// ```
///
/// ## Advanced Configuration
///
/// ```rust
/// use gonfig::Environment;
///
/// let env = Environment::new()
///     .with_prefix("MYAPP")
///     .separator("__")  // Use double underscore
///     .case_sensitive(true)
///     .override_with("database_url", "postgres://override/db")
///     .with_field_mapping("db_url", "CUSTOM_DB_CONNECTION");
/// ```
#[derive(Debug, Clone)]
pub struct Environment {
    prefix: Option<Prefix>,
    separator: String,
    case_sensitive: bool,
    overrides: HashMap<String, String>,
    field_mappings: HashMap<String, String>,
    nested: bool,
}

impl Default for Environment {
    fn default() -> Self {
        Self {
            prefix: None,
            separator: "_".to_string(),
            case_sensitive: false,
            overrides: HashMap::new(),
            field_mappings: HashMap::new(),
            nested: false,
        }
    }
}

impl Environment {
    /// Create a new environment variable source with default settings.
    ///
    /// Default configuration:
    /// - No prefix
    /// - Separator: `"_"`
    /// - Case sensitive: `false` (environment variables are converted to uppercase)
    /// - No overrides or field mappings
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::Environment;
    ///
    /// let env = Environment::new();
    /// ```
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the environment variable prefix.
    ///
    /// When a prefix is set, environment variables will be expected in the format
    /// `{PREFIX}{SEPARATOR}{FIELD_NAME}`. For example, with prefix "APP" and
    /// separator "_", a field named `database_url` would map to `APP_DATABASE_URL`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::Environment;
    ///
    /// let env = Environment::new().with_prefix("MYAPP");
    /// // Will look for MYAPP_* environment variables
    /// ```
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(Prefix::new(prefix));
        self
    }

    /// Set the separator used between prefix and field names.
    ///
    /// The default separator is `"_"`. This affects how environment variable
    /// names are constructed from the prefix and field names.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::Environment;
    ///
    /// let env = Environment::new()
    ///     .with_prefix("APP")
    ///     .separator("__");  // Results in APP__FIELD_NAME
    /// ```
    pub fn separator(mut self, sep: impl Into<String>) -> Self {
        self.separator = sep.into();
        self
    }

    /// Control case sensitivity for environment variable names.
    ///
    /// When `false` (default), all environment variable names are converted
    /// to uppercase. When `true`, the exact case is preserved.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::Environment;
    ///
    /// let env = Environment::new()
    ///     .with_prefix("app")
    ///     .case_sensitive(true);
    /// // Will look for app_field_name instead of APP_FIELD_NAME
    /// ```
    pub fn case_sensitive(mut self, sensitive: bool) -> Self {
        self.case_sensitive = sensitive;
        self
    }

    /// Override a specific field with a hardcoded value.
    ///
    /// This is useful for providing default values or overriding environment
    /// variables programmatically. Overrides take precedence over actual
    /// environment variables.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::Environment;
    ///
    /// let env = Environment::new()
    ///     .override_with("debug", "true")
    ///     .override_with("timeout", "30");
    /// ```
    pub fn override_with(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.overrides.insert(key.into(), value.into());
        self
    }

    /// Map a specific field to a custom environment variable name.
    ///
    /// This allows you to override the default environment variable naming
    /// for specific fields. The mapping takes precedence over the standard
    /// prefix and separator rules.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::Environment;
    ///
    /// let env = Environment::new()
    ///     .with_prefix("APP")
    ///     .with_field_mapping("database_url", "DATABASE_CONNECTION_STRING");
    /// // database_url will read from DATABASE_CONNECTION_STRING instead of APP_DATABASE_URL
    /// ```
    pub fn with_field_mapping(
        mut self,
        field_name: impl Into<String>,
        env_key: impl Into<String>,
    ) -> Self {
        self.field_mappings
            .insert(field_name.into(), env_key.into());
        self
    }

    /// Enable nested mode to convert flat environment variable keys into nested structures.
    ///
    /// When enabled, environment variables with the configured separator (default: `_`) will be split
    /// into nested paths. For example, `APP_HTTP_PORT=9000` becomes `{"http": {"port": 9000}}`.
    ///
    /// This is essential for properly overriding nested configuration file values with
    /// environment variables when using the Deep merge strategy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::{Environment, ConfigBuilder, MergeStrategy};
    ///
    /// // With nested=true, APP_HTTP_PORT will override http.port in config file
    /// let env = Environment::new()
    ///     .with_prefix("APP")
    ///     .nested(true);
    /// ```
    pub fn nested(mut self, nested: bool) -> Self {
        self.nested = nested;
        self
    }

    fn build_env_key(&self, path: &[&str]) -> String {
        let mut parts = Vec::new();

        if let Some(prefix) = &self.prefix {
            parts.push(prefix.as_str().to_string());
        }

        for part in path {
            parts.push(part.to_string());
        }

        let key = parts.join(&self.separator);

        if self.case_sensitive {
            key
        } else {
            key.to_uppercase()
        }
    }

    fn parse_env_value(value: &str) -> Value {
        if let Ok(b) = value.parse::<bool>() {
            return json!(b);
        }

        if let Ok(n) = value.parse::<i64>() {
            return json!(n);
        }

        if let Ok(n) = value.parse::<f64>() {
            return json!(n);
        }

        if value.starts_with('[') && value.ends_with(']') {
            if let Ok(arr) = serde_json::from_str::<Vec<Value>>(value) {
                return json!(arr);
            }
        }

        if value.starts_with('{') && value.ends_with('}') {
            if let Ok(obj) = serde_json::from_str::<Value>(value) {
                return obj;
            }
        }

        json!(value)
    }

    /// Recursively insert a value into a nested map structure based on a path of keys.
    ///
    /// This helper function takes a flat key path (e.g., ["http", "server", "port"])
    /// and creates the necessary nested structure in the map, inserting the value
    /// at the deepest level.
    fn insert_nested(map: &mut Map<String, Value>, parts: &[&str], value: Value) {
        if parts.is_empty() {
            return;
        }

        if parts.len() == 1 {
            // Base case: insert the value at this key
            map.insert(parts[0].to_string(), value);
            return;
        }

        // Recursive case: get or create the nested object
        let key = parts[0].to_string();
        let nested_map = map
            .entry(key.clone())
            .or_insert_with(|| Value::Object(Map::new()));

        // If the entry exists but is not an object, replace it with an object
        if let Value::Object(ref mut nested) = nested_map {
            Self::insert_nested(nested, &parts[1..], value);
        } else {
            // Replace non-object with a new object containing the nested value
            let mut new_map = Map::new();
            Self::insert_nested(&mut new_map, &parts[1..], value);
            map.insert(key, Value::Object(new_map));
        }
    }

    pub fn collect_for_struct(
        &self,
        struct_name: &str,
        fields: &[(&str, Option<&str>)],
    ) -> HashMap<String, Value> {
        let mut result = HashMap::new();

        for (field_name, field_override) in fields {
            let env_key = if let Some(override_name) = field_override {
                override_name.to_string()
            } else if let Some(prefix) = &self.prefix {
                format!(
                    "{}_{}_{}_{}",
                    prefix.as_str().to_uppercase(),
                    struct_name.to_uppercase(),
                    field_name.to_uppercase(),
                    ""
                )
                .trim_end_matches('_')
                .to_string()
            } else {
                format!(
                    "{}_{}",
                    struct_name.to_uppercase(),
                    field_name.to_uppercase()
                )
            };

            if let Some(override_value) = self.overrides.get(&env_key) {
                result.insert(
                    field_name.to_string(),
                    Self::parse_env_value(override_value),
                );
            } else if let Ok(value) = env::var(&env_key) {
                result.insert(field_name.to_string(), Self::parse_env_value(&value));
            }
        }

        result
    }

    pub fn collect_with_flat_keys(&self) -> Result<Value> {
        let mut flat_map = HashMap::new();

        // First collect from environment variables
        for (key, value) in env::vars() {
            if let Some(prefix) = &self.prefix {
                let prefix_str = if self.case_sensitive {
                    prefix.as_str().to_string()
                } else {
                    prefix.as_str().to_uppercase()
                };

                let key_check = if self.case_sensitive {
                    key.clone()
                } else {
                    key.to_uppercase()
                };

                if key_check.starts_with(&prefix_str) {
                    let trimmed = key_check[prefix_str.len()..].trim_start_matches(&self.separator);
                    // Keep case for nested mode, lowercase for flat mode
                    let key_for_map = if self.nested {
                        trimmed.to_string()
                    } else {
                        trimmed.to_lowercase()
                    };
                    flat_map.insert(key_for_map, Self::parse_env_value(&value));
                }
            } else {
                flat_map.insert(key.to_lowercase(), Self::parse_env_value(&value));
            }
        }

        // Then apply overrides (overrides take precedence)
        for (override_key, override_value) in &self.overrides {
            if let Some(prefix) = &self.prefix {
                let prefix_str = if self.case_sensitive {
                    prefix.as_str().to_string()
                } else {
                    prefix.as_str().to_uppercase()
                };

                let key_check = if self.case_sensitive {
                    override_key.clone()
                } else {
                    override_key.to_uppercase()
                };

                if key_check.starts_with(&prefix_str) {
                    let trimmed = key_check[prefix_str.len()..].trim_start_matches(&self.separator);
                    // Keep case for nested mode, lowercase for flat mode
                    let key_for_map = if self.nested {
                        trimmed.to_string()
                    } else {
                        trimmed.to_lowercase()
                    };
                    flat_map.insert(key_for_map, Self::parse_env_value(override_value));
                }
            } else {
                flat_map.insert(
                    override_key.to_lowercase(),
                    Self::parse_env_value(override_value),
                );
            }
        }

        // Convert flat keys into nested structures if enabled
        let mut result = Map::new();
        for (key, value) in flat_map {
            if self.nested {
                // Split on separator to create nested structure
                let parts: Vec<&str> = key.split(&self.separator).collect();
                if parts.len() == 1 {
                    // Single part, insert directly (lowercase it)
                    result.insert(key.to_lowercase(), value);
                } else {
                    // Multiple parts, create nested structure
                    // Lowercase each part individually
                    let lowercase_parts: Vec<String> =
                        parts.iter().map(|p| p.to_lowercase()).collect();
                    let lowercase_parts_refs: Vec<&str> =
                        lowercase_parts.iter().map(|s| s.as_str()).collect();
                    Self::insert_nested(&mut result, &lowercase_parts_refs, value);
                }
            } else {
                // Keep keys flat (backward compatible behavior)
                result.insert(key.to_lowercase(), value);
            }
        }

        Ok(Value::Object(result))
    }
}

impl ConfigSource for Environment {
    fn source_type(&self) -> Source {
        Source::Environment
    }

    fn collect(&self) -> Result<Value> {
        if !self.field_mappings.is_empty() {
            // Use field mappings when available
            let mut result = Map::new();

            // First collect using field mappings
            for (field_name, env_key) in &self.field_mappings {
                // Check overrides first, then environment
                if let Some(override_value) = self.overrides.get(env_key) {
                    result.insert(field_name.clone(), Self::parse_env_value(override_value));
                } else if let Ok(value) = env::var(env_key) {
                    result.insert(field_name.clone(), Self::parse_env_value(&value));
                }
            }

            // Then collect any prefixed variables not in mappings
            if let Some(prefix) = &self.prefix {
                for (key, value) in env::vars() {
                    let prefix_str = if self.case_sensitive {
                        prefix.as_str().to_string()
                    } else {
                        prefix.as_str().to_uppercase()
                    };

                    let key_check = if self.case_sensitive {
                        key.clone()
                    } else {
                        key.to_uppercase()
                    };

                    if key_check.starts_with(&prefix_str)
                        && !self.field_mappings.values().any(|v| v == &key)
                    {
                        let trimmed =
                            key_check[prefix_str.len()..].trim_start_matches(&self.separator);
                        let field_name = trimmed.to_lowercase();
                        if !result.contains_key(&field_name) {
                            result.insert(field_name, Self::parse_env_value(&value));
                        }
                    }
                }
            }

            Ok(Value::Object(result))
        } else {
            self.collect_with_flat_keys()
        }
    }

    fn has_value(&self, key: &str) -> bool {
        let env_key = self.build_env_key(&[key]);
        self.overrides.contains_key(&env_key) || env::var(&env_key).is_ok()
    }

    fn get_value(&self, key: &str) -> Option<Value> {
        let env_key = self.build_env_key(&[key]);

        if let Some(override_value) = self.overrides.get(&env_key) {
            Some(Self::parse_env_value(override_value))
        } else {
            env::var(&env_key).ok().map(|v| Self::parse_env_value(&v))
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
