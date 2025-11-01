use gonfig::{ConfigBuilder, ConfigFormat, Environment, MergeStrategy};
use serde::{Deserialize, Serialize};
use std::io::Write;
use tempfile::NamedTempFile;
use tracing_subscriber::EnvFilter;

/// Multi-level nested configuration structure
#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct AppConfig {
    service: ServiceConfig,
    http: HttpConfig,
    database: DatabaseConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ServiceConfig {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct HttpConfig {
    host: String,
    port: u16,
    timeout: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DatabaseConfig {
    host: String,
    port: u16,
    name: String,
    pool: PoolConfig,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PoolConfig {
    minsize: u32,
    maxsize: u32,
}

fn main() -> gonfig::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("=== Issue #18 Verification: Nested Environment Variable Override ===\n");

    // Create a temporary config file with nested values
    let mut config_file = NamedTempFile::new().expect("Failed to create temp file");
    writeln!(
        config_file,
        r#"
service:
  name: "MyApp"
  version: "1.0.0"

http:
  host: "127.0.0.1"
  port: 3000
  timeout: 30

database:
  host: "localhost"
  port: 5432
  name: "production_db"
  pool:
    minsize: 5
    maxsize: 20
"#
    )
    .expect("Failed to write config");
    config_file.flush().expect("Failed to flush");

    tracing::info!("1. Loading configuration FROM FILE ONLY:");
    tracing::info!("   Config file path: {:?}", config_file.path());

    let config_file_only: AppConfig = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_file_format(config_file.path(), ConfigFormat::Yaml)?
        .build()?;

    tracing::info!(
        "   Service: {} v{}",
        config_file_only.service.name,
        config_file_only.service.version
    );
    tracing::info!(
        "   HTTP: {}:{} (timeout: {}s)",
        config_file_only.http.host,
        config_file_only.http.port,
        config_file_only.http.timeout
    );
    tracing::info!(
        "   Database: {}:{}/{}",
        config_file_only.database.host,
        config_file_only.database.port,
        config_file_only.database.name
    );
    tracing::info!(
        "   Pool: min={}, max={}",
        config_file_only.database.pool.minsize,
        config_file_only.database.pool.maxsize
    );

    // Verify file values
    assert_eq!(
        config_file_only.http.port, 3000,
        "File config should have port 3000"
    );
    assert_eq!(
        config_file_only.database.pool.maxsize, 20,
        "File config should have maxsize 20"
    );

    tracing::info!("\n2. Setting ENVIRONMENT VARIABLES to override nested values:");
    std::env::set_var("APP_HTTP_PORT", "9000");
    std::env::set_var("APP_HTTP_TIMEOUT", "60");
    std::env::set_var("APP_DATABASE_POOL_MAXSIZE", "50");
    std::env::set_var("APP_DATABASE_NAME", "staging_db");

    tracing::info!("   APP_HTTP_PORT=9000");
    tracing::info!("   APP_HTTP_TIMEOUT=60");
    tracing::info!("   APP_DATABASE_POOL_MAXSIZE=50");
    tracing::info!("   APP_DATABASE_NAME=staging_db");

    tracing::info!("\n3. Loading configuration WITH NESTED ENV OVERRIDE:");
    let config_with_env: AppConfig = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_file_format(config_file.path(), ConfigFormat::Yaml)?
        .with_env_custom(Environment::new().with_prefix("APP").nested(true))
        .build()?;

    tracing::info!(
        "   Service: {} v{}",
        config_with_env.service.name,
        config_with_env.service.version
    );
    tracing::info!(
        "   HTTP: {}:{} (timeout: {}s)",
        config_with_env.http.host,
        config_with_env.http.port,
        config_with_env.http.timeout
    );
    tracing::info!(
        "   Database: {}:{}/{}",
        config_with_env.database.host,
        config_with_env.database.port,
        config_with_env.database.name
    );
    tracing::info!(
        "   Pool: min={}, max={}",
        config_with_env.database.pool.minsize,
        config_with_env.database.pool.maxsize
    );

    tracing::info!("\n4. VERIFICATION RESULTS:");

    // Critical assertion: env vars should override file values
    if config_with_env.http.port == 9000 {
        tracing::info!("   ✅ PASS: HTTP port overridden by env (9000)");
    } else {
        tracing::error!(
            "   ❌ FAIL: HTTP port NOT overridden (expected 9000, got {})",
            config_with_env.http.port
        );
        panic!("Issue #18 NOT FIXED: Environment variable failed to override nested config value");
    }

    if config_with_env.http.timeout == 60 {
        tracing::info!("   ✅ PASS: HTTP timeout overridden by env (60)");
    } else {
        tracing::error!(
            "   ❌ FAIL: HTTP timeout NOT overridden (expected 60, got {})",
            config_with_env.http.timeout
        );
        panic!("Issue #18 NOT FIXED: Environment variable failed to override nested config value");
    }

    if config_with_env.database.pool.maxsize == 50 {
        tracing::info!("   ✅ PASS: Database pool maxsize overridden by env (50)");
    } else {
        tracing::error!(
            "   ❌ FAIL: Database pool maxsize NOT overridden (expected 50, got {})",
            config_with_env.database.pool.maxsize
        );
        panic!("Issue #18 NOT FIXED: Environment variable failed to override deeply nested config value");
    }

    if config_with_env.database.name == "staging_db" {
        tracing::info!("   ✅ PASS: Database name overridden by env (staging_db)");
    } else {
        tracing::error!(
            "   ❌ FAIL: Database name NOT overridden (expected staging_db, got {})",
            config_with_env.database.name
        );
        panic!("Issue #18 NOT FIXED: Environment variable failed to override nested config value");
    }

    // Verify non-overridden values remain from file
    assert_eq!(
        config_with_env.http.host, "127.0.0.1",
        "Non-overridden values should remain from file"
    );
    assert_eq!(
        config_with_env.service.name, "MyApp",
        "Non-overridden values should remain from file"
    );
    tracing::info!("   ✅ PASS: Non-overridden values preserved from config file");

    tracing::info!("\n5. Testing WITHOUT nested mode (backward compatibility):");
    let config_flat: Result<AppConfig, _> = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_file_format(config_file.path(), ConfigFormat::Yaml)?
        .with_env_custom(Environment::new().with_prefix("APP").nested(false))
        .build();

    match config_flat {
        Ok(cfg) => {
            tracing::info!("   Config loaded with nested=false");
            tracing::info!(
                "   HTTP port: {} (should be from file: 3000)",
                cfg.http.port
            );
            if cfg.http.port == 3000 {
                tracing::info!(
                    "   ✅ PASS: Backward compatibility maintained - nested=false keeps flat keys"
                );
            }
        }
        Err(e) => {
            tracing::info!("   Note: Config might fail without nested mode (expected): {e}");
        }
    }

    tracing::info!("\n=== CONCLUSION ===");
    tracing::info!("✅ Issue #18 is FIXED!");
    tracing::info!("   Environment variables now properly override nested config file values");
    tracing::info!("   when using .nested(true) with Deep merge strategy.");

    // Clean up
    std::env::remove_var("APP_HTTP_PORT");
    std::env::remove_var("APP_HTTP_TIMEOUT");
    std::env::remove_var("APP_DATABASE_POOL_MAXSIZE");
    std::env::remove_var("APP_DATABASE_NAME");

    Ok(())
}
