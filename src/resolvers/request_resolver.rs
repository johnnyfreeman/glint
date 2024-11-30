use super::Resolver;
use serde_json::Value;
use std::collections::HashMap;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum RequestResolverError {
    #[error("Request `{request:?}` was not found in history")]
    RequestNotFound { request: String },
    #[error("Path `{path:?}` not found in request {request:?}")]
    InvalidPath { path: String, request: String },
}

#[derive(Debug)]
pub struct RequestResolver {
    history: HashMap<String, Value>,
}

impl RequestResolver {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    #[tracing::instrument]
    pub fn save_to_history(&mut self, request: String, response: Value) -> Option<Value> {
        info!("Saving response for {} to history", request.clone());
        self.history.insert(request, response)
    }
}

impl Resolver for RequestResolver {
    type Arguments = (String, String);
    type Error = RequestResolverError;

    fn resolve(
        &mut self,
        (request, path): (String, String),
    ) -> Result<String, RequestResolverError> {
        info!("Resolving {} from request {}", path, request);
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
            Err(RequestResolverError::RequestNotFound { request })
        }
    }
}
