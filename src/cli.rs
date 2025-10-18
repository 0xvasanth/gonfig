use crate::{
    error::Result,
    source::{ConfigSource, Source},
};
use clap::Parser;
use serde_json::Value;
use std::any::Any;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct Cli {
    parsed_values: HashMap<String, Value>,
    field_mappings: HashMap<String, String>,
}

impl Cli {
    pub fn from_args() -> Self {
        Self::from_vec(std::env::args().collect())
    }

    pub fn from_vec(args: Vec<String>) -> Self {
        let mut parsed_values = HashMap::new();

        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];

            if arg.starts_with("--") {
                let key = arg.trim_start_matches("--");

                if i + 1 < args.len() && !args[i + 1].starts_with("--") {
                    let value = &args[i + 1];
                    parsed_values.insert(key.to_string(), Self::parse_value(value));
                    i += 2;
                } else {
                    parsed_values.insert(key.to_string(), Value::Bool(true));
                    i += 1;
                }
            } else if arg.starts_with("-") && arg.len() == 2 {
                let key = arg.trim_start_matches("-");

                if i + 1 < args.len() && !args[i + 1].starts_with("-") {
                    let value = &args[i + 1];
                    parsed_values.insert(key.to_string(), Self::parse_value(value));
                    i += 2;
                } else {
                    parsed_values.insert(key.to_string(), Value::Bool(true));
                    i += 1;
                }
            } else {
                i += 1;
            }
        }

        Self {
            parsed_values,
            field_mappings: HashMap::new(),
        }
    }

    pub fn with_clap_app<T: Parser + serde::Serialize>() -> Result<Self> {
        let app = T::parse();

        let json_value = serde_json::to_value(&app).map_err(|e| {
            crate::error::Error::Serialization(format!("Failed to serialize clap args: {e}"))
        })?;

        let mut parsed_values = HashMap::new();
        if let Value::Object(map) = json_value {
            for (key, value) in map {
                parsed_values.insert(key, value);
            }
        }

        Ok(Self {
            parsed_values,
            field_mappings: HashMap::new(),
        })
    }

    pub fn with_field_mapping(
        mut self,
        field_name: impl Into<String>,
        cli_key: impl Into<String>,
    ) -> Self {
        self.field_mappings
            .insert(field_name.into(), cli_key.into());
        self
    }

    fn parse_value(value: &str) -> Value {
        if let Ok(b) = value.parse::<bool>() {
            return Value::Bool(b);
        }

        if let Ok(n) = value.parse::<i64>() {
            return Value::Number(n.into());
        }

        if let Ok(n) = value.parse::<f64>() {
            // Handle NaN and infinite values safely
            if let Some(num) = serde_json::Number::from_f64(n) {
                return Value::Number(num);
            }
        }

        if value.starts_with('[') && value.ends_with(']') {
            if let Ok(arr) = serde_json::from_str::<Vec<Value>>(value) {
                return Value::Array(arr);
            }
        }

        Value::String(value.to_string())
    }

    pub fn get_matches(&self) -> &HashMap<String, Value> {
        &self.parsed_values
    }
}

impl ConfigSource for Cli {
    fn source_type(&self) -> Source {
        Source::Cli
    }

    fn collect(&self) -> Result<Value> {
        Ok(Value::Object(
            self.parsed_values
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
        ))
    }

    fn has_value(&self, key: &str) -> bool {
        self.parsed_values.contains_key(key)
    }

    fn get_value(&self, key: &str) -> Option<Value> {
        self.parsed_values.get(key).cloned()
    }

    fn as_any(&self) -> &dyn Any {
        self
    }
}
