//! Configuration builder for assembling multiple configuration sources.

use crate::{
    cli::Cli,
    config::{Config, ConfigFormat},
    environment::Environment,
    error::{Error, Result},
    merge::{ConfigMerger, MergeStrategy},
    source::ConfigSource,
};
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::path::Path;

/// Type alias for validation functions to reduce complexity.
type ValidationFn = Box<dyn Fn(&Value) -> Result<()>>;

/// Builder for assembling configuration from multiple sources.
///
/// The `ConfigBuilder` allows you to combine environment variables, config files,
/// and CLI arguments with different merge strategies and validation rules.
///
/// # Examples
///
/// ```rust,no_run
/// use gonfig::{ConfigBuilder, MergeStrategy};
/// use serde::{Deserialize, Serialize};
///
/// #[derive(Deserialize)]
/// struct Config {
///     database_url: String,
///     port: u16,
/// }
///
/// # fn example() -> gonfig::Result<()> {
/// let config: Config = ConfigBuilder::new()
///     .with_merge_strategy(MergeStrategy::Deep)
///     .with_env("APP")
///     .with_cli()
///     .build()?;
/// # Ok(())
/// # }
/// ```
pub struct ConfigBuilder {
    sources: Vec<Box<dyn ConfigSource>>,
    merge_strategy: MergeStrategy,
    validate: Option<ValidationFn>,
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ConfigBuilder {
    /// Create a new configuration builder.
    pub fn new() -> Self {
        Self {
            sources: Vec::new(),
            merge_strategy: MergeStrategy::Deep,
            validate: None,
        }
    }

    /// Set the merge strategy for combining configuration sources.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::{ConfigBuilder, MergeStrategy};
    ///
    /// let builder = ConfigBuilder::new()
    ///     .with_merge_strategy(MergeStrategy::Replace);
    /// ```
    pub fn with_merge_strategy(mut self, strategy: MergeStrategy) -> Self {
        self.merge_strategy = strategy;
        self
    }

    /// Add a custom configuration source.
    ///
    /// This method allows you to add any type that implements the [`ConfigSource`] trait.
    /// Sources are processed in the order they are added, with later sources potentially
    /// overriding values from earlier sources based on the merge strategy.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::{ConfigBuilder, Environment};
    ///
    /// let env_source = Environment::new().with_prefix("CUSTOM");
    /// let builder = ConfigBuilder::new()
    ///     .add_source(Box::new(env_source));
    /// ```
    pub fn add_source(mut self, source: Box<dyn ConfigSource>) -> Self {
        self.sources.push(source);
        self
    }

    /// Add environment variables with a prefix.
    ///
    /// This is a convenience method that creates an [`Environment`] source with the
    /// specified prefix and default separator (`_`). Environment variables will be
    /// matched using the pattern `{PREFIX}_{FIELD_NAME}`.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::ConfigBuilder;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct Config {
    ///     database_url: String,
    ///     port: u16,
    /// }
    ///
    /// // This will look for APP_DATABASE_URL and APP_PORT
    /// let builder = ConfigBuilder::new()
    ///     .with_env("APP");
    /// ```
    pub fn with_env(self, prefix: impl Into<String>) -> Self {
        let env_source = Environment::new().with_prefix(prefix);
        self.add_source(Box::new(env_source))
    }

    /// Add a custom environment configuration.
    ///
    /// Use this method when you need more control over environment variable parsing,
    /// such as custom separators or case sensitivity settings.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::{ConfigBuilder, Environment};
    ///
    /// let custom_env = Environment::new()
    ///     .with_prefix("MYAPP")
    ///     .separator("__")  // Use double underscore separator
    ///     .case_sensitive(false);
    ///
    /// let builder = ConfigBuilder::new()
    ///     .with_env_custom(custom_env);
    /// ```
    pub fn with_env_custom(self, env: Environment) -> Self {
        self.add_source(Box::new(env))
    }

    /// Add a required configuration file.
    ///
    /// The file format is automatically detected from the file extension:
    /// - `.json` for JSON files
    /// - `.yaml` or `.yml` for YAML files
    /// - `.toml` for TOML files
    ///
    /// Returns an error if the file doesn't exist or can't be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gonfig::ConfigBuilder;
    ///
    /// let builder = ConfigBuilder::new()
    ///     .with_file("config.json")?;
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns [`Error::Io`] if the file cannot be read, or [`Error::Config`]
    /// if the file cannot be parsed.
    pub fn with_file(self, path: impl AsRef<Path>) -> Result<Self> {
        let config = Config::from_file(path)?;
        Ok(self.add_source(Box::new(config)))
    }

    /// Add an optional configuration file.
    ///
    /// Unlike [`with_file`], this method won't return an error if the file doesn't exist.
    /// It will still return an error if the file exists but can't be parsed.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gonfig::ConfigBuilder;
    ///
    /// // This won't fail if config.json doesn't exist
    /// let builder = ConfigBuilder::new()
    ///     .with_file_optional("config.json")?;
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    ///
    /// [`with_file`]: ConfigBuilder::with_file
    pub fn with_file_optional(self, path: impl AsRef<Path>) -> Result<Self> {
        let config = Config::from_file_optional(path)?;
        Ok(self.add_source(Box::new(config)))
    }

    /// Add a configuration file with explicit format.
    ///
    /// Use this method when you need to override the automatic format detection
    /// or when working with files that don't have standard extensions.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gonfig::{ConfigBuilder, ConfigFormat};
    ///
    /// // Force JSON parsing for a file without .json extension
    /// let builder = ConfigBuilder::new()
    ///     .with_file_format("my-config", ConfigFormat::Json)?;
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    pub fn with_file_format(self, path: impl AsRef<Path>, format: ConfigFormat) -> Result<Self> {
        let config = Config::with_format(path, format)?;
        Ok(self.add_source(Box::new(config)))
    }

    /// Add CLI arguments from `std::env::args()`.
    ///
    /// This creates a basic CLI source that parses arguments in the format:
    /// - `--key=value` or `--key value`
    /// - `-k=value` or `-k value` (for single character keys)
    ///
    /// For more advanced CLI parsing with clap, use [`with_clap`] instead.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::ConfigBuilder;
    ///
    /// // Parses CLI args like: --database-url=postgres://... --port=8080
    /// let builder = ConfigBuilder::new()
    ///     .with_cli();
    /// ```
    ///
    /// [`with_clap`]: ConfigBuilder::with_clap
    pub fn with_cli(self) -> Self {
        let cli = Cli::from_args();
        self.add_source(Box::new(cli))
    }

    /// Add a custom CLI configuration.
    pub fn with_cli_custom(self, cli: Cli) -> Self {
        self.add_source(Box::new(cli))
    }

    /// Add CLI arguments using clap parser.
    ///
    /// This method integrates with clap's derive API for advanced CLI argument parsing.
    /// Your struct must implement both `clap::Parser` and `serde::Serialize`.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gonfig::ConfigBuilder;
    /// use serde::{Deserialize, Serialize};
    /// use clap::Parser;
    ///
    /// #[derive(Parser, Serialize, Deserialize)]
    /// struct CliArgs {
    ///     #[arg(long, env = "DATABASE_URL")]
    ///     database_url: String,
    ///     
    ///     #[arg(short, long, default_value = "8080")]
    ///     port: u16,
    /// }
    ///
    /// let builder = ConfigBuilder::new()
    ///     .with_clap::<CliArgs>()?;
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    pub fn with_clap<T: clap::Parser + serde::Serialize>(self) -> Result<Self> {
        let cli = Cli::with_clap_app::<T>()?;
        Ok(self.add_source(Box::new(cli)))
    }

    /// Add default values as a fallback configuration source.
    ///
    /// Default values are applied with the lowest priority, so they will be overridden
    /// by any other configuration source (environment variables, config files, CLI args).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::ConfigBuilder;
    /// use serde_json::json;
    ///
    /// let defaults = json!({
    ///     "port": 8080,
    ///     "debug": false,
    ///     "database": {
    ///         "pool_size": 10
    ///     }
    /// });
    ///
    /// let builder = ConfigBuilder::new()
    ///     .with_defaults(defaults)?;
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    pub fn with_defaults(mut self, defaults: Value) -> Result<Self> {
        // Create a custom source for defaults with lowest priority
        struct DefaultsSource {
            value: Value,
        }

        impl ConfigSource for DefaultsSource {
            fn collect(&self) -> Result<Value> {
                Ok(self.value.clone())
            }

            fn source_type(&self) -> crate::source::Source {
                // Use Default source type which has the lowest priority
                crate::source::Source::Default
            }

            fn has_value(&self, key: &str) -> bool {
                self.value.get(key).is_some()
            }

            fn get_value(&self, key: &str) -> Option<Value> {
                self.value.get(key).cloned()
            }

            fn as_any(&self) -> &dyn std::any::Any {
                self
            }
        }

        // Add defaults as the first source (lowest priority)
        self.sources
            .insert(0, Box::new(DefaultsSource { value: defaults }));
        Ok(self)
    }

    /// Add a validation function that will be called on the final merged configuration.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use gonfig::{ConfigBuilder, Error};
    ///
    /// let builder = ConfigBuilder::new()
    ///     .validate_with(|value| {
    ///         if let Some(port) = value.get("port").and_then(|p| p.as_u64()) {
    ///             if port > 65535 {
    ///                 return Err(Error::Validation("Invalid port number".into()));
    ///             }
    ///         }
    ///         Ok(())
    ///     });
    /// ```
    pub fn validate_with<F>(mut self, validator: F) -> Self
    where
        F: Fn(&Value) -> Result<()> + 'static,
    {
        self.validate = Some(Box::new(validator));
        self
    }

    /// Build the final configuration by merging all sources.
    ///
    /// This method processes all registered sources in order, applies the configured
    /// merge strategy, runs any validation, and deserializes the result into the
    /// target configuration type.
    ///
    /// # Type Parameters
    ///
    /// * `T` - The target configuration type that implements [`serde::de::DeserializeOwned`]
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use gonfig::ConfigBuilder;
    /// use serde::Deserialize;
    ///
    /// #[derive(Deserialize)]
    /// struct AppConfig {
    ///     database_url: String,
    ///     port: u16,
    ///     debug: bool,
    /// }
    ///
    /// let config: AppConfig = ConfigBuilder::new()
    ///     .with_env("APP")
    ///     .with_file_optional("config.json")?
    ///     .with_cli()
    ///     .build()?;
    /// # Ok::<(), gonfig::Error>(())
    /// ```
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Any required configuration source fails to load
    /// - Validation fails
    /// - The final merged configuration cannot be deserialized into type `T`
    pub fn build<T: DeserializeOwned>(self) -> Result<T> {
        let merger = ConfigMerger::new(self.merge_strategy);

        let mut source_values = Vec::new();
        for source in &self.sources {
            let value = source.collect()?;
            let priority = source.source_type().priority();
            source_values.push((value, priority));
        }

        let merged = merger.merge_sources(source_values);

        if let Some(validator) = &self.validate {
            validator(&merged)?;
        }

        serde_json::from_value(merged)
            .map_err(|e| Error::Serialization(format!("Failed to deserialize config: {e}")))
    }

    pub fn build_value(self) -> Result<Value> {
        let merger = ConfigMerger::new(self.merge_strategy);

        let mut source_values = Vec::new();
        for source in &self.sources {
            let value = source.collect()?;
            let priority = source.source_type().priority();
            source_values.push((value, priority));
        }

        let merged = merger.merge_sources(source_values);

        if let Some(validator) = &self.validate {
            validator(&merged)?;
        }

        Ok(merged)
    }

    pub fn sources(&self) -> &[Box<dyn ConfigSource>] {
        &self.sources
    }

    pub fn get_source<T: ConfigSource + 'static>(&self) -> Option<&T> {
        self.sources
            .iter()
            .find_map(|source| source.as_any().downcast_ref::<T>())
    }
}
