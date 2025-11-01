use gonfig::{ConfigBuilder, Environment, MergeStrategy};
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Serialize, Deserialize)]
struct SimpleConfig {
    name: String,
    port: u16,
    debug: bool,
}

fn main() -> gonfig::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    // For this manual approach, we need flat naming
    std::env::set_var("APP_NAME", "Gonfig Test");
    std::env::set_var("APP_PORT", "8080");
    std::env::set_var("APP_DEBUG", "true");

    let env = Environment::new().with_prefix("APP").separator("_");

    let builder = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .add_source(Box::new(env));

    match builder.build::<SimpleConfig>() {
        Ok(config) => {
            tracing::info!("Configuration loaded successfully:");
            tracing::info!("Name: {}", config.name);
            tracing::info!("Port: {}", config.port);
            tracing::info!("Debug: {}", config.debug);
        }
        Err(e) => tracing::error!("Error: {e}"),
    }

    Ok(())
}
