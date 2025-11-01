use gonfig::Gonfig;
use serde::{Deserialize, Serialize};

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
    println!("=== Your Configuration Management Example ===\n");

    // Set up environment variables as expected by your design
    setup_environment_variables();

    println!("1. Loading Config with hierarchical environment variables:");
    match Config::from_gonfig() {
        Ok(config) => {
            println!("✅ Successfully loaded configuration:");
            print_config(&config);
        }
        Err(e) => println!("❌ Error loading config: {e}"),
    }

    println!("\n2. Testing individual component loading:");

    // Test Mongo component with CLI support
    println!("Mongo configuration (supports CLI):");
    std::env::set_var("MD_MONGO_USERNAME", "mongo_user");
    std::env::set_var("MD_MONGO_PASSWORD", "mongo_pass");

    match Mongo::from_gonfig() {
        Ok(mongo) => {
            println!("  Username: {}", mongo.username);
            println!("  Password: [REDACTED]");
        }
        Err(e) => println!("  Error: {e}"),
    }

    // Test Application component
    println!("\nApplication configuration:");
    match Application::from_gonfig() {
        Ok(app) => {
            println!("  Username: {}", app.username);
            println!("  Password: [REDACTED]");
            println!("  Client: {:?} (skipped in gonfig)", app.client);
        }
        Err(e) => println!("  Error: {e}"),
    }

    println!("\n3. Environment variable mapping demonstration:");
    show_environment_mapping();

    println!("\n4. CLI argument demonstration:");
    show_cli_mapping();

    Ok(())
}

fn setup_environment_variables() {
    println!("Setting up environment variables with your expected pattern:");

    // For Config struct with env_prefix="MD"
    std::env::set_var("MD_MONGO_USERNAME", "production_mongo_user");
    std::env::set_var("MD_MONGO_PASSWORD", "super_secret_mongo_password");
    std::env::set_var("MD_APP_USERNAME", "app_user");
    std::env::set_var("MD_APP_PASSWORD", "app_password");

    println!("  MD_MONGO_USERNAME=production_mongo_user");
    println!("  MD_MONGO_PASSWORD=super_secret_mongo_password");
    println!("  MD_APP_USERNAME=app_user");
    println!("  MD_APP_PASSWORD=app_password");
    println!();
}

fn print_config(config: &Config) {
    println!("📋 Complete Configuration:");
    println!("  🗄️  MongoDB:");
    println!("     Username: {}", config.mongo.username);
    println!(
        "     Password: [REDACTED - {} chars]",
        config.mongo.password.len()
    );

    println!("  📱 Application:");
    println!("     Username: {}", config.app.username);
    println!(
        "     Password: [REDACTED - {} chars]",
        config.app.password.len()
    );
    println!("     Client: {:?} (field skipped)", config.app.client);
}

fn show_environment_mapping() {
    println!("Environment variable naming patterns:");
    println!("  Config struct has env_prefix='MD'");
    println!("  └── mongo: Mongo");
    println!("      ├── username → MD_MONGO_USERNAME");
    println!("      └── password → MD_MONGO_PASSWORD");
    println!("  └── app: Application");
    println!("      ├── username → MD_APP_USERNAME");
    println!("      ├── password → MD_APP_PASSWORD");
    println!("      └── client → [skipped with #[skip_gonfig]]");
}

fn show_cli_mapping() {
    println!("CLI argument naming patterns:");
    println!("  Mongo struct has allow_cli=true");
    println!("  └── username → --mongo-username");
    println!("  └── password → --mongo-password");
    println!("  ");
    println!("  Application struct (no CLI support)");
    println!("  └── (CLI arguments not generated)");
    println!();
    println!("Example CLI usage:");
    println!("  cargo run -- --mongo-username myuser --mongo-password mypass");
}
