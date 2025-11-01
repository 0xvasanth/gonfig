use gonfig::Gonfig;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing_subscriber::EnvFilter;

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
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("=== Skip Attributes Demonstration ===\n");

    // Set up environment variables
    setup_environment();

    tracing::info!("1. Loading AppConfig with skipped fields:");
    match AppConfig::from_gonfig() {
        Ok(mut config) => {
            tracing::info!("✅ Configuration loaded successfully:");
            print_app_config(&config);

            // Manually initialize skipped fields
            config.runtime_client = Some(DatabaseClient {
                connection: "manual connection".to_string(),
            });
            config.internal_state = vec!["state1".to_string(), "state2".to_string()];
            config.thread_pool = Some(Arc::new(ThreadPool { threads: 8 }));

            tracing::info!("\n2. After manual initialization of skipped fields:");
            print_app_config_with_skipped(&config);
        }
        Err(e) => tracing::error!("❌ Error: {e}"),
    }

    tracing::info!("\n3. Loading DatabaseConfig with selective skipping:");
    match DatabaseConfig::from_gonfig() {
        Ok(mut db_config) => {
            tracing::info!("✅ Database config loaded:");
            print_db_config(&db_config);

            // Manually set the skipped password field
            db_config.password = Some("super_secret_password".to_string());
            db_config.connection_pool = Some("connection_pool_instance".to_string());

            tracing::info!("\n   After setting skipped fields manually:");
            tracing::info!("   Password: [MANUALLY SET]");
            tracing::info!("   Pool: [MANUALLY INITIALIZED]");
        }
        Err(e) => tracing::error!("❌ Database config error: {e}"),
    }

    tracing::info!("\n4. Skip attribute use cases:");
    show_skip_use_cases();

    Ok(())
}

fn setup_environment() {
    tracing::info!("Setting up environment variables (skipped fields won't be read):");

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

    tracing::info!("  APP_DATABASE_URL=postgres://localhost:5432/myapp");
    tracing::info!("  APP_PORT=8080");
    tracing::info!("  APP_DEBUG_MODE=true");
    tracing::info!("  APP_LOG_LEVEL=info");
    tracing::info!("  APP_RUNTIME_CLIENT=this_will_be_ignored  # [SKIPPED]");
    tracing::info!("  DB_HOST=localhost");
    tracing::info!("  DB_PASSWORD=ignored_env_password  # [SKIPPED]");
    tracing::info!("");
}

fn print_app_config(config: &AppConfig) {
    tracing::info!("📱 AppConfig:");
    tracing::info!("   Database URL: {}", config.database_url);
    tracing::info!("   Port: {}", config.port);
    tracing::info!("   Debug Mode: {}", config.debug_mode);
    tracing::info!("   Log Level: {}", config.log_level);
    tracing::info!(
        "   Runtime Client: {:?} (skipped, set to None)",
        config.runtime_client
    );
    tracing::info!(
        "   Internal State: {:?} (skipped, empty)",
        config.internal_state
    );
    tracing::info!("   Thread Pool: None (skipped)");
}

fn print_app_config_with_skipped(config: &AppConfig) {
    tracing::info!("📱 AppConfig (with manual fields):");
    tracing::info!("   Database URL: {}", config.database_url);
    tracing::info!("   Port: {}", config.port);
    tracing::info!("   Debug Mode: {}", config.debug_mode);
    tracing::info!("   Log Level: {}", config.log_level);
    tracing::info!("   Runtime Client: [MANUALLY INITIALIZED]");
    tracing::info!("   Internal State: {:?}", config.internal_state);
    tracing::info!(
        "   Thread Pool: [MANUALLY INITIALIZED - {} threads]",
        config.thread_pool.as_ref().map(|p| p.threads).unwrap_or(0)
    );
}

fn print_db_config(config: &DatabaseConfig) {
    tracing::info!("🗄️  DatabaseConfig:");
    tracing::info!("   Host: {}", config.host);
    tracing::info!("   Port: {}", config.port);
    tracing::info!("   Username: {}", config.username);
    tracing::info!("   Max Connections: {}", config.max_connections);
    tracing::info!("   Password: {:?} (skipped, None)", config.password);
    tracing::info!(
        "   Connection Pool: {:?} (skipped, None)",
        config.connection_pool
    );
}

fn show_skip_use_cases() {
    tracing::info!("Common use cases for skip attributes:");
    tracing::info!("");
    tracing::info!("1. Non-serializable types:");
    tracing::info!("   #[skip]");
    tracing::info!("   database_client: Option<DatabaseClient>,  // Custom client instance");
    tracing::info!("");
    tracing::info!("2. Runtime state:");
    tracing::info!("   #[skip_gonfig]");
    tracing::info!("   cache: HashMap<String, Value>,  // Runtime cache");
    tracing::info!("");
    tracing::info!("3. Sensitive data (manual initialization):");
    tracing::info!("   #[skip]");
    tracing::info!("   api_key: Option<String>,  // Set from secure vault");
    tracing::info!("");
    tracing::info!("4. Complex computed fields:");
    tracing::info!("   #[skip]");
    tracing::info!("   thread_pool: Option<ThreadPool>,  // Initialized based on config");
    tracing::info!("");
    tracing::info!("5. Implementation details:");
    tracing::info!("   #[skip_gonfig]");
    tracing::info!("   _internal_buffer: Vec<u8>,  // Internal implementation detail");
}
