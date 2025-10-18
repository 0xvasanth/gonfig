use gonfig::Gonfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Example demonstrating various skip attribute usages
#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig(env_prefix = "APP", allow_cli)]
struct AppConfig {
    // Regular configuration fields - will be loaded from env/CLI
    /// Environment variable: APP_DATABASE_URL
    database_url: String,

    /// Environment variable: APP_PORT
    port: u16,

    /// Environment variable: APP_DEBUG_MODE
    debug_mode: bool,

    // Skip this field completely from all configuration sources
    #[skip]
    #[serde(skip)]
    runtime_client: Option<DatabaseClient>,

    // Alternative skip syntax
    #[skip_gonfig]
    #[serde(skip)]
    internal_state: Vec<String>,

    // This field will be included in configuration
    /// Environment variable: APP_LOG_LEVEL
    log_level: String,

    // Skip with complex types
    #[skip]
    #[serde(skip)]
    thread_pool: Option<Arc<ThreadPool>>,
}

/// Example struct that would not be serializable
#[derive(Debug)]
struct DatabaseClient {
    #[allow(dead_code)]
    connection: String,
}

/// Example struct that would not be serializable
#[derive(Debug)]
struct ThreadPool {
    threads: usize,
}

#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig(env_prefix = "DB")]
struct DatabaseConfig {
    host: String,
    port: u16,
    username: String,

    #[skip]
    #[serde(skip)]
    password: Option<String>, // Skip password from config, set manually

    /// Environment variable: DB_MAX_CONNECTIONS
    max_connections: u32,

    #[skip_gonfig]
    #[serde(skip)]
    connection_pool: Option<String>, // Skip connection pool instance
}

fn main() -> gonfig::Result<()> {
    println!("=== Skip Attributes Demonstration ===\n");

    // Set up environment variables
    setup_environment();

    println!("1. Loading AppConfig with skipped fields:");
    match AppConfig::from_gonfig() {
        Ok(mut config) => {
            println!("✅ Configuration loaded successfully:");
            print_app_config(&config);

            // Manually initialize skipped fields
            config.runtime_client = Some(DatabaseClient {
                connection: "manual connection".to_string(),
            });
            config.internal_state = vec!["state1".to_string(), "state2".to_string()];
            config.thread_pool = Some(Arc::new(ThreadPool { threads: 8 }));

            println!("\n2. After manual initialization of skipped fields:");
            print_app_config_with_skipped(&config);
        }
        Err(e) => println!("❌ Error: {e}"),
    }

    println!("\n3. Loading DatabaseConfig with selective skipping:");
    match DatabaseConfig::from_gonfig() {
        Ok(mut db_config) => {
            println!("✅ Database config loaded:");
            print_db_config(&db_config);

            // Manually set the skipped password field
            db_config.password = Some("super_secret_password".to_string());
            db_config.connection_pool = Some("connection_pool_instance".to_string());

            println!("\n   After setting skipped fields manually:");
            println!("   Password: [MANUALLY SET]");
            println!("   Pool: [MANUALLY INITIALIZED]");
        }
        Err(e) => println!("❌ Database config error: {e}"),
    }

    println!("\n4. Skip attribute use cases:");
    show_skip_use_cases();

    Ok(())
}

fn setup_environment() {
    println!("Setting up environment variables (skipped fields won't be read):");

    // AppConfig environment variables
    std::env::set_var("APP_DATABASE_URL", "postgres://localhost:5432/myapp");
    std::env::set_var("APP_PORT", "8080");
    std::env::set_var("APP_DEBUG_MODE", "true");
    std::env::set_var("APP_LOG_LEVEL", "info");

    // These won't be read due to #[skip] attributes
    std::env::set_var("APP_RUNTIME_CLIENT", "this_will_be_ignored");
    std::env::set_var("APP_INTERNAL_STATE", "this_will_also_be_ignored");

    // DatabaseConfig environment variables
    std::env::set_var("DB_HOST", "localhost");
    std::env::set_var("DB_PORT", "5432");
    std::env::set_var("DB_USERNAME", "dbuser");
    std::env::set_var("DB_MAX_CONNECTIONS", "20");

    // These won't be read due to #[skip] attributes
    std::env::set_var("DB_PASSWORD", "ignored_env_password");
    std::env::set_var("DB_CONNECTION_POOL", "ignored_pool");

    println!("  APP_DATABASE_URL=postgres://localhost:5432/myapp");
    println!("  APP_PORT=8080");
    println!("  APP_DEBUG_MODE=true");
    println!("  APP_LOG_LEVEL=info");
    println!("  APP_RUNTIME_CLIENT=this_will_be_ignored  # [SKIPPED]");
    println!("  DB_HOST=localhost");
    println!("  DB_PASSWORD=ignored_env_password  # [SKIPPED]");
    println!();
}

fn print_app_config(config: &AppConfig) {
    println!("📱 AppConfig:");
    println!("   Database URL: {}", config.database_url);
    println!("   Port: {}", config.port);
    println!("   Debug Mode: {}", config.debug_mode);
    println!("   Log Level: {}", config.log_level);
    println!(
        "   Runtime Client: {:?} (skipped, set to None)",
        config.runtime_client
    );
    println!(
        "   Internal State: {:?} (skipped, empty)",
        config.internal_state
    );
    println!("   Thread Pool: None (skipped)");
}

fn print_app_config_with_skipped(config: &AppConfig) {
    println!("📱 AppConfig (with manual fields):");
    println!("   Database URL: {}", config.database_url);
    println!("   Port: {}", config.port);
    println!("   Debug Mode: {}", config.debug_mode);
    println!("   Log Level: {}", config.log_level);
    println!("   Runtime Client: [MANUALLY INITIALIZED]");
    println!("   Internal State: {:?}", config.internal_state);
    println!(
        "   Thread Pool: [MANUALLY INITIALIZED - {} threads]",
        config.thread_pool.as_ref().map(|p| p.threads).unwrap_or(0)
    );
}

fn print_db_config(config: &DatabaseConfig) {
    println!("🗄️  DatabaseConfig:");
    println!("   Host: {}", config.host);
    println!("   Port: {}", config.port);
    println!("   Username: {}", config.username);
    println!("   Max Connections: {}", config.max_connections);
    println!("   Password: {:?} (skipped, None)", config.password);
    println!(
        "   Connection Pool: {:?} (skipped, None)",
        config.connection_pool
    );
}

fn show_skip_use_cases() {
    println!("Common use cases for skip attributes:");
    println!();
    println!("1. Non-serializable types:");
    println!("   #[skip]");
    println!("   database_client: Option<DatabaseClient>,  // Custom client instance");
    println!();
    println!("2. Runtime state:");
    println!("   #[skip_gonfig]");
    println!("   cache: HashMap<String, Value>,  // Runtime cache");
    println!();
    println!("3. Sensitive data (manual initialization):");
    println!("   #[skip]");
    println!("   api_key: Option<String>,  // Set from secure vault");
    println!();
    println!("4. Complex computed fields:");
    println!("   #[skip]");
    println!("   thread_pool: Option<ThreadPool>,  // Initialized based on config");
    println!();
    println!("5. Implementation details:");
    println!("   #[skip_gonfig]");
    println!("   _internal_buffer: Vec<u8>,  // Internal implementation detail");
}
