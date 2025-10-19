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

/// Derive macro for automatic configuration management from environment variables, CLI arguments, and config files.
///
/// # Generated Methods
///
/// - `from_gonfig() -> Result<Self>` - Loads configuration from all enabled sources
/// - `from_gonfig_with_builder(builder: ConfigBuilder) -> Result<Self>` - Uses custom builder for advanced configuration
/// - `gonfig_builder() -> ConfigBuilder` - Returns a pre-configured builder
///
/// # Container Attributes
///
/// ## `#[Gonfig(env_prefix = "PREFIX")]`
/// Prefixes environment variables with the given string. Fields are uppercased and appended.
///
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(env_prefix = "APP")]
/// struct Config {
///     database_url: String,  // Reads from APP_DATABASE_URL
///     port: u16,             // Reads from APP_PORT
/// }
/// ```
///
/// ## `#[Gonfig(allow_cli)]`
/// Enables CLI argument parsing with automatic kebab-case conversion.
///
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(allow_cli)]
/// struct Config {
///     max_connections: u32,  // CLI: --max-connections
/// }
/// ```
///
/// ## `#[Gonfig(allow_config)]`
/// Automatically loads from `config.{toml,yaml,json}` in current directory.
///
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(allow_config)]
/// struct Config {
///     setting: String,  // Loaded from config file if present
/// }
/// ```
///
/// # Field Attributes
///
/// ## `#[gonfig(env_name = "CUSTOM_NAME")]`
/// Overrides the environment variable name for a field.
///
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(env_prefix = "APP")]
/// struct Config {
///     #[gonfig(env_name = "DATABASE_CONNECTION_STRING")]
///     database_url: String,  // Uses DATABASE_CONNECTION_STRING (not APP_DATABASE_URL)
/// }
/// ```
///
/// ## `#[gonfig(cli_name = "custom-name")]`
/// Overrides the CLI argument name for a field.
///
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// #[Gonfig(allow_cli)]
/// struct Config {
///     #[gonfig(cli_name = "db-url")]
///     database_url: String,  // CLI: --db-url (not --database-url)
/// }
/// ```
///
/// ## `#[gonfig(default = "value")]`
/// Sets a default value (JSON-compatible string).
///
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
/// Excludes field from configuration loading.
///
/// ```rust,ignore
/// #[derive(Gonfig, Deserialize)]
/// struct Config {
///     database_url: String,
///
///     #[skip]
///     #[serde(skip)]
///     runtime_data: Option<String>,  // Not loaded from config
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
///     database_url: String,              // MYAPP_DATABASE_URL, --database-url
///
///     #[gonfig(default = "8080")]
///     port: u16,                         // MYAPP_PORT, --port, default: 8080
///
///     #[gonfig(env_name = "LOG_LEVEL")]
///     log_level: String,                 // LOG_LEVEL (custom name)
///
///     #[skip]
///     #[serde(skip)]
///     start_time: Option<std::time::Instant>,  // Runtime-only field
/// }
///
/// fn main() -> gonfig::Result<()> {
///     // Simple: loads from all enabled sources
///     let config = AppConfig::from_gonfig()?;
///
///     // Advanced: custom builder with additional config file
///     let mut builder = AppConfig::gonfig_builder();
///     builder = builder.with_file("custom.toml")?;
///     let config = AppConfig::from_gonfig_with_builder(builder)?;
///
///     Ok(())
/// }
/// ```
///
/// # Attribute Reference
///
/// - `Gonfig` - Container attribute for struct options (env_prefix, allow_cli, allow_config)
/// - `gonfig` - Field attribute for customization (env_name, cli_name, default)
/// - `skip` / `skip_gonfig` - Field attribute to exclude from configuration
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

    let allow_cli = opts.allow_cli;
    let allow_config = opts.allow_config;

    let env_prefix = opts.env_prefix.as_ref().cloned().unwrap_or_default();

    let fields = opts
        .data
        .as_ref()
        .take_struct()
        .expect("Only structs are supported")
        .fields;

    let mut regular_mappings = Vec::new();
    let mut default_mappings = Vec::new();

    for f in fields.iter().filter(|f| !f.skip_gonfig && !f.skip) {
        let field_name = f.ident.as_ref().unwrap();
        let field_str = field_name.to_string();

        // Generate environment variable name
        let env_key = f.env_name.clone().unwrap_or_else(|| {
            let upper = field_str.to_uppercase();
            if env_prefix.is_empty() {
                upper
            } else {
                format!("{env_prefix}_{upper}")
            }
        });

        // Generate CLI argument name
        let cli_key = f
            .cli_name
            .clone()
            .unwrap_or_else(|| field_str.replace('_', "-"));

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

    // Shared logic for configuring environment and CLI sources
    let setup_env_cli = quote! {
        let field_mappings: Vec<(String, String, String)> = vec![#(#regular_mappings),*];

        // Environment is always enabled
        let mut env = ::gonfig::Environment::new();
        if !#env_prefix.is_empty() {
            env = env.with_prefix(#env_prefix);
        }
        for (field_name, env_key, _) in &field_mappings {
            env = env.with_field_mapping(field_name, env_key);
        }
        builder = builder.with_env_custom(env);

        if #allow_cli {
            let mut cli = ::gonfig::Cli::from_args();
            for (field_name, _, cli_key) in &field_mappings {
                cli = cli.with_field_mapping(field_name, cli_key);
            }
            builder = builder.with_cli_custom(cli);
        }
    };

    quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn from_gonfig() -> ::gonfig::Result<Self> {
                Self::from_gonfig_with_builder(::gonfig::ConfigBuilder::new())
            }

            pub fn from_gonfig_with_builder(mut builder: ::gonfig::ConfigBuilder) -> ::gonfig::Result<Self> {
                #setup_env_cli

                if #allow_config {
                    use std::path::Path;

                    // Try loading config files in order of preference
                    let config_files = ["config.toml", "config.yaml", "config.json"];
                    for config_file in config_files {
                        if Path::new(config_file).exists() {
                            builder = builder.with_file(config_file)?;
                            break;
                        }
                    }
                }

                // Apply default values
                let default_values: Vec<(String, String)> = vec![#(#default_mappings),*];
                if !default_values.is_empty() {
                    let mut defaults_json = ::serde_json::Map::new();
                    for (field_name, default_value) in default_values {
                        let value = default_value.parse::<::serde_json::Value>()
                            .unwrap_or_else(|_| ::serde_json::Value::String(default_value));
                        defaults_json.insert(field_name, value);
                    }
                    builder = builder.with_defaults(::serde_json::Value::Object(defaults_json))?;
                }

                builder.build::<Self>()
            }

            pub fn gonfig_builder() -> ::gonfig::ConfigBuilder {
                let mut builder = ::gonfig::ConfigBuilder::new();
                #setup_env_cli
                // Note: Config file and defaults not supported here due to Result handling
                builder
            }
        }
    }
}
