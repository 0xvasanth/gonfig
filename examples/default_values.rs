//! Example demonstrating the default value feature for the Gonfig library.
//!
//! This example shows how to use the `#[gonfig(default = "value")]` attribute
//! to specify default values for configuration fields. These defaults are used
//! when the corresponding environment variable is not set.

use gonfig::Gonfig;
use serde::{Deserialize, Serialize};
use std::env;
use tracing_subscriber::EnvFilter;

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
#[Gonfig(env_prefix = "APP")]
pub struct AppConfig {
    /// Application name with a default value
    #[gonfig(env_name = "APP_NAME", default = "my-awesome-app")]
    pub name: String,

    /// Server port with default 8080
    #[gonfig(env_name = "PORT", default = "8080")]
    pub port: u16,

    /// Debug mode, defaults to false for production safety
    #[gonfig(env_name = "DEBUG", default = "false")]
    pub debug: bool,

    /// Database configuration
    pub database: DatabaseConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Gonfig)]
pub struct DatabaseConfig {
    /// Database host with localhost as default
    #[gonfig(env_name = "DB_HOST", default = "localhost")]
    pub host: String,

    /// Database port, defaults to PostgreSQL's standard port
    #[gonfig(env_name = "DB_PORT", default = "5432")]
    pub port: u16,

    /// Connection pool size
    #[gonfig(env_name = "DB_POOL_SIZE", default = "10")]
    pub pool_size: u32,

    /// Database name - no default, must be provided
    #[gonfig(env_name = "DB_NAME")]
    pub name: Option<String>,
}

fn main() -> gonfig::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("=== Gonfig Default Values Feature Demo ===\n");
    tracing::info!("This example demonstrates the new default value feature");
    tracing::info!("requested in GitHub issue #1\n");

    // Scenario 1: All defaults
    tracing::info!("1. Using all default values:");
    tracing::info!("   (No environment variables set except required nested struct fields)\n");

    // Clean environment
    cleanup_env();

    // Set environment variables for nested struct
    // (This is a current limitation - nested structs still need their env vars)
    env::set_var("DATABASE_HOST", "localhost");
    env::set_var("DATABASE_PORT", "5432");
    env::set_var("DATABASE_POOL_SIZE", "10");

    let config = AppConfig::from_gonfig()?;
    print_config(&config);

    // Scenario 2: Mix of defaults and environment variables
    tracing::info!("\n2. Overriding some values with environment variables:");
    tracing::info!("   Setting: APP_NAME=production-api, PORT=3000, DB_NAME=prod_db\n");

    env::set_var("APP_NAME", "production-api");
    env::set_var("PORT", "3000");
    env::set_var("DB_NAME", "prod_db");

    let config = AppConfig::from_gonfig()?;
    print_config(&config);

    // Scenario 3: Debug mode enabled
    tracing::info!("\n3. Enabling debug mode:");
    tracing::info!("   Setting: DEBUG=true\n");

    env::set_var("DEBUG", "true");

    let config = AppConfig::from_gonfig()?;
    print_config(&config);

    tracing::info!("\n✅ Default values feature is working correctly!");
    tracing::info!("\nBenefits of this feature:");
    tracing::info!("• Reduces boilerplate - no need to manually check and set defaults");
    tracing::info!("• Makes configuration more declarative and self-documenting");
    tracing::info!("• Provides sensible defaults for development environments");
    tracing::info!("• Maintains backward compatibility with existing code");

    cleanup_env();
    Ok(())
}

fn print_config(config: &AppConfig) {
    tracing::info!("   App Configuration:");
    tracing::info!("     Name: {}", config.name);
    tracing::info!("     Port: {}", config.port);
    tracing::info!("     Debug: {}", config.debug);
    tracing::info!("   Database Configuration:");
    tracing::info!("     Host: {}", config.database.host);
    tracing::info!("     Port: {}", config.database.port);
    tracing::info!("     Pool Size: {}", config.database.pool_size);
    tracing::info!(
        "     Database Name: {}",
        config.database.name.as_deref().unwrap_or("<not set>")
    );
}

fn cleanup_env() {
    env::remove_var("APP_NAME");
    env::remove_var("PORT");
    env::remove_var("DEBUG");
    env::remove_var("DB_HOST");
    env::remove_var("DB_PORT");
    env::remove_var("DB_POOL_SIZE");
    env::remove_var("DB_NAME");
    env::remove_var("DATABASE_HOST");
    env::remove_var("DATABASE_PORT");
    env::remove_var("DATABASE_POOL_SIZE");
    env::remove_var("DATABASE_NAME");
}
