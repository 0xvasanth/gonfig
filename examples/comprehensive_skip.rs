/// Comprehensive example demonstrating skip functionality in Gonfig
use gonfig::Gonfig;
use serde::{Deserialize, Serialize};

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
    println!("=== Comprehensive Skip Demonstration ===\n");

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

    println!("Environment variables set:");
    println!("  APP_DATABASE_URL=postgres://localhost/app    ✅ Will be loaded");
    println!("  APP_PORT=3000                               ✅ Will be loaded");
    println!("  APP_LOG_LEVEL=debug                         ✅ Will be loaded");
    println!("  APP_RUNTIME_CONNECTION=ignored_value        ❌ Will be skipped");
    println!("  APP_INTERNAL_CACHE=also_ignored             ❌ Will be skipped");
    println!("  DB_HOST=localhost                           ✅ Will be loaded");
    println!("  DB_PASSWORD=ignored_password                ❌ Will be skipped");
    println!();

    // Load AppConfig
    println!("1. Loading AppConfig:");
    match AppConfig::from_gonfig() {
        Ok(mut config) => {
            println!("✅ AppConfig loaded successfully:");
            println!("   Database URL: {}", config.database_url);
            println!("   Port: {}", config.port);
            println!("   Log Level: {}", config.log_level);
            println!(
                "   Runtime Connection: {:?} (skipped field)",
                config.runtime_connection
            );
            println!(
                "   Internal Cache: {:?} (skipped field)",
                config.internal_cache
            );

            // Manually set skipped fields
            config.runtime_connection = Some("manually_set_connection".to_string());
            config.internal_cache = vec!["manual_entry".to_string()];

            println!("\n   After manual initialization:");
            println!("   Runtime Connection: {:?}", config.runtime_connection);
            println!("   Internal Cache: {:?}", config.internal_cache);
        }
        Err(e) => println!("❌ Error loading AppConfig: {e}"),
    }

    println!("\n2. Loading DatabaseConfig:");
    match DatabaseConfig::from_gonfig() {
        Ok(mut config) => {
            println!("✅ DatabaseConfig loaded successfully:");
            println!("   Host: {}", config.host);
            println!("   Port: {}", config.port);
            println!("   Max Connections: {}", config.max_connections);
            println!("   Password: {:?} (skipped field)", config.password);

            // Manually set the password from a secure source
            config.password = Some("secure_password_from_vault".to_string());

            println!("\n   After setting password from secure vault:");
            println!("   Password: [SET FROM VAULT]");
        }
        Err(e) => println!("❌ Error loading DatabaseConfig: {e}"),
    }

    println!("\n3. Skip vs Include Comparison:");
    show_skip_comparison();

    Ok(())
}

fn show_skip_comparison() {
    println!("Field processing behavior:");
    println!();
    println!("┌─────────────────────────┬─────────────────┬─────────────────┐");
    println!("│ Field Declaration       │ Environment Var │ Configuration   │");
    println!("├─────────────────────────┼─────────────────┼─────────────────┤");
    println!("│ database_url: String    │ APP_DATABASE_URL│ ✅ Loaded       │");
    println!("│ port: u16              │ APP_PORT        │ ✅ Loaded       │");
    println!("│ #[skip]                │ APP_RUNTIME_*   │ ❌ Ignored      │");
    println!("│ runtime_connection     │                 │                 │");
    println!("│ #[skip_gonfig]         │ APP_INTERNAL_*  │ ❌ Ignored      │");
    println!("│ internal_cache         │                 │                 │");
    println!("│ log_level: String      │ APP_LOG_LEVEL   │ ✅ Loaded       │");
    println!("└─────────────────────────┴─────────────────┴─────────────────┘");
    println!();
    println!("Key benefits of skip attributes:");
    println!("• Security: Skip sensitive fields (passwords, API keys)");
    println!("• Runtime data: Skip computed or runtime-only fields");
    println!("• Non-serializable: Skip complex types that can't be serialized");
    println!("• Manual control: Initialize certain fields programmatically");
    println!("• Clean separation: Keep configuration and runtime state separate");
}
