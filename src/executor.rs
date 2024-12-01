use crate::options::Options;
use crate::request::{Dependency, Request};
use crate::resolvers::env_var_resolver::EnvVarResolver;
use crate::resolvers::one_password_resolver::OnePasswordResolver;
use crate::resolvers::prompt_resolver::PromptResolver;
use crate::resolvers::request_resolver::RequestResolver;
use crate::resolvers::Resolver;
use crate::response::Response;
use bat::PrettyPrinter;
use console::style;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;
use thiserror::Error;

lazy_static! {
    static ref PLACEHOLDER_REGEX: Regex = Regex::new(r"\{(\w+)\}").unwrap();
}

#[derive(Error, Debug)]
pub enum ExecutionError {
    #[error("Request `{request:?}` was not found in history")]
    RequestNotFound { request: String },
    #[error("Template `{template:?}` could not be resolved")]
    ResolutionFailed { template: String },
    #[error("Unknown execution error")]
    Unknown,
}

// Cache for loaded env files
static ENV_FILES_CACHE: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct Executor {
    requests: HashMap<String, Request>,
    options: Options,
    http: Client,
    env_var_resolver: EnvVarResolver,
    prompt_resolver: PromptResolver,
    request_resolver: RequestResolver,
    one_password_resolver: OnePasswordResolver,
}

impl Executor {
    pub fn new(requests: Vec<Request>, options: Options) -> Self {
        Self {
            requests: requests
                .into_iter()
                .map(|request| (request.name.clone(), request))
                .collect(),
            options,
            http: Client::new(),
            env_var_resolver: EnvVarResolver::new(),
            prompt_resolver: PromptResolver::new(),
            request_resolver: RequestResolver::new(),
            one_password_resolver: OnePasswordResolver::new(),
        }
    }

