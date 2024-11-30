use super::Resolver;
use dialoguer::{theme::ColorfulTheme, Input};
use std::collections::HashMap;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum PromptResolverError {
    #[error("Prompt for `{key}` failed")]
    PromptFailed { key: String },
}

#[derive(Debug)]
pub struct PromptResolver {
    cache: HashMap<String, String>,
}

impl PromptResolver {
    /// Create a new `PromptResolver`
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Save a user-provided value to the cache
    fn save_to_cache(&mut self, key: String, value: String) -> Option<String> {
        info!("Caching user input: {} = {}", key, value);
        self.cache.insert(key, value)
    }

    /// Prompt the user for input
    fn prompt_user(&self, key: &str) -> Result<String, PromptResolverError> {
        Input::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Enter value for {}", key))
            .interact_text()
            .map_err(|_| PromptResolverError::PromptFailed {
                key: key.to_string(),
            })
    }
}

impl Resolver for PromptResolver {
    type Arguments = String;
    type Error = PromptResolverError;

    /// Resolve a user input value, caching the result
    #[tracing::instrument]
    fn resolve(&mut self, key: String) -> Result<String, PromptResolverError> {
        // Check the cache first
        if let Some(cached_value) = self.cache.get(&key) {
            info!("Cache hit for user input: {}", key);
            return Ok(cached_value.clone());
        }

        // Prompt the user for input if not in cache
        let value = self.prompt_user(&key)?;
        info!("Resolved user input: {} = {}", key, value);

        // Save the value to the cache
        self.save_to_cache(key.clone(), value.clone());

        Ok(value)
    }
}
