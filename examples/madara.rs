use gonfig::{ConfigBuilder, Gonfig, MergeStrategy};
use serde::{Deserialize, Serialize};

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
}

#[derive(Debug, Serialize, Deserialize, Gonfig)]
#[Gonfig]
struct ServerConfig {
    host: String,
    port: u16,

    #[gonfig(env_name = "WORKERS")]
    worker_threads: Option<usize>,
}

fn main() -> gonfig::Result<()> {
    std::env::set_var("MDR_MONGO_URI", "mongodb://localhost:27017");
    std::env::set_var("MDR_MONGO_DATABASE", "madara_db");
    std::env::set_var("MDR_SERVER_HOST", "0.0.0.0");
    std::env::set_var("MDR_SERVER_PORT", "8080");
    std::env::set_var("MDR_SERVER_WORKERS", "4");

    let config = Madara::from_gonfig()?;
    println!("Loaded config from environment: {config:#?}");

    let builder = ConfigBuilder::new()
        .with_merge_strategy(MergeStrategy::Deep)
        .with_env("MDR")
        .with_cli()
        .validate_with(|value| {
            if let Some(port) = value.get("server").and_then(|s| s.get("port")) {
                if let Some(port_num) = port.as_u64() {
                    if port_num > 65535 {
                        return Err(gonfig::Error::Validation("Port must be <= 65535".into()));
                    }
                }
            }
            Ok(())
        });

    match builder.build::<Madara>() {
        Ok(config) => {
            println!("\nValidated config: {config:#?}");
            println!("\nMongo URI: {}", config.mongo.uri);
            println!("Server: {}:{}", config.server.host, config.server.port);
            if let Some(workers) = config.server.worker_threads {
                println!("Workers: {workers}");
            }
        }
        Err(e) => eprintln!("Config error: {e}"),
    }

    Ok(())
}
