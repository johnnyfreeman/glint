use super::Resolver;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum RequestResolverError {
    #[error("Request `{request:?}` is not a valid request")]
    InvalidRequest { request: String },
    #[error("Path `{path:?}` not found in request {request:?}")]
    InvalidPath { path: String, request: String },
}

pub struct RequestResolver {
    history: HashMap<String, Value>,
}

impl RequestResolver {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    pub fn save_to_history(&mut self, request: String, response: Value) -> Option<Value> {
        self.history.insert(request, response)
    }
}

impl Resolver for RequestResolver {
    type Arguments = (String, String);
    type Error = RequestResolverError;

    fn resolve(&self, (request, path): (String, String)) -> Result<String, RequestResolverError> {
        if let Some(json) = self.history.get(&request) {
            if let Some(extracted) = json.pointer(&path) {
                if extracted.is_null() {
                    Ok("".to_string())
                } else if let Some(value_str) = extracted.as_str() {
                    Ok(value_str.to_string())
                } else {
                    Ok(extracted.to_string())
                }
            } else {
                Err(RequestResolverError::InvalidPath { path, request })
            }
        } else {
            Err(RequestResolverError::InvalidRequest { request })
        }
    }
}
