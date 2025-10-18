use gonfig::{ConfigSource, Environment};
use std::env;

#[test]
fn debug_environment_collection() {
    // Set test environment variables
    env::set_var("DATABASE_URL", "postgres://localhost");
    env::set_var("PORT", "5432");

    let env = Environment::new();
    let result = env.collect().unwrap();

    println!("Collected environment: {result:#?}");

    // Clean up
    env::remove_var("DATABASE_URL");
    env::remove_var("PORT");
}
