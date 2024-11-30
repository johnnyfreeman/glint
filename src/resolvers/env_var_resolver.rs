use super::{
    prompt_resolver::{PromptResolver, PromptResolverError},
    Resolver,
};
use std::collections::HashMap;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum EnvVarResolverError {
    #[error("Environment variable `{name}` not found")]
    EnvVarNotFound { name: String },
    #[error("Prompt error")]
    PromptError(#[from] PromptResolverError),
}

#[derive(Debug)]
pub struct EnvVarResolver {
    cache: HashMap<String, String>,
    prompt_resolver: PromptResolver,
}

impl EnvVarResolver {
    /// Create a new `EnvVarResolver`
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            prompt_resolver: PromptResolver::new(),
        }
    }

    /// Save resolved environment variable to cache
    pub fn save_to_cache(&mut self, key: String, value: String) -> Option<String> {
        info!("Caching environment variable");
        self.cache.insert(key, value)
    }
}

impl Resolver for EnvVarResolver {
    type Arguments = (String, Option<String>);
    type Error = EnvVarResolverError;

    /// Resolve an environment variable, checking the cache first
    #[tracing::instrument]
    fn resolve(
        &mut self,
        (name, prompt): (String, Option<String>),
    ) -> Result<String, EnvVarResolverError> {
        // Check the cache first
        if let Some(cached_value) = self.cache.get(&name) {
            info!("Cache hit for environment variable");
            return Ok(cached_value.clone());
        }

        // Fetch from environment if not in cache
        match std::env::var(&name) {
            Ok(value) => {
                self.save_to_cache(name, value.clone());
                info!("Resolved environment variable");
                Ok(value)
            }
            Err(_) => {
                if let Ok(value) = self
                    .prompt_resolver
                    .resolve(name.clone())
                    .map_err(|_| EnvVarResolverError::EnvVarNotFound { name: name.clone() })
                {
                    self.save_to_cache(name, value.clone());
                    return Ok(value);
                }

                info!("Environment variable not found: {}", name);
                Err(EnvVarResolverError::EnvVarNotFound { name })
            }
        }
    }
}
