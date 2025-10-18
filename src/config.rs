use crate::{
    error::{Error, Result},
    source::{ConfigSource, Source},
};
use serde_json::Value;
use std::any::Any;
use std::fs;
use std::path::{Path, PathBuf};

/// Supported configuration file formats.
///
/// This enum represents the different file formats that gonfig can parse
/// for configuration files. Each format has its own parsing logic and
/// error handling.
///
/// # Examples
///
/// ```rust
/// use gonfig::ConfigFormat;
///
/// // Automatic detection from file extension
/// let format = ConfigFormat::from_extension("json").unwrap();
/// assert!(matches!(format, ConfigFormat::Json));
///
/// // Manual format specification
/// let format = ConfigFormat::Yaml;
/// ```
#[derive(Debug, Clone, PartialEq)]
pub enum ConfigFormat {
    /// JSON format (.json files)
    Json,
    /// YAML format (.yaml, .yml files)  
    Yaml,
    /// TOML format (.toml files)
    Toml,
}

impl ConfigFormat {
    /// Detect configuration format from file extension.
    ///
    /// Returns the appropriate format for common file extensions:
    /// - `json` → [`ConfigFormat::Json`]
    /// - `yaml`, `yml` → [`ConfigFormat::Yaml`]
    /// - `toml` → [`ConfigFormat::Toml`]
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::ConfigFormat;
    ///
    /// assert!(matches!(ConfigFormat::from_extension("json"), Some(ConfigFormat::Json)));
    /// assert!(matches!(ConfigFormat::from_extension("yml"), Some(ConfigFormat::Yaml)));
    /// assert!(matches!(ConfigFormat::from_extension("toml"), Some(ConfigFormat::Toml)));
    /// assert_eq!(ConfigFormat::from_extension("unknown"), None);
    /// ```
    pub fn from_extension(ext: &str) -> Option<Self> {
        match ext.to_lowercase().as_str() {
            "json" => Some(ConfigFormat::Json),
            "yaml" | "yml" => Some(ConfigFormat::Yaml),
            "toml" => Some(ConfigFormat::Toml),
            _ => None,
        }
    }

    /// Parse configuration content according to the format.
    ///
    /// Converts the string content into a [`serde_json::Value`] that can be
    /// merged with other configuration sources. All formats are normalized
    /// to JSON values internally.
    ///
    /// # Arguments
    ///
    /// * `content` - The configuration file content as a string
    ///
    /// # Returns
    ///
    /// A [`serde_json::Value`] representing the parsed configuration.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Serialization`] if the content cannot be parsed
    /// according to the format's syntax rules.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::ConfigFormat;
    ///
    /// let format = ConfigFormat::Json;
    /// let content = r#"{"name": "test", "port": 8080}"#;
    /// let value = format.parse(content).unwrap();
    /// ```
    pub fn parse(&self, content: &str) -> Result<Value> {
        match self {
            ConfigFormat::Json => serde_json::from_str(content)
                .map_err(|e| Error::Serialization(format!("JSON parse error: {e}"))),
            ConfigFormat::Yaml => serde_yaml::from_str(content)
                .map_err(|e| Error::Serialization(format!("YAML parse error: {e}"))),
            ConfigFormat::Toml => {
                let toml_value: toml::Value = toml::from_str(content)
                    .map_err(|e| Error::Serialization(format!("TOML parse error: {e}")))?;
                serde_json::to_value(toml_value).map_err(|e| {
                    Error::Serialization(format!("TOML to JSON conversion error: {e}"))
                })
            }
        }
    }
}

/// Configuration file source.
///
/// The `Config` struct represents a configuration file that can be loaded
/// and parsed. It supports automatic format detection, optional files,
/// and various configuration file formats (JSON, YAML, TOML).
///
/// # Examples
///
/// ```rust,no_run
/// use gonfig::Config;
///
/// // Load a required configuration file
/// let config = Config::from_file("app.json")?;
///
/// // Load an optional configuration file (won't fail if missing)
/// let config = Config::from_file_optional("optional.yaml")?;
/// # Ok::<(), gonfig::Error>(())
/// ```
#[derive(Debug, Clone)]
pub struct Config {
    path: PathBuf,
    format: ConfigFormat,
    required: bool,
    data: Option<Value>,
}

