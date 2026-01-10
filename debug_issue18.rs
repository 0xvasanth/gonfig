// Debug test to understand what's happening with issue #18
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
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

    println!("Building config with nested environment...");
    let config: TestConfig = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_file_format(file.path(), ConfigFormat::Yaml)?
        .with_env_custom(Environment::new().with_prefix("APP").nested(true))
        .build()?;

    println!("Config built successfully!");
    println!("HTTP port: {} (expected: 9000)", config.http.port);
    println!(
        "Database name: {} (expected: test_db)",
        config.database.name
    );
    println!("HTTP timeout: {} (expected: 30)", config.http.timeout);
    println!("HTTP host: {} (expected: 127.0.0.1)", config.http.host);

    // Cleanup
    env::remove_var("APP_HTTP_PORT");
    env::remove_var("APP_DATABASE_NAME");

    Ok(())
}
