/// Test for Issue #18: Environment variables should override nested config file values
///
/// This test verifies that environment variables with nested paths (using separators)
/// properly override corresponding nested values from configuration files when using
/// the Deep merge strategy and nested mode.
use gonfig::{ConfigBuilder, ConfigFormat, Environment, MergeStrategy};
use serde::{Deserialize, Serialize};
use std::env;
use std::io::Write;
use tempfile::NamedTempFile;

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct TestConfig {
    service: ServiceInfo,
    http: HttpSettings,
    database: DatabaseSettings,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct ServiceInfo {
    name: String,
    version: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct HttpSettings {
    host: String,
    port: u16,
    timeout: u32,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct DatabaseSettings {
    host: String,
    port: u16,
    name: String,
    pool: PoolSettings,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
struct PoolSettings {
    minsize: u32,
    maxsize: u32,
}

#[test]
fn test_issue_18_basic_nested_override() -> Result<(), Box<dyn std::error::Error>> {
    // Clean environment first to avoid test interference
    env::remove_var("APP_HTTP_PORT");
    env::remove_var("APP_DATABASE_NAME");
    env::remove_var("APP_DATABASE_POOL_MAXSIZE");
    env::remove_var("APP_SERVICE_VERSION");
    env::remove_var("APP_HTTP_TIMEOUT");
    env::remove_var("APP_DATABASE_HOST");
    env::remove_var("APP_DATABASE_POOL_MINSIZE");

    // Create temp config file with nested structure
    let mut file = NamedTempFile::new()?;
    writeln!(
        file,
        r#"
service:
  name: "TestApp"
  version: "1.0.0"
http:
  host: "127.0.0.1"
  port: 3000
  timeout: 30
database:
  host: "localhost"
  port: 5432
  name: "prod_db"
  pool:
    minsize: 5
    maxsize: 20
"#
    )?;
    file.flush()?;

    // Set environment variables to override nested values
    env::set_var("APP_HTTP_PORT", "9000");
    env::set_var("APP_DATABASE_NAME", "test_db");

    let config: TestConfig = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_file_format(file.path(), ConfigFormat::Yaml)?
        .with_env_custom(Environment::new().with_prefix("APP").nested(true))
        .build()?;

    // Verify env vars overrode config file values
    assert_eq!(
        config.http.port, 9000,
        "HTTP port should be overridden by env var"
    );
    assert_eq!(
        config.database.name, "test_db",
        "Database name should be overridden by env var"
    );

    // Verify non-overridden values remain from file
    assert_eq!(config.http.host, "127.0.0.1");
    assert_eq!(config.http.timeout, 30);
    assert_eq!(config.service.name, "TestApp");

    // Cleanup
    env::remove_var("APP_HTTP_PORT");
    env::remove_var("APP_DATABASE_NAME");

    Ok(())
}

#[test]
fn test_issue_18_deep_nested_override() -> Result<(), Box<dyn std::error::Error>> {
    // Clean environment first to avoid test interference
    env::remove_var("APP_HTTP_PORT");
    env::remove_var("APP_DATABASE_NAME");
    env::remove_var("APP_DATABASE_POOL_MAXSIZE");
    env::remove_var("APP_SERVICE_VERSION");
    env::remove_var("APP_HTTP_TIMEOUT");
    env::remove_var("APP_DATABASE_HOST");
    env::remove_var("APP_DATABASE_POOL_MINSIZE");

    // Test 3-level nesting: database.pool.maxsize
    let mut file = NamedTempFile::new()?;
    writeln!(
        file,
        r#"
service:
  name: "TestApp"
  version: "1.0.0"
http:
  host: "127.0.0.1"
  port: 3000
  timeout: 30
database:
  host: "localhost"
  port: 5432
  name: "prod_db"
  pool:
    minsize: 5
    maxsize: 20
"#
    )?;
    file.flush()?;

    // Set env var for deeply nested value
    env::set_var("APP_DATABASE_POOL_MAXSIZE", "100");

    let config: TestConfig = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_file_format(file.path(), ConfigFormat::Yaml)?
        .with_env_custom(Environment::new().with_prefix("APP").nested(true))
        .build()?;

    // Verify deep nested override
    assert_eq!(
        config.database.pool.maxsize, 100,
        "Deeply nested pool maxsize should be overridden"
    );
    assert_eq!(
        config.database.pool.minsize, 5,
        "Non-overridden nested value should remain"
    );

    // Cleanup
    env::remove_var("APP_DATABASE_POOL_MAXSIZE");

    Ok(())
}

#[test]
fn test_issue_18_multiple_nested_overrides() -> Result<(), Box<dyn std::error::Error>> {
    // Clean environment first to avoid test interference
    env::remove_var("APP_HTTP_PORT");
    env::remove_var("APP_DATABASE_NAME");
    env::remove_var("APP_DATABASE_POOL_MAXSIZE");
    env::remove_var("APP_SERVICE_VERSION");
    env::remove_var("APP_HTTP_TIMEOUT");
    env::remove_var("APP_DATABASE_HOST");
    env::remove_var("APP_DATABASE_POOL_MINSIZE");

    // Test multiple env vars overriding different nested levels
    let mut file = NamedTempFile::new()?;
    writeln!(
        file,
        r#"
service:
  name: "TestApp"
  version: "1.0.0"
http:
  host: "127.0.0.1"
  port: 3000
  timeout: 30
database:
  host: "localhost"
  port: 5432
  name: "prod_db"
  pool:
    minsize: 5
    maxsize: 20
"#
    )?;
    file.flush()?;

    // Override multiple nested values at different levels
    env::set_var("APP_SERVICE_VERSION", "2.0.0");
    env::set_var("APP_HTTP_PORT", "8080");
    env::set_var("APP_HTTP_TIMEOUT", "60");
    env::set_var("APP_DATABASE_HOST", "db.example.com");
    env::set_var("APP_DATABASE_POOL_MINSIZE", "10");
    env::set_var("APP_DATABASE_POOL_MAXSIZE", "50");

    let config: TestConfig = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_file_format(file.path(), ConfigFormat::Yaml)?
        .with_env_custom(Environment::new().with_prefix("APP").nested(true))
        .build()?;

    // Verify all overrides
    assert_eq!(config.service.version, "2.0.0");
    assert_eq!(config.http.port, 8080);
    assert_eq!(config.http.timeout, 60);
    assert_eq!(config.database.host, "db.example.com");
    assert_eq!(config.database.pool.minsize, 10);
    assert_eq!(config.database.pool.maxsize, 50);

    // Verify non-overridden values
    assert_eq!(config.service.name, "TestApp");
    assert_eq!(config.http.host, "127.0.0.1");

    // Cleanup
    env::remove_var("APP_SERVICE_VERSION");
    env::remove_var("APP_HTTP_PORT");
    env::remove_var("APP_HTTP_TIMEOUT");
    env::remove_var("APP_DATABASE_HOST");
    env::remove_var("APP_DATABASE_POOL_MINSIZE");
    env::remove_var("APP_DATABASE_POOL_MAXSIZE");

    Ok(())
}

#[test]
fn test_issue_18_backward_compatibility() -> Result<(), Box<dyn std::error::Error>> {
    // Verify that nested=false (default) maintains backward compatibility
    let mut file = NamedTempFile::new()?;
    writeln!(
        file,
        r#"
service:
  name: "TestApp"
  version: "1.0.0"
http:
  host: "127.0.0.1"
  port: 3000
  timeout: 30
database:
  host: "localhost"
  port: 5432
  name: "prod_db"
  pool:
    minsize: 5
    maxsize: 20
"#
    )?;
    file.flush()?;

    // Set env var that would override in nested mode
    env::set_var("APP_HTTP_PORT", "9000");

    // Build with nested=false (default)
    let config: TestConfig = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_file_format(file.path(), ConfigFormat::Yaml)?
        .with_env_custom(Environment::new().with_prefix("APP").nested(false))
        .build()?;

    // With nested=false, env var won't override nested config value
    // because the environment variable "APP_HTTP_PORT" is interpreted as the flat key "http_port", which does not match the expected nested path "http.port" in the config structure
    assert_eq!(
        config.http.port, 3000,
        "Port should remain from file when nested=false"
    );

    // Cleanup
    env::remove_var("APP_HTTP_PORT");

    Ok(())
}
