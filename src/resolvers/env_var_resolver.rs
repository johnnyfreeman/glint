use super::Resolver;
use std::collections::HashMap;
use thiserror::Error;
use tracing::info;

#[derive(Error, Debug)]
pub enum EnvVarResolverError {
    #[error("Environment variable `{name}` not found")]
    EnvVarNotFound { name: String },
}

#[derive(Debug)]
pub struct EnvVarResolver {
    cache: HashMap<String, String>,
}

impl EnvVarResolver {
    /// Create a new `EnvVarResolver`
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
        }
    }

    /// Save resolved environment variable to cache
    #[tracing::instrument]
    pub fn save_to_cache(&mut self, key: String, value: String) -> Option<String> {
        info!("Caching environment variable: {} = {}", key, value);
        self.cache.insert(key, value)
    }
}

impl Resolver for EnvVarResolver {
    type Arguments = String;
    type Error = EnvVarResolverError;

    /// Resolve an environment variable, checking the cache first
    #[tracing::instrument]
    fn resolve(&mut self, name: String) -> Result<String, EnvVarResolverError> {
        // Check the cache first
        if let Some(cached_value) = self.cache.get(&name) {
            info!("Cache hit for environment variable: {}", name);
            return Ok(cached_value.clone());
        }

        // Fetch from environment if not in cache
        match std::env::var(&name) {
            Ok(value) => {
                self.save_to_cache(name.clone(), value.clone());
                info!("Resolved environment variable: {} = {}", name, value);
                Ok(value)
            }
            Err(_) => {
                info!("Environment variable not found: {}", name);
                Err(EnvVarResolverError::EnvVarNotFound { name })
            }
        }
    }
}
