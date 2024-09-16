use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub requests: Vec<Request>,
}

#[derive(Debug, Deserialize)]
pub struct Request {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<String>,
    pub dependencies: Option<HashMap<String, Dependency>>,
}

#[derive(Debug, Deserialize)]
#[serde(tag = "source", rename_all = "lowercase")]
pub enum Dependency {
    EnvVar {
        name: String,
        prompt: Option<String>,
    },
    EnvFile {
        env_file: String,
        key: String,
        prompt: Option<String>,
    },
    File {
        path: String,
    },
    Prompt {
        label: String,
    },
    Request {
        request: String,
        path: String,
    },
}

pub fn load_requests_from_toml(file: &str) -> Result<Vec<Request>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config.requests)
}
