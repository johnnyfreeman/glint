use super::Resolver;
use crate::{request::ResponseTarget, response::Response};
use std::collections::HashMap;
use thiserror::Error;
use tracing::{debug, error, warn};

#[derive(Error, Debug)]
pub enum ResponseResolverError {
    #[error("Request `{request:?}` was not found in history")]
    RequestNotFound { request: String },
    #[error("Path `{path:?}` not found in request {request:?}")]
    InvalidPath { path: String, request: String },
    #[error("Header `{key:?}` not found in request {request:?}")]
    HeaderNotFound { key: String, request: String },
    #[error("Invalid format for header `{key:?}` in request {request:?}")]
    InvalidHeaderFormat { key: String, request: String },
}

#[derive(Debug)]
pub struct ResponseResolver {
    history: HashMap<String, Response>,
}

impl ResponseResolver {
    pub fn new() -> Self {
        Self {
            history: HashMap::new(),
        }
    }

    #[tracing::instrument]
    pub fn save_to_history(&mut self, response: Response) -> Option<Response> {
        debug!("Saving response: {:?}", response);
        self.history.insert(response.request.name.clone(), response)
    }

    fn resolve_header(
        &self,
        request: &String,
        key: &String,
    ) -> Result<String, ResponseResolverError> {
        debug!("Resolving header '{}' for request '{}'", key, request);
        if let Some(response) = self.history.get(request) {
            debug!("Found response for request '{}': {:?}", request, response);
            if let Some(value) = response.headers.get(key) {
                debug!("Found header '{}' in request '{}'", key, request);
                if let Ok(value_str) = value.to_str() {
                    return Ok(value_str.to_string());
                } else {
                    return Err(ResponseResolverError::InvalidHeaderFormat {
                        key: key.clone(),
                        request: request.clone(),
                    });
                }
            } else {
                error!("Header '{}' not found in request '{}'", key, request);
                return Err(ResponseResolverError::HeaderNotFound {
                    key: key.clone(),
                    request: request.clone(),
                });
            }
        } else {
            warn!("Request '{}' not found in history", request);
            return Err(ResponseResolverError::RequestNotFound {
                request: request.clone(),
            });
        }
    }

    fn resolve_body(
        &self,
        request: &String,
        path: &String,
    ) -> Result<String, ResponseResolverError> {
        debug!("Resolving JSON path '{}' for request '{}'", path, request);
        if let Some(response) = self.history.get(request) {
            if let Ok(json) = &response.json() {
                debug!("Found JSON body for request '{}': {:?}", request, json);
                if let Some(extracted) = json.pointer(path) {
                    if extracted.is_null() {
                        return Ok("".to_string());
                    } else if let Some(value_str) = extracted.as_str() {
                        return Ok(value_str.to_string());
                    } else {
                        return Ok(extracted.to_string());
                    }
                } else {
                    error!("Path '{}' not found in {:?}", path, response);
                    return Err(ResponseResolverError::InvalidPath {
                        path: path.clone(),
                        request: request.clone(),
                    });
                }
            } else {
                return Err(ResponseResolverError::InvalidPath {
                    path: path.clone(),
                    request: request.clone(),
                });
            }
        } else {
            return Err(ResponseResolverError::RequestNotFound {
                request: request.clone(),
            });
        }
    }
}

impl Resolver for ResponseResolver {
    type Arguments = (String, ResponseTarget);
    type Error = ResponseResolverError;

    fn resolve(
        &mut self,
        (request, resolution_type): (String, ResponseTarget),
    ) -> Result<String, ResponseResolverError> {
        match resolution_type {
            ResponseTarget::HeaderValue { key } => self.resolve_header(&request, &key),
            ResponseTarget::JsonBody { pointer } => self.resolve_body(&request, &pointer),
        }
    }
}