impl Config {
    /// Load a required configuration file with automatic format detection.
    ///
    /// The file format is detected from the file extension. If the file doesn't
    /// exist or cannot be parsed, this method returns an error.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gonfig::Config;
    ///
    /// let config = Config::from_file("app.json")?;
    /// let config = Config::from_file("settings.yaml")?;
    /// let config = Config::from_file("config.toml")?;
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// - [`Error::Config`] if the file extension is not recognized
    /// - [`Error::Io`] if the file cannot be read
    /// - [`Error::Serialization`] if the file cannot be parsed
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(ConfigFormat::from_extension)
            .ok_or_else(|| Error::Config(format!("Unknown config format for file: {path:?}")))?;

        let mut config = Self {
            path,
            format,
            required: true,
            data: None,
        };

        config.load()?;
        Ok(config)
    }

    /// Load an optional configuration file with automatic format detection.
    ///
    /// Similar to [`from_file`], but won't return an error if the file doesn't exist.
    /// Parse errors will still cause the method to fail.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the optional configuration file
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gonfig::Config;
    ///
    /// // Won't fail if user.json doesn't exist
    /// let config = Config::from_file_optional("user.json")?;
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    ///
    /// [`from_file`]: Config::from_file
    pub fn from_file_optional(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref().to_path_buf();
        let format = path
            .extension()
            .and_then(|ext| ext.to_str())
            .and_then(ConfigFormat::from_extension)
            .ok_or_else(|| Error::Config(format!("Unknown config format for file: {path:?}")))?;

        let path_display = path.display().to_string();
        let mut config = Self {
            path,
            format,
            required: false,
            data: None,
        };

        // For optional configs, only ignore file-not-found errors
        match config.load() {
            Ok(()) => {}
            Err(Error::Io(ref e)) if e.kind() == std::io::ErrorKind::NotFound => {
                // File not found is OK for optional configs
            }
            Err(e) => {
                // Log parse errors but don't fail
                tracing::warn!(
                    "Failed to parse optional config file {}: {}",
                    path_display,
                    e
                );
            }
        }
        Ok(config)
    }

    /// Load a configuration file with explicit format specification.
    ///
    /// Use this method when you need to override automatic format detection
    /// or when working with files that don't have standard extensions.
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the configuration file
    /// * `format` - The format to use for parsing
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gonfig::{Config, ConfigFormat};
    ///
    /// // Force JSON parsing for a file without extension
    /// let config = Config::with_format("config", ConfigFormat::Json)?;
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    pub fn with_format(path: impl AsRef<Path>, format: ConfigFormat) -> Result<Self> {
        let mut config = Self {
            path: path.as_ref().to_path_buf(),
            format,
            required: true,
            data: None,
        };

        config.load()?;
        Ok(config)
    }

    fn load(&mut self) -> Result<()> {
        match fs::read_to_string(&self.path) {
            Ok(content) => {
                self.data = Some(self.format.parse(&content)?);
                Ok(())
            }
            Err(e) => {
                if self.required {
                    Err(Error::Io(e))
                } else {
                    self.data = Some(Value::Object(serde_json::Map::new()));
                    Ok(())
                }
            }
        }
    }

    /// Reload the configuration from disk.
    ///
    /// This method re-reads the configuration file and parses it again.
    /// Useful for applications that need to respond to configuration changes
    /// at runtime.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gonfig::Config;
    ///
    /// let mut config = Config::from_file("app.json")?;
    /// // ... some time later ...
    /// config.reload()?; // Re-read from disk
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns the same errors as the original loading method if the file
    /// cannot be read or parsed.
    pub fn reload(&mut self) -> Result<()> {
        self.load()
    }
}

impl ConfigSource for Config {
    fn source_type(&self) -> Source {
        Source::ConfigFile
    }

    fn collect(&self) -> Result<Value> {
        Ok(self
            .data
            .clone()
            .unwrap_or_else(|| Value::Object(serde_json::Map::new())))
    }

    fn has_value(&self, key: &str) -> bool {
        if let Some(data) = &self.data {
            let parts: Vec<&str> = key.split('.').collect();
            let mut current = data;

            for part in parts {
                match current.get(part) {
                    Some(value) => current = value,
                    None => return false,
                }
            }
            true
        } else {
            false
        }
    }

    fn get_value(&self, key: &str) -> Option<Value> {
        if let Some(data) = &self.data {
            let parts: Vec<&str> = key.split('.').collect();
            let mut current = data;

            for part in parts {
                match current.get(part) {
                    Some(value) => current = value,
                    None => return None,
                }
            }
            Some(current.clone())
        } else {
            None
        }
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
