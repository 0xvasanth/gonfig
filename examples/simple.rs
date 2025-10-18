use gonfig::{ConfigBuilder, Environment, MergeStrategy};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct SimpleConfig {
    name: String,
    port: u16,
    debug: bool,
}

fn main() -> gonfig::Result<()> {
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
            println!("Configuration loaded successfully:");
            println!("Name: {}", config.name);
            println!("Port: {}", config.port);
            println!("Debug: {}", config.debug);
        }
        Err(e) => eprintln!("Error: {e}"),
    }

    Ok(())
}
