use std::process::Command;
use thiserror::Error;
use tracing::info;

use super::Resolver;

/// Errors that may occur when resolving 1Password variables
#[derive(Error, Debug)]
pub enum OnePasswordResolverError {
    #[error("1Password CLI not found or not executable")]
    CliNotFound,

    #[error("Failed to fetch 1Password variable `{reference}`: {message}")]
    FetchError { reference: String, message: String },

    #[error("Failed to parse 1Password CLI output")]
    ParseError,

    #[error("Vault `{vault}` does not exist or is inaccessible")]
    VaultNotFound { vault: String },

    #[error("Item `{item}` not found in vault `{vault}`")]
    ItemNotFound { vault: String, item: String },

    #[error("Field `{field}` not found in item `{item}` of vault `{vault}`")]
    FieldNotFound { vault: String, item: String, field: String },
}

#[derive(Debug)]
pub struct OnePasswordResolver;

impl OnePasswordResolver {
    pub fn new() -> Self {
        Self
    }
}

impl Resolver for OnePasswordResolver {
    type Arguments = (String, String, String); // (vault, item, field)
    type Error = OnePasswordResolverError;

    /// Resolve a variable from 1Password using the CLI
    #[tracing::instrument]
    fn resolve(
        &mut self,
        (vault, item, field): Self::Arguments,
    ) -> Result<String, OnePasswordResolverError> {
        let reference = format!("op://{}/{}/{}", vault, item, field);

        // Execute the `op` CLI command
        let output = Command::new("op")
            .arg("read")
            .arg(&reference)
            .output();

        match output {
            Ok(output) if output.status.success() => {
                // Parse the output
                let value = String::from_utf8(output.stdout)
                    .map_err(|_| OnePasswordResolverError::ParseError)?
                    .trim()
                    .to_string();

                info!("Successfully resolved 1Password variable: {}", reference);
                Ok(value)
            }
            Ok(output) => {
                // Handle known errors based on stderr
                let error_message = String::from_utf8_lossy(&output.stderr).to_string();

                if error_message.contains("Vault not found") {
                    return Err(OnePasswordResolverError::VaultNotFound { vault });
                } else if error_message.contains("Item not found") {
                    return Err(OnePasswordResolverError::ItemNotFound { vault, item });
                } else if error_message.contains("Field not found") {
                    return Err(OnePasswordResolverError::FieldNotFound { vault, item, field });
                }

                Err(OnePasswordResolverError::FetchError {
                    reference,
                    message: error_message,
                })
            }
            Err(_) => Err(OnePasswordResolverError::CliNotFound),
        }
    }
}

