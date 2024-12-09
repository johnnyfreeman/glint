use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;

use crate::masking::MaskingRule;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub requests: Vec<Request>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Request {
    pub name: String,
    pub method: String,
    pub url: String,
    pub headers: Option<HashMap<String, String>>,
    pub body: Option<RequestBody>,
    pub dependencies: Option<Dependencies>,
    #[serde(default)]
    pub masking_rules: Vec<MaskingRule>,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type")]
pub enum RequestBody {
    Text(String),
    Json(Value),
    Form(HashMap<String, String>),
}

pub type Dependencies = HashMap<String, Dependency>;

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "source")]
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
    OnePassword {
        vault: String,
        item: String,
        field: String,
    },
    File {
        path: String,
    },
    Prompt {
        label: String,
    },
    Response {
        request: String,
        target: ResponseTarget,
    },
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "source")]
pub enum ResponseTarget {
    HeaderValue { key: String },
    JsonBody { pointer: String },
}

#[tracing::instrument]
pub fn load_requests_from_toml(file: &str) -> Result<Vec<Request>, Box<dyn std::error::Error>> {
    let content = std::fs::read_to_string(file)?;
    let config: Config = toml::from_str(&content)?;
    Ok(config.requests)
}
