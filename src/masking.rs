use jsonpath_lib::replace_with;
use regex::Regex;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use thiserror::Error;
use tracing::{debug, error, info, warn};

#[derive(Debug, Clone, Deserialize)]
pub struct MaskingRule {
    pub path: String,        // JSONPath for locating the fields
    pub regex: RegexWrapper, // Custom wrapper around Regex
    pub replace: String,     // Replacement string for masking
}

/// Wrapper around `Regex` to implement `Deserialize`.
#[derive(Debug, Clone)] // Removed Deserialize here
pub struct RegexWrapper(pub Regex);

impl<'de> Deserialize<'de> for RegexWrapper {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let regex_str = String::deserialize(deserializer)?;
        debug!(%regex_str, "Deserializing regex");
        Regex::new(&regex_str)
            .map(RegexWrapper)
            .map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Error)]
pub enum MaskingError {
    #[error("Invalid JSONPath: {0}")]
    InvalidJsonPath(String),
}

pub fn mask_json(json: Value, rules: &[MaskingRule]) -> Result<Value, MaskingError> {
    info!("Starting JSON masking process with {} rules", rules.len());
    let mut new_json = json;

    for (index, rule) in rules.iter().enumerate() {
        info!(rule_index = index, path = %rule.path, "Processing masking rule");
        debug!(regex = %rule.regex.0, replace = %rule.replace, "Masking rule details");

        new_json = replace_with(new_json, &rule.path, &mut |v| {
            if let Some(original) = v.as_str() {
                debug!(original_value = %original, "Applying regex masking");
                let masked = rule
                    .regex
                    .0
                    .replace_all(original, &rule.replace)
                    .to_string();
                info!(masked_value = %masked, "Masked value successfully");
                Some(Value::String(masked)) // Return the masked value
            } else {
                warn!(value = ?v, "Value is not a string; skipping masking");
                None // Leave non-string values unchanged
            }
        })
        .map_err(|e| {
            error!(error = %e, path = %rule.path, "Failed to process JSONPath");
            MaskingError::InvalidJsonPath(e.to_string())
        })?;
    }

    info!("Completed JSON masking process");
    Ok(new_json)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_mask_json_success() {
        let json = json!({
            "sensitive_key": "1234-5678",
            "nested": {
                "another_key": "9876-5432"
            }
        });

        let rules = vec![
            MaskingRule {
                path: "$.sensitive_key".to_string(),
                regex: RegexWrapper(Regex::new(r"\d{4}-\d{4}").unwrap()),
                replace: "****-****".to_string(),
            },
            MaskingRule {
                path: "$.nested.another_key".to_string(),
                regex: RegexWrapper(Regex::new(r"\d{4}-\d{4}").unwrap()),
                replace: "****-****".to_string(),
            },
        ];

        let masked_json = mask_json(json, &rules).unwrap();

        let expected = json!({
            "sensitive_key": "****-****",
            "nested": {
                "another_key": "****-****"
            }
        });

        assert_eq!(masked_json, expected);
    }

    #[test]
    fn test_regex_wrapper_deserialization() {
        let json = r#"
            {
                "path": "$.sensitive_key",
                "regex": "\\d{4}-\\d{4}",
                "replace": "****-****"
            }
        "#;

        let rule: MaskingRule = serde_json::from_str(json).unwrap();
        assert_eq!(rule.path, "$.sensitive_key");
        assert_eq!(rule.replace, "****-****");
        assert!(rule.regex.0.is_match("1234-5678"));
    }
}
