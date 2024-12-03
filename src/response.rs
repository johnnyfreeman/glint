use crate::request::Request;
use reqwest::{header::HeaderMap, StatusCode};
use serde_json::Value;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ResponseError {
    #[error("Failed to parse JSON: {0}")]
    JsonParseError(#[from] serde_json::Error),
}

#[derive(Clone, Debug)]
pub struct Response {
    pub request: Request,
    pub headers: HeaderMap,
    pub status: StatusCode,
    pub text: String,
}

impl Response {
    pub fn json(&self) -> Result<Value, ResponseError> {
        serde_json::from_str::<Value>(&self.text).map_err(ResponseError::from)
    }
}
