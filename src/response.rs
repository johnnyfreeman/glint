use crate::request::Request;
use reqwest::{header::HeaderMap, StatusCode};

#[derive(Clone, Debug)]
pub struct Response {
    pub request: Request,
    pub headers: HeaderMap,
    pub status: StatusCode,
    pub text: String,
}
