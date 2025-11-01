use gonfig::Gonfig;
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig(allow_cli)]
struct Mongo {
    // expected ENV variable - MD_MONGO_USERNAME
    // cli argument should be based on the structname and attribute
    username: String,
    // expected ENV variable - MD_MONGO_PASSWORD
    password: String,
}

#[derive(Debug, Serialize, Deserialize, Gonfig)]
struct Application {
    // expected ENV variable - MD_USERNAME
    username: String,
    // expected ENV variable - MD_PASSWORD
    password: String,

    #[skip_gonfig]
    client: Option<String>, // Using Option<String> instead of Client for demo
}

#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig(env_prefix = "MD")]
pub struct Config {
    mongo: Mongo,
    app: Application,
}

fn main() -> gonfig::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("=== Your Configuration Management Example ===\n");

    // Set up environment variables as expected by your design
    setup_environment_variables();

    tracing::info!("1. Loading Config with hierarchical environment variables:");
    match Config::from_gonfig() {
        Ok(config) => {
            tracing::info!("✅ Successfully loaded configuration:");
            print_config(&config);
        }
        Err(e) => tracing::error!("❌ Error loading config: {e}"),
    }

    tracing::info!("\n2. Testing individual component loading:");

    // Test Mongo component with CLI support
    tracing::info!("Mongo configuration (supports CLI):");
    std::env::set_var("MD_MONGO_USERNAME", "mongo_user");
    std::env::set_var("MD_MONGO_PASSWORD", "mongo_pass");

    match Mongo::from_gonfig() {
        Ok(mongo) => {
            tracing::info!("  Username: {}", mongo.username);
            tracing::info!("  Password: [REDACTED]");
        }
        Err(e) => tracing::error!("  Error: {e}"),
    }

    // Test Application component
    tracing::info!("\nApplication configuration:");
    match Application::from_gonfig() {
        Ok(app) => {
            tracing::info!("  Username: {}", app.username);
            tracing::info!("  Password: [REDACTED]");
            tracing::info!("  Client: {:?} (skipped in gonfig)", app.client);
        }
        Err(e) => tracing::error!("  Error: {e}"),
    }

    tracing::info!("\n3. Environment variable mapping demonstration:");
    show_environment_mapping();

    tracing::info!("\n4. CLI argument demonstration:");
    show_cli_mapping();

    Ok(())
}

fn setup_environment_variables() {
    tracing::info!("Setting up environment variables with your expected pattern:");

    // For Config struct with env_prefix="MD"
    std::env::set_var("MD_MONGO_USERNAME", "production_mongo_user");
    std::env::set_var("MD_MONGO_PASSWORD", "super_secret_mongo_password");
    std::env::set_var("MD_APP_USERNAME", "app_user");
    std::env::set_var("MD_APP_PASSWORD", "app_password");

    tracing::info!("  MD_MONGO_USERNAME=production_mongo_user");
    tracing::info!("  MD_MONGO_PASSWORD=super_secret_mongo_password");
    tracing::info!("  MD_APP_USERNAME=app_user");
    tracing::info!("  MD_APP_PASSWORD=app_password");
    tracing::info!("");
}

fn print_config(config: &Config) {
    tracing::info!("📋 Complete Configuration:");
    tracing::info!("  🗄️  MongoDB:");
    tracing::info!("     Username: {}", config.mongo.username);
    tracing::info!(
        "     Password: [REDACTED - {} chars]",
        config.mongo.password.len()
    );

    tracing::info!("  📱 Application:");
    tracing::info!("     Username: {}", config.app.username);
    tracing::info!(
        "     Password: [REDACTED - {} chars]",
        config.app.password.len()
    );
    tracing::info!("     Client: {:?} (field skipped)", config.app.client);
}

fn show_environment_mapping() {
    tracing::info!("Environment variable naming patterns:");
    tracing::info!("  Config struct has env_prefix='MD'");
    tracing::info!("  └── mongo: Mongo");
    tracing::info!("      ├── username → MD_MONGO_USERNAME");
    tracing::info!("      └── password → MD_MONGO_PASSWORD");
    tracing::info!("  └── app: Application");
    tracing::info!("      ├── username → MD_APP_USERNAME");
    tracing::info!("      ├── password → MD_APP_PASSWORD");
    tracing::info!("      └── client → [skipped with #[skip_gonfig]]");
}

fn show_cli_mapping() {
    tracing::info!("CLI argument naming patterns:");
    tracing::info!("  Mongo struct has allow_cli=true");
    tracing::info!("  └── username → --mongo-username");
    tracing::info!("  └── password → --mongo-password");
    tracing::info!("  ");
    tracing::info!("  Application struct (no CLI support)");
    tracing::info!("  └── (CLI arguments not generated)");
    tracing::info!("");
    tracing::info!("Example CLI usage:");
    tracing::info!("  cargo run -- --mongo-username myuser --mongo-password mypass");
}
