use crate::options::Options;
use crate::request::{Dependency, Request};
use crate::resolvers::request_resolver::RequestResolver;
use crate::resolvers::Resolver;
use bat::PrettyPrinter;
use console::style;
use dialoguer::theme::ColorfulTheme;
use dialoguer::Input;
use lazy_static::lazy_static;
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::header::{HeaderMap, HeaderName};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Mutex;

lazy_static! {
    static ref PLACEHOLDER_REGEX: Regex = Regex::new(r"\{(\w+)\}").unwrap();
}

// Cache for loaded env files
static ENV_FILES_CACHE: Lazy<Mutex<HashMap<String, HashMap<String, String>>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[derive(Debug)]
pub struct Executor {
    requests: HashMap<String, Request>,
    options: Options,
    http: Client,
    request_resolver: RequestResolver,
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
            request_resolver: RequestResolver::new(),
        }
    }

    #[tracing::instrument(skip(self))]
    pub async fn execute(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        match &self.options.request {
            Some(request) => {
                let cloned_request = self.requests.get(request).expect("Invalid Request").clone();
                self.execute_request(cloned_request).await?;
            }
            None => {
                let cloned_requests: Vec<_> = self.requests.values().cloned().collect();

                for request in cloned_requests {
                    self.execute_request(request).await?;
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    pub async fn execute_request(
        &mut self,
        request: Request,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Resolve URL
        let url =
            resolve_placeholders(&request.url, &self.request_resolver, &request.dependencies)?;
        let headers = if let Some(header_map) = &request.headers {
            let mut resolved_headers = HeaderMap::new();
            for (key, value) in header_map {
                let resolved_key =
                    resolve_placeholders(key, &self.request_resolver, &request.dependencies)?;
                let resolved_value =
                    resolve_placeholders(value, &self.request_resolver, &request.dependencies)?;
                let header_name = HeaderName::from_bytes(resolved_key.as_bytes())?;
                resolved_headers.insert(header_name, resolved_value.parse()?);
            }
            resolved_headers
        } else {
            HeaderMap::new()
        };

        // Resolve the request body, if it exists
        let body = if let Some(body_template) = &request.body {
            Some(resolve_placeholders(
                body_template,
                &self.request_resolver,
                &request.dependencies,
            )?)
        } else {
            None
        };

        // Execute the request and capture the response
        let res = self
            .http
            .request(
                reqwest::Method::from_bytes(request.method.as_bytes())?,
                &url,
            )
            .headers(headers)
            .body(body.unwrap_or_default())
            .send()
            .await?;

        // Extract the headers first, since `res.text()` will consume `res`
        let headers: HeaderMap = res.headers().clone(); // Clone the headers to avoid borrowing issues

        // Now you can safely consume the body
        let status = res.status();
        let body_text = res.text().await?;

        println!(
            "\n{} {}",
            if let true = status.is_client_error() {
                style(" ".to_string() + status.as_str() + " ")
                    .on_yellow()
                    .black()
            } else if let true = status.is_server_error() {
                style(" ".to_string() + status.as_str() + " ")
                    .on_red()
                    .black()
            } else {
                style(" ".to_string() + status.as_str() + " ")
                    .on_green()
                    .black()
            },
            style(&request.name).bold(),
            // status.canonical_reason().unwrap_or(""),
        );
        // println!("{}", style("─".repeat(50)).dim());

        if self.options.show_headers {
            // Prepare the headers in a formatted string for pretty printing
            let mut headers_formatted = String::new();
            for (key, value) in headers {
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
                .print()?;
        }

        println!();

        // Pretty print the body, if it is valid JSON
        if let Ok(pretty_json) =
            serde_json::to_string_pretty(&serde_json::from_str::<serde_json::Value>(&body_text)?)
        {
            PrettyPrinter::new()
                .input_from_bytes(pretty_json.as_bytes()) // Use the formatted pretty JSON
                .language("json") // Specify JSON language for highlighting
                .print()?;
            println!();
        } else {
            // If it's not JSON, print the raw body as plain text
            PrettyPrinter::new()
                .input_from_bytes(body_text.as_bytes()) // Use raw body text
                .language("plain") // Print as plain text
                .print()?;
            println!();
        }

        // Store the response for use in future requests, if applicable
        if let Ok(json) = serde_json::from_str::<Value>(&body_text) {
            let _ = &self
                .request_resolver
                .save_to_history(request.name.clone(), json);
        }

        Ok(())
    }
}

// TODO: replace below

#[tracing::instrument(skip(request_resolver))]
fn resolve_placeholders(
    template: &str,
    request_resolver: &RequestResolver,
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
            resolve_dependency_value(dep, placeholder, request_resolver)?
        } else {
            return Err(format!("Unable to resolve placeholder: {}", placeholder).into());
        };

        // Replace the placeholder in the resolved string
        resolved = resolved.replace(&format!("{{{}}}", placeholder), &value);
    }

    Ok(resolved)
}

#[tracing::instrument(skip(request_resolver))]
fn resolve_dependency_value(
    dep: &Dependency,
    _placeholder: &str,
    request_resolver: &RequestResolver,
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
                let value = prompt_user(prompt);
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
            if let Ok(env_value) = std::env::var(name) {
                Ok(env_value)
            } else if let Some(prompt) = prompt {
                Ok(prompt_user(prompt))
            } else {
                Err(format!("Environment variable '{}' not found", name).into())
            }
        }
        Dependency::File { path } => {
            let file_content = std::fs::read_to_string(path)?;
            Ok(file_content.trim().to_string())
        }
        Dependency::Request { request, path } => {
            Ok(request_resolver.resolve((request.to_owned(), path.to_owned()))?)
        }
        Dependency::Prompt { label } => Ok(prompt_user(label)),
    }
}

#[tracing::instrument]
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

#[tracing::instrument(skip(data))]
fn save_env_file(
    env_file: &str,
    data: &HashMap<String, String>,
) -> Result<(), Box<dyn std::error::Error>> {
    let content = toml::to_string(data)?;
    std::fs::write(env_file, content)?;
    Ok(())
}

#[tracing::instrument]
fn prompt_user(prompt: &str) -> String {
    Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .interact_text()
        .unwrap()
}
