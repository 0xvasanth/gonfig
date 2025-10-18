use darling::{FromDeriveInput, FromField};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[derive(Debug, FromDeriveInput)]
#[darling(attributes(gonfig, Gonfig))]
struct GonfigOpts {
    ident: syn::Ident,
    generics: syn::Generics,
    data: darling::ast::Data<(), GonfigField>,

    #[darling(default)]
    env_prefix: Option<String>,

    #[darling(default)]
    allow_cli: bool,

    #[darling(default)]
    allow_config: bool,
}

#[derive(Debug, FromField)]
#[darling(attributes(gonfig, skip_gonfig, skip))]
struct GonfigField {
    ident: Option<syn::Ident>,

    // Reserved for future use (flatten feature)
    #[allow(dead_code)]
    ty: syn::Type,

    #[darling(default)]
    env_name: Option<String>,

    #[darling(default)]
    cli_name: Option<String>,

    #[darling(default)]
    skip_gonfig: bool,

    #[darling(default)]
    skip: bool,

    // Reserved for future use (flatten feature)
    #[allow(dead_code)]
    #[darling(default)]
    flatten: bool,

    #[darling(default)]
    default: Option<String>,
}

/// Derive macro for the `Gonfig` trait, enabling declarative configuration management.
///
/// This macro generates configuration loading methods for your struct, supporting multiple
/// configuration sources: environment variables, CLI arguments, and configuration files.
///
/// # Generated Methods
///
/// The macro generates three public methods on your struct:
///
/// - `from_gonfig() -> Result<Self>` - Loads configuration from all enabled sources
/// - `from_gonfig_with_builder(builder: ConfigBuilder) -> Result<Self>` - Advanced configuration with custom builder
/// - `gonfig_builder() -> ConfigBuilder` - Returns a pre-configured builder for advanced use cases
///
/// # Container Attributes
///
/// ## `#[Gonfig(env_prefix = "PREFIX")]`
/// Sets the prefix for environment variables. Field names are automatically uppercased and
/// appended to the prefix.
///
/// **Example:**
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(env_prefix = "APP")]
/// struct Config {
///     database_url: String,  // Environment variable: APP_DATABASE_URL
///     port: u16,             // Environment variable: APP_PORT
/// }
/// ```
///
/// ## `#[Gonfig(allow_cli)]`
/// Enables CLI argument parsing. Field names are converted to kebab-case.
///
/// **Example:**
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(allow_cli)]
/// struct Config {
///     max_connections: u32,  // CLI argument: --max-connections
/// }
/// ```
///
/// ## `#[Gonfig(allow_config)]`
/// Enables automatic config file loading. Checks for `config.toml`, `config.yaml`, or
/// `config.json` in the current directory.
///
/// **Example:**
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(allow_config)]
/// struct Config {
///     // Loads from config.toml, config.yaml, or config.json if present
///     setting: String,
/// }
/// ```
///
/// # Field Attributes
///
/// ## `#[gonfig(env_name = "CUSTOM_NAME")]`
/// Override the environment variable name for a specific field.
///
/// **Example:**
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(env_prefix = "APP")]
/// struct Config {
///     #[gonfig(env_name = "DATABASE_CONNECTION_STRING")]
///     database_url: String,  // Uses DATABASE_CONNECTION_STRING instead of APP_DATABASE_URL
/// }
/// ```
///
/// ## `#[gonfig(cli_name = "custom-name")]`
/// Override the CLI argument name for a specific field.
///
/// **Example:**
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(allow_cli)]
/// struct Config {
///     #[gonfig(cli_name = "db-url")]
///     database_url: String,  // CLI argument: --db-url instead of --database-url
/// }
/// ```
///
/// ## `#[gonfig(default = "value")]`
/// Specify a default value for a field. The value should be a JSON-compatible string.
///
/// **Example:**
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// struct Config {
///     #[gonfig(default = "8080")]
///     port: u16,
///
///     #[gonfig(default = r#"["localhost"]"#)]
///     allowed_hosts: Vec<String>,
/// }
/// ```
///
/// ## `#[skip]` or `#[skip_gonfig]`
/// Exclude a field from configuration loading. Useful for non-serializable fields or
/// fields that should only be set at runtime.
///
/// **Example:**
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// struct Config {
///     database_url: String,
///
///     #[skip]
///     #[serde(skip)]
///     runtime_data: Option<String>,  // Not loaded from config sources
/// }
/// ```
///
/// # Configuration Priority
///
/// Configuration sources are merged in the following priority order (later sources override earlier ones):
///
/// 1. Default values (from `#[gonfig(default)]` attributes)
/// 2. Configuration files (if `allow_config` is set)
/// 3. Environment variables (always enabled)
/// 4. CLI arguments (if `allow_cli` is set)
///
/// # Complete Example
///
/// ```rust,ignore
/// use gonfig::Gonfig;
/// use serde::Deserialize;
///
/// #[derive(Debug, Deserialize, Gonfig)]
/// #[Gonfig(env_prefix = "MYAPP", allow_cli, allow_config)]
/// struct AppConfig {
///     /// Database connection URL
///     /// - Environment: MYAPP_DATABASE_URL
///     /// - CLI: --database-url
///     database_url: String,
///
///     /// Server port (default: 8080)
///     /// - Environment: MYAPP_PORT
///     /// - CLI: --port
///     #[gonfig(default = "8080")]
///     port: u16,
///
///     /// Custom environment variable name
///     #[gonfig(env_name = "LOG_LEVEL")]
///     log_level: String,
///
///     /// Runtime field (not loaded from config)
///     #[skip]
///     #[serde(skip)]
///     start_time: Option<std::time::Instant>,
/// }
///
/// fn main() -> gonfig::Result<()> {
///     // Simple usage
///     let config = AppConfig::from_gonfig()?;
///
///     // Advanced usage with custom builder
///     let mut builder = AppConfig::gonfig_builder();
///     builder = builder.with_file("custom.toml")?;
///     let config = AppConfig::from_gonfig_with_builder(builder)?;
///
///     println!("Config: {:?}", config);
///     Ok(())
/// }
/// ```
///
/// # Supported Attributes
///
/// - `gonfig` - Field-level attribute for configuration options
/// - `skip_gonfig` - Field-level attribute to skip a field
/// - `skip` - Alternative field-level skip attribute (compatible with serde)
/// - `Gonfig` - Container-level attribute for struct-wide options
#[proc_macro_derive(Gonfig, attributes(gonfig, skip_gonfig, skip, Gonfig))]
pub fn derive_gonfig(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let opts = match GonfigOpts::from_derive_input(&input) {
        Ok(opts) => opts,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let expanded = generate_gonfig_impl(&opts);
    TokenStream::from(expanded)
}

fn generate_gonfig_impl(opts: &GonfigOpts) -> proc_macro2::TokenStream {
    let name = &opts.ident;
    let (impl_generics, ty_generics, where_clause) = opts.generics.split_for_impl();

    let allow_env = true; // Always enable environment variables by default
    let allow_cli = opts.allow_cli;
    let allow_config = opts.allow_config;

    let env_prefix = opts.env_prefix.as_ref().cloned().unwrap_or_default();

    let fields = opts
        .data
        .as_ref()
        .take_struct()
        .expect("Only structs are supported")
        .fields;

    // Separate regular fields from flattened fields
    let mut regular_mappings = Vec::new();
    let mut default_mappings = Vec::new();

    for f in fields.iter().filter(|f| !f.skip_gonfig && !f.skip) {
        let field_name = f.ident.as_ref().unwrap();
        let field_str = field_name.to_string();

        // Note: flatten feature is not yet fully implemented
        // For now, treat all fields as regular fields
        {
            // Generate expected environment variable name
            let env_key = if let Some(custom_name) = &f.env_name {
                // Use custom name directly if provided
                custom_name.clone()
            } else if !env_prefix.is_empty() {
                // Use prefix + field name pattern
                format!("{}_{}", env_prefix, field_str.to_uppercase())
            } else {
                // Just field name in uppercase
                field_str.to_uppercase()
            };

            // Generate CLI argument name (kebab-case)
            let cli_key = if let Some(custom_name) = &f.cli_name {
                custom_name.clone()
            } else {
                field_str.replace('_', "-")
            };

            regular_mappings.push(quote! {
                (#field_str.to_string(), #env_key.to_string(), #cli_key.to_string())
            });

            // Handle default values
            if let Some(default_value) = &f.default {
                default_mappings.push(quote! {
                    (#field_str.to_string(), #default_value.to_string())
                });
            }
        }
    }

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn from_gonfig() -> ::gonfig::Result<Self> {
                Self::from_gonfig_with_builder(::gonfig::ConfigBuilder::new())
            }

            pub fn from_gonfig_with_builder(mut builder: ::gonfig::ConfigBuilder) -> ::gonfig::Result<Self> {
                // Regular field mappings: (field_name, env_key, cli_key)
                let field_mappings: Vec<(String, String, String)> = vec![#(#regular_mappings),*];

                // Default value mappings: (field_name, default_value)
                let default_values: Vec<(String, String)> = vec![#(#default_mappings),*];

                if #allow_env {
                    // Create custom environment source with field mappings
                    let mut env = ::gonfig::Environment::new();

                    if !#env_prefix.is_empty() {
                        env = env.with_prefix(#env_prefix);
                    }

                    // Apply field-level mappings for regular fields
                    for (field_name, env_key, _cli_key) in &field_mappings {
                        env = env.with_field_mapping(field_name, env_key);
                    }

                    builder = builder.with_env_custom(env);
                }

                if #allow_cli {
                    // Create custom CLI source with field mappings
                    let mut cli = ::gonfig::Cli::from_args();

                    // Apply field-level CLI mappings for regular fields
                    for (field_name, _env_key, cli_key) in &field_mappings {
                        cli = cli.with_field_mapping(field_name, cli_key);
                    }

                    builder = builder.with_cli_custom(cli);
                }

                if #allow_config {
                    // Config file support - check for default config files
                    use std::path::Path;

                    if Path::new("config.toml").exists() {
                        builder = match builder.with_file("config.toml") {
                            Ok(b) => b,
                            Err(e) => return Err(e),
                        };
                    } else if Path::new("config.yaml").exists() {
                        builder = match builder.with_file("config.yaml") {
                            Ok(b) => b,
                            Err(e) => return Err(e),
                        };
                    } else if Path::new("config.json").exists() {
                        builder = match builder.with_file("config.json") {
                            Ok(b) => b,
                            Err(e) => return Err(e),
                        };
                    }
                }

                // Apply default values
                if !default_values.is_empty() {
                    let mut defaults_json = ::serde_json::Map::new();
                    for (field_name, default_value) in default_values {
                        // Try to parse as JSON first, otherwise use as string
                        let value = default_value.parse::<::serde_json::Value>()
                            .unwrap_or_else(|_| ::serde_json::Value::String(default_value));
                        defaults_json.insert(field_name, value);
                    }
                    builder = builder.with_defaults(::serde_json::Value::Object(defaults_json))?;
                }

                // Build the final configuration with explicit type
                builder.build::<Self>()
            }

            pub fn gonfig_builder() -> ::gonfig::ConfigBuilder {
                let mut builder = ::gonfig::ConfigBuilder::new();

                // Regular field mappings: (field_name, env_key, cli_key)
                let field_mappings: Vec<(String, String, String)> = vec![#(#regular_mappings),*];

                if #allow_env {
                    // Create custom environment source with field mappings
                    let mut env = ::gonfig::Environment::new();

                    if !#env_prefix.is_empty() {
                        env = env.with_prefix(#env_prefix);
                    }

                    // Apply field-level mappings for regular fields
                    for (field_name, env_key, _cli_key) in &field_mappings {
                        env = env.with_field_mapping(field_name, env_key);
                    }

                    builder = builder.with_env_custom(env);
                }

                if #allow_cli {
                    // Create custom CLI source with field mappings
                    let mut cli = ::gonfig::Cli::from_args();

                    // Apply field-level CLI mappings for regular fields
                    for (field_name, _env_key, cli_key) in &field_mappings {
                        cli = cli.with_field_mapping(field_name, cli_key);
                    }

                    builder = builder.with_cli_custom(cli);
                }

                // Note: Config file loading and defaults are not supported in gonfig_builder()
                // due to Result handling requirements. Use from_gonfig_with_builder() instead
                // for full config file and default value support.

                builder
            }
        }
    }
}
