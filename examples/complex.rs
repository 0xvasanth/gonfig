use gonfig::{ConfigBuilder, Environment, MergeStrategy};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
struct AppConfig {
    app_name: String,
    version: String,
    features: Features,
    databases: HashMap<String, DatabaseConfig>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Features {
    auth_enabled: bool,
    rate_limiting: bool,
    max_requests_per_minute: u32,
}

#[derive(Debug, Serialize, Deserialize)]
struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,
    #[serde(skip_serializing)]
    #[allow(dead_code)]
    password: Option<String>,
}

fn main() -> gonfig::Result<()> {
    std::env::set_var("APP_APP_NAME", "MyApp");
    std::env::set_var("APP_VERSION", "2.0.0");
    std::env::set_var("APP_FEATURES_AUTH_ENABLED", "true");
    std::env::set_var("APP_FEATURES_RATE_LIMITING", "true");
    std::env::set_var("APP_FEATURES_MAX_REQUESTS_PER_MINUTE", "100");
    std::env::set_var("APP_DATABASES_PRIMARY_HOST", "primary.db.local");
    std::env::set_var("APP_DATABASES_PRIMARY_PORT", "5432");
    std::env::set_var("APP_DATABASES_PRIMARY_USERNAME", "admin");
    std::env::set_var("APP_DATABASES_PRIMARY_PASSWORD", "secret123");

    let env = Environment::new()
        .with_prefix("APP")
        .separator("_")
        .case_sensitive(false);

    let builder = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_file_optional("examples/config.toml")?
        .add_source(Box::new(env))
        .with_cli()
        .validate_with(|value| {
            if let Some(features) = value.get("features") {
                if let Some(max_req) = features.get("max_requests_per_minute") {
                    if let Some(max_req_num) = max_req.as_u64() {
                        if max_req_num == 0 {
                            return Err(gonfig::Error::Validation(
                                "max_requests_per_minute must be greater than 0".into(),
                            ));
                        }
                    }
                }
            }
            Ok(())
        });

    let value = builder.build_value()?;

    match serde_json::from_value::<AppConfig>(value.clone()) {
        Ok(config) => {
            println!("Loaded configuration:");
            println!("App: {} v{}", config.app_name, config.version);
            println!("\nFeatures:");
            println!("  Auth enabled: {}", config.features.auth_enabled);
            println!("  Rate limiting: {}", config.features.rate_limiting);
            println!(
                "  Max requests/min: {}",
                config.features.max_requests_per_minute
            );

            println!("\nDatabases:");
            for (name, db) in &config.databases {
                println!(
                    "  {}: {}:{} (user: {})",
                    name, db.host, db.port, db.username
                );
            }
        }
        Err(e) => eprintln!("Configuration error: {e}"),
    }
    println!("\nRaw merged configuration:");
    match serde_json::to_string_pretty(&value) {
        Ok(json_str) => println!("{json_str}"),
        Err(e) => eprintln!("Failed to serialize to JSON: {e}"),
    }

    Ok(())
}
