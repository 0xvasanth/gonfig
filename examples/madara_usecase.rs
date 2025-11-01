use gonfig::{ConfigBuilder, Gonfig, MergeStrategy};
use serde::{Deserialize, Serialize};
use tracing_subscriber::EnvFilter;

#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig(allow_cli, env_prefix = "MDR")]
struct Madara {
    #[gonfig(env_name = "MADARA_MONGO")]
    mongo: MongoConfig,

    #[gonfig(env_name = "MADARA_SERVER")]
    server: ServerConfig,

    #[skip]
    #[serde(skip)]
    _internal: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig(env_prefix = "MONGO")]
struct MongoConfig {
    uri: String,

    #[gonfig(env_name = "MONGO_DATABASE")]
    database: String,

    connection_timeout: Option<u64>,

    max_pool_size: Option<u32>,
}

#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig(allow_cli, env_prefix = "SERVER")]
struct ServerConfig {
    host: String,
    port: u16,

    #[gonfig(env_name = "WORKERS")]
    worker_threads: Option<usize>,

    enable_cors: Option<bool>,
}

fn main() -> gonfig::Result<()> {
    // Initialize tracing subscriber
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive(tracing::Level::INFO.into()))
        .init();

    tracing::info!("=== Madara Configuration Management Demo ===\n");

    // Set up environment variables as they would be in production
    setup_environment_variables();

    tracing::info!("1. Loading with derive macro (simple approach):");
    match Madara::from_gonfig() {
        Ok(config) => {
            tracing::info!("✅ Loaded config from environment:");
            print_madara_config(&config);
        }
        Err(e) => tracing::error!("❌ Error: {e}"),
    }

    tracing::info!("\n2. Loading with custom builder (advanced approach):");
    let builder = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_env("MDR")
        .with_cli()
        .validate_with(|value| {
            // Validate port range
            if let Some(server) = value.get("server") {
                if let Some(port) = server.get("port") {
                    if let Some(port_num) = port.as_u64() {
                        if port_num > 65535 {
                            return Err(gonfig::Error::Validation("Port must be <= 65535".into()));
                        }
                    }
                }
            }

            // Validate MongoDB URI format
            if let Some(mongo) = value.get("mongo") {
                if let Some(uri) = mongo.get("uri") {
                    if let Some(uri_str) = uri.as_str() {
                        if !uri_str.starts_with("mongodb://")
                            && !uri_str.starts_with("mongodb+srv://")
                        {
                            return Err(gonfig::Error::Validation(
                                "MongoDB URI must start with mongodb:// or mongodb+srv://".into(),
                            ));
                        }
                    }
                }
            }

            Ok(())
        });

    match builder.build::<Madara>() {
        Ok(config) => {
            tracing::info!("✅ Validated config:");
            print_madara_config(&config);

            tracing::info!("\n3. Individual component access:");
            tracing::info!("MongoDB Connection: {}", config.mongo.uri);
            tracing::info!("Database: {}", config.mongo.database);
            tracing::info!(
                "Server Address: {}:{}",
                config.server.host,
                config.server.port
            );

            if let Some(workers) = config.server.worker_threads {
                tracing::info!("Worker Threads: {workers}");
            }

            if let Some(timeout) = config.mongo.connection_timeout {
                tracing::info!("Connection Timeout: {timeout}s");
            }
        }
        Err(e) => tracing::error!("❌ Validation failed: {e}"),
    }

    tracing::info!("\n4. Testing different environment variable patterns:");
    test_environment_patterns();

    Ok(())
}

fn setup_environment_variables() {
    tracing::info!("Setting up environment variables with Madara pattern:");

    // MDR prefix + component override
    std::env::set_var("MADARA_MONGO_URI", "mongodb://localhost:27017");
    std::env::set_var("MADARA_MONGO_DATABASE", "madara_production");
    std::env::set_var("MADARA_MONGO_CONNECTION_TIMEOUT", "30");
    std::env::set_var("MADARA_MONGO_MAX_POOL_SIZE", "10");

    // Server configuration
    std::env::set_var("MADARA_SERVER_HOST", "0.0.0.0");
    std::env::set_var("MADARA_SERVER_PORT", "8080");
    std::env::set_var("WORKERS", "4"); // Override name
    std::env::set_var("MADARA_SERVER_ENABLE_CORS", "true");

    tracing::info!("  MADARA_MONGO_URI=mongodb://localhost:27017");
    tracing::info!("  MADARA_MONGO_DATABASE=madara_production");
    tracing::info!("  MADARA_MONGO_CONNECTION_TIMEOUT=30");
    tracing::info!("  MADARA_SERVER_HOST=0.0.0.0");
    tracing::info!("  MADARA_SERVER_PORT=8080");
    tracing::info!("  WORKERS=4  # (field override)");
    tracing::info!("");
}

fn print_madara_config(config: &Madara) {
    tracing::info!("📋 Madara Configuration:");
    tracing::info!("  🗄️  MongoDB:");
    tracing::info!("     URI: {}", config.mongo.uri);
    tracing::info!("     Database: {}", config.mongo.database);
    if let Some(timeout) = config.mongo.connection_timeout {
        tracing::info!("     Timeout: {timeout}s");
    }
    if let Some(pool_size) = config.mongo.max_pool_size {
        tracing::info!("     Pool Size: {pool_size}");
    }

    tracing::info!("  🌐 Server:");
    tracing::info!("     Host: {}", config.server.host);
    tracing::info!("     Port: {}", config.server.port);
    if let Some(workers) = config.server.worker_threads {
        tracing::info!("     Workers: {workers}");
    }
    if let Some(cors) = config.server.enable_cors {
        tracing::info!("     CORS: {cors}");
    }
}

fn test_environment_patterns() {
    tracing::info!("Testing different prefix patterns:");

    // Test case 1: Standard hierarchy
    std::env::set_var("TEST_MADARA_MONGO_URI", "mongodb://test1:27017");
    tracing::info!("  TEST_MADARA_MONGO_URI → hierarchical structure");

    // Test case 2: Field override
    std::env::set_var("CUSTOM_MONGO_URI", "mongodb://test2:27017");
    tracing::info!("  CUSTOM_MONGO_URI → field name override");

    // Test case 3: Nested structure
    std::env::set_var("APP_DATABASE_CONFIG_HOST", "test.db.com");
    tracing::info!("  APP_DATABASE_CONFIG_HOST → nested configuration");

    tracing::info!("\nPrefix resolution examples:");
    tracing::info!("  With prefix 'MDR' and struct 'Madara':");
    tracing::info!("    field 'mongo.uri' → MDR_MADARA_MONGO_URI");
    tracing::info!("    field with env_name='CUSTOM' → CUSTOM");
    tracing::info!(
        "    nested field 'mongo.connection_timeout' → MDR_MADARA_MONGO_CONNECTION_TIMEOUT"
    );
}