    pub async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.options.request {
            Some(request_name) => {
                let request = self
                    .requests
                    .get(request_name)
                    .ok_or(ExecutionError::RequestNotFound {
                        request: request_name.to_owned(),
                    })?
                    .clone();

                let response = self.execute_request(request.clone()).await?;

                self.render_output(response).await?;
            }
            None => {
                let cloned_requests: Vec<_> = self.requests.values().cloned().collect();

                for request in cloned_requests {
                    let response = self.execute_request(request.clone()).await?;

                    self.render_output(response).await?;
                }
            }
        }
        Ok(())
    }

    pub async fn execute_request(&mut self, request: Request) -> Result<Response, ExecutionError> {
        // Resolve URL
        let url = self
            .resolve_placeholders(&request.url, &request.dependencies)
            .await
            .map_err(|_| ExecutionError::ResolutionFailed {
                template: request.url.clone(),
            })?;
        let headers = if let Some(header_map) = &request.headers {
            let mut resolved_headers = HeaderMap::new();
            for (key, value) in header_map {
                let resolved_key = self
                    .resolve_placeholders(key, &request.dependencies)
                    .await
                    .map_err(|_| ExecutionError::ResolutionFailed {
                        template: key.to_owned(),
                    })?;
                let resolved_value = self
                    .resolve_placeholders(value, &request.dependencies)
                    .await
                    .map_err(|_| ExecutionError::ResolutionFailed {
                        template: value.to_owned(),
                    })?;
                let header_name =
                    HeaderName::from_bytes(resolved_key.as_bytes()).map_err(|_| {
                        ExecutionError::ResolutionFailed {
                            template: value.to_owned(),
                        }
                    })?;
                resolved_headers.insert(
                    header_name,
                    resolved_value
                        .parse()
                        .map_err(|_| ExecutionError::Unknown)?,
                );
            }
            resolved_headers
        } else {
            HeaderMap::new()
        };

        // Resolve the request body, if it exists
        let body = if let Some(body_template) = &request.body {
            Some(
                self.resolve_placeholders(body_template, &request.dependencies)
                    .await
                    .map_err(|_| ExecutionError::Unknown)?,
            )
        } else {
            None
        };

        // Execute the request and capture the response
        let response = self
            .http
            .request(
                reqwest::Method::from_bytes(request.method.as_bytes())
                    .map_err(|_| ExecutionError::Unknown)?,
                &url,
            )
            .headers(headers)
            .body(body.unwrap_or_default())
            .send()
            .await
            .map_err(|_| ExecutionError::Unknown)?;

        let response = Response {
            request: request.clone(),
            headers: response.headers().clone(),
            status: response.status(),
            text: response.text().await.map_err(|_| ExecutionError::Unknown)?,
        };

        // Store the response for use in future requests, if applicable
        if let Ok(json) = serde_json::from_str::<Value>(&response.text) {
            let _ = &self
                .request_resolver
                .save_to_history(request.name.clone(), json);
        }

        Ok(response)
    }

    async fn render_output(&mut self, response: Response) -> Result<(), ExecutionError> {
        println!(
            "\n{} {}",
            if let true = response.status.is_client_error() {
                style(" ".to_string() + response.status.as_str() + " ")
                    .on_yellow()
                    .black()
            } else if let true = response.status.is_server_error() {
                style(" ".to_string() + response.status.as_str() + " ")
                    .on_red()
                    .black()
            } else {
                style(" ".to_string() + response.status.as_str() + " ")
                    .on_green()
                    .black()
            },
            style(&response.request.name).bold(),
            // status.canonical_reason().unwrap_or(""),
        );
        // println!("{}", style("─".repeat(50)).dim());

        if self.options.show_headers {
            // Prepare the headers in a formatted string for pretty printing
            let mut headers_formatted = String::new();
            for (key, value) in response.headers {
                let key_str = key.as_ref().map(|k| k.as_str()).unwrap_or(""); // Safely unwrap the header key
                let value_str = value.to_str().unwrap_or(""); // Convert HeaderValue to str, fallback to empty string if invalid
                headers_formatted.push_str(&format!("{}: {}\n", key_str, value_str));
                // Format as key: value without quotes
            }

            println!();

            // Pretty print the headers
            PrettyPrinter::new()
                .input_from_bytes(headers_formatted.as_bytes()) // Use the formatted headers
                .language("toml") // Print as TOML (or use "yaml" for a similar format)
                .print()
                .map_err(|_| ExecutionError::Unknown)?;
        }

        println!();

        // Pretty print the body, if it is valid JSON
        if let Ok(pretty_json) = serde_json::to_string_pretty(
            &serde_json::from_str::<serde_json::Value>(&response.text)
                .map_err(|_| ExecutionError::Unknown)?,
        ) {
            PrettyPrinter::new()
                .input_from_bytes(pretty_json.as_bytes()) // Use the formatted pretty JSON
                .language("json") // Specify JSON language for highlighting
                .print()
                .map_err(|_| ExecutionError::Unknown)?;
            println!();
        } else {
            // If it's not JSON, print the raw body as plain text
            PrettyPrinter::new()
                .input_from_bytes(response.text.as_bytes()) // Use raw body text
                .language("plain") // Print as plain text
                .print()
                .map_err(|_| ExecutionError::Unknown)?;
            println!();
        }

        Ok(())
    }

    async fn resolve_placeholders(
        &mut self,
        template: &str,
        request_dependencies: &Option<HashMap<String, Dependency>>,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut resolved = template.to_string();

        // Find all placeholders in the template
        for caps in PLACEHOLDER_REGEX.captures_iter(template) {
            let placeholder = &caps[1]; // The name inside {}

            // Try to resolve the placeholder
            let value = if let Some(dep) = request_dependencies
                .as_ref()
                .and_then(|deps| deps.get(placeholder))
            {
                self.resolve_dependency_value(dep, placeholder).await?
            } else {
                return Err(format!("Unable to resolve placeholder: {}", placeholder).into());
            };

            // Replace the placeholder in the resolved string
            resolved = resolved.replace(&format!("{{{}}}", placeholder), &value);
        }

        Ok(resolved)
    }

    async fn resolve_dependency_value(
        &mut self,
        dep: &Dependency,
        _placeholder: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        match dep {
            Dependency::EnvFile {
                env_file,
                key,
                prompt,
            } => {
                // Load the TOML file
                let mut env_data = load_env_file(env_file)?;

                if let Some(value) = env_data.get(key) {
                    Ok(value.clone())
                } else if let Some(prompt) = prompt {
                    // Prompt the user
                    let value = self.prompt_resolver.resolve(prompt.clone())?;
                    // Optionally, save the value back to the env file or cache
                    env_data.insert(key.clone(), value.clone());
                    // Save back to the file
                    save_env_file(env_file, &env_data)?;
                    // Update the cache
                    let mut cache = ENV_FILES_CACHE.lock().unwrap();
                    cache.insert(env_file.clone(), env_data);
                    Ok(value)
                } else {
                    Err(format!("Key '{}' not found in file '{}'", key, env_file).into())
                }
            }
            Dependency::EnvVar { name, prompt } => {
                if let Ok(env_value) = self
                    .env_var_resolver
                    .resolve((name.to_owned(), prompt.clone()))
                {
                    Ok(env_value)
                } else {
                    Err(format!("Environment variable '{}' not found", name).into())
                }
            }
            Dependency::OnePassword { vault, item, field } => {
                if let Ok(value) =
                    self.one_password_resolver
                        .resolve((vault.clone(), item.clone(), field.clone()))
                {
                    Ok(value)
                } else {
                    Err(format!("1Password variable '{}' not found", vault).into())
                }
            }
            Dependency::File { path } => {
                let file_content = std::fs::read_to_string(path)?;
                Ok(file_content.trim().to_string())
            }
            Dependency::Request { request, path } => {
                // Check if the request is already resolved
                if let Ok(value) = self
                    .request_resolver
                    .resolve((request.clone(), path.clone()))
                {
                    return Ok(value);
                }

                // TODO: Resolve fresh request if this fails
                let cloned_request = self
                    .requests
                    .get(request)
                    .ok_or(ExecutionError::RequestNotFound {
                        request: request.to_owned(),
                    })?
                    .clone();

                Box::pin(self.execute_request(cloned_request)).await?;

                Ok(self
                    .request_resolver
                    .resolve((request.to_owned(), path.to_owned()))?)
            }
            Dependency::Prompt { label } => Ok(self.prompt_resolver.resolve(label.clone())?),
        }
    }
}

fn load_env_file(env_file: &str) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let mut cache = ENV_FILES_CACHE.lock().unwrap();

    if let Some(data) = cache.get(env_file) {
        Ok(data.clone())
    } else {
        // Read and parse the TOML file
        let content = std::fs::read_to_string(env_file)?;
        let data: HashMap<String, String> = toml::from_str(&content)?;
        cache.insert(env_file.to_string(), data.clone());
        Ok(data)
    }
}

fn save_env_file(
    env_file: &str,
    data: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = toml::to_string(data)?;
    std::fs::write(env_file, content)?;
    Ok(())
}
