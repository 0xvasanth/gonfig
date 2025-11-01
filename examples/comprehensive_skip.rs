/// Comprehensive example demonstrating skip functionality in Gonfig
use gonfig::Gonfig;
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig(env_prefix = "APP")]
struct AppConfig {
    // ✅ Included: Environment variable APP_DATABASE_URL
    database_url: String,

    // ✅ Included: Environment variable APP_PORT
    port: u16,

    // ❌ Skipped: Completely excluded from configuration
    #[skip]
    #[serde(skip)]
    runtime_connection: Option<String>,

    // ❌ Skipped: Alternative syntax
    #[skip_gonfig]
    #[serde(skip)]
    internal_cache: Vec<String>,

    // ✅ Included: Environment variable APP_LOG_LEVEL
    log_level: String,
}

#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig(env_prefix = "DB")]
struct DatabaseConfig {
    // ✅ Included: Environment variable DB_HOST
    host: String,

    // ✅ Included: Environment variable DB_PORT
    port: u16,

    // ❌ Skipped: Password excluded for security
    #[skip]
    #[serde(skip)]
    password: Option<String>,

    // ✅ Included: Environment variable DB_MAX_CONNECTIONS
    max_connections: u32,
}

fn main() -> gonfig::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("=== Comprehensive Skip Demonstration ===\n");

    // Set up environment variables
    std::env::set_var("APP_DATABASE_URL", "postgres://localhost/app");
    std::env::set_var("APP_PORT", "3000");
    std::env::set_var("APP_LOG_LEVEL", "debug");

    // These environment variables will be IGNORED due to #[skip]
    std::env::set_var("APP_RUNTIME_CONNECTION", "ignored_value");
    std::env::set_var("APP_INTERNAL_CACHE", "also_ignored");

    std::env::set_var("DB_HOST", "localhost");
    std::env::set_var("DB_PORT", "5432");
    std::env::set_var("DB_MAX_CONNECTIONS", "20");

    // This will be IGNORED due to #[skip]
    std::env::set_var("DB_PASSWORD", "ignored_password");

    tracing::info!("Environment variables set:");
    tracing::info!("  APP_DATABASE_URL=postgres://localhost/app    ✅ Will be loaded");
    tracing::info!("  APP_PORT=3000                               ✅ Will be loaded");
    tracing::info!("  APP_LOG_LEVEL=debug                         ✅ Will be loaded");
    tracing::info!("  APP_RUNTIME_CONNECTION=ignored_value        ❌ Will be skipped");
    tracing::info!("  APP_INTERNAL_CACHE=also_ignored             ❌ Will be skipped");
    tracing::info!("  DB_HOST=localhost                           ✅ Will be loaded");
    tracing::info!("  DB_PASSWORD=ignored_password                ❌ Will be skipped");
    tracing::info!("");

    // Load AppConfig
    tracing::info!("1. Loading AppConfig:");
    match AppConfig::from_gonfig() {
        Ok(mut config) => {
            tracing::info!("✅ AppConfig loaded successfully:");
            tracing::info!("   Database URL: {}", config.database_url);
            tracing::info!("   Port: {}", config.port);
            tracing::info!("   Log Level: {}", config.log_level);
            tracing::info!(
                "   Runtime Connection: {:?} (skipped field)",
                config.runtime_connection
            );
            tracing::info!(
                "   Internal Cache: {:?} (skipped field)",
                config.internal_cache
            );

            // Manually set skipped fields
            config.runtime_connection = Some("manually_set_connection".to_string());
            config.internal_cache = vec!["manual_entry".to_string()];

            tracing::info!("\n   After manual initialization:");
            tracing::info!("   Runtime Connection: {:?}", config.runtime_connection);
            tracing::info!("   Internal Cache: {:?}", config.internal_cache);
        }
        Err(e) => tracing::error!("❌ Error loading AppConfig: {e}"),
    }

    tracing::info!("\n2. Loading DatabaseConfig:");
    match DatabaseConfig::from_gonfig() {
        Ok(mut config) => {
            tracing::info!("✅ DatabaseConfig loaded successfully:");
            tracing::info!("   Host: {}", config.host);
            tracing::info!("   Port: {}", config.port);
            tracing::info!("   Max Connections: {}", config.max_connections);
            tracing::info!("   Password: {:?} (skipped field)", config.password);

            // Manually set the password from a secure source
            config.password = Some("secure_password_from_vault".to_string());

            tracing::info!("\n   After setting password from secure vault:");
            tracing::info!("   Password: [SET FROM VAULT]");
        }
        Err(e) => tracing::error!("❌ Error loading DatabaseConfig: {e}"),
    }

    tracing::info!("\n3. Skip vs Include Comparison:");
    show_skip_comparison();

    Ok(())
}

fn show_skip_comparison() {
    tracing::info!("Field processing behavior:");
    tracing::info!("");
    tracing::info!("┌─────────────────────────┬─────────────────┬─────────────────┐");
    tracing::info!("│ Field Declaration       │ Environment Var │ Configuration   │");
    tracing::info!("├─────────────────────────┼─────────────────┼─────────────────┤");
    tracing::info!("│ database_url: String    │ APP_DATABASE_URL│ ✅ Loaded       │");
    tracing::info!("│ port: u16              │ APP_PORT        │ ✅ Loaded       │");
    tracing::info!("│ #[skip]                │ APP_RUNTIME_*   │ ❌ Ignored      │");
    tracing::info!("│ runtime_connection     │                 │                 │");
    tracing::info!("│ #[skip_gonfig]         │ APP_INTERNAL_*  │ ❌ Ignored      │");
    tracing::info!("│ internal_cache         │                 │                 │");
    tracing::info!("│ log_level: String      │ APP_LOG_LEVEL   │ ✅ Loaded       │");
    tracing::info!("└─────────────────────────┴─────────────────┴─────────────────┘");
    tracing::info!("");
    tracing::info!("Key benefits of skip attributes:");
    tracing::info!("• Security: Skip sensitive fields (passwords, API keys)");
    tracing::info!("• Runtime data: Skip computed or runtime-only fields");
    tracing::info!("• Non-serializable: Skip complex types that can't be serialized");
    tracing::info!("• Manual control: Initialize certain fields programmatically");
    tracing::info!("• Clean separation: Keep configuration and runtime state separate");
}
