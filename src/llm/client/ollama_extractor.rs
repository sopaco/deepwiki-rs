//! Ollama Structured Output Wrapper
//!
//! Ollama does not support native structured output (unlike OpenAI), so this module
//! provides a wrapper to parse TOON/JSON from Ollama's text responses and validate against schemas.
//! TOON (Token-Oriented Object Notation) is used to reduce token usage in prompts.

use anyhow::{Context, Result};
use regex::Regex;
use rig::{agent::Agent, completion::Prompt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::LazyLock;
use toon_format::{decode_default as toon_decode, encode_default as toon_encode};

/// JSON code block regex pattern
static JSON_CODE_BLOCK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"```(?:json)?\s*(\{[\s\S]*?\})\s*```").unwrap());

/// TOON code block regex pattern
static TOON_CODE_BLOCK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"```toon\s*([\s\S]*?)\s*```").unwrap());

/// Empty array pattern in TOON (e.g., "functions[0]:" with no content)
static EMPTY_ARRAY_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(\w+)\[0\]:\s*$").unwrap());

/// Malformed array pattern (e.g., "items[0]{{...}}:" with no content)
static EMPTY_ARRAY_WITH_SCHEMA_PATTERN: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"(?m)^\s*(\w+)\[0\]\{\{[^}]*\}\}:\s*$").unwrap());

/// Ollama structured output extractor
pub struct OllamaExtractorWrapper<T> {
    agent: Agent<rig::providers::ollama::CompletionModel<reqwest::Client>>,
    max_retries: u32,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> OllamaExtractorWrapper<T>
where
    T: JsonSchema + Serialize + for<'de> Deserialize<'de>,
{
    /// Create a new Ollama extractor
    pub fn new(
        agent: Agent<rig::providers::ollama::CompletionModel<reqwest::Client>>,
        max_retries: u32,
    ) -> Self {
        Self {
            agent,
            max_retries,
            _phantom: std::marker::PhantomData,
        }
    }

    /// Execute structured extraction
    pub async fn extract(&self, prompt: &str) -> Result<T> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            let enhanced_prompt = self.build_prompt(prompt, last_error.as_deref());

            match self.try_extract(&enhanced_prompt, attempt as usize).await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    last_error = Some(format!("{:#}", e));
                    if attempt < self.max_retries {
                        tokio::time::sleep(tokio::time::Duration::from_millis(2000)).await;
                    }
                }
            }
        }

        Err(anyhow::anyhow!(
            "Failed after {} attempts. Last error: {}",
            self.max_retries,
            last_error.unwrap_or_else(|| "Unknown error".to_string())
        ))
    }

    /// Build enhanced prompt with schema and instructions using TOON format for token efficiency
    /// Falls back to JSON instructions on retry attempts for better compatibility
    fn build_prompt(&self, base_prompt: &str, previous_error: Option<&str>) -> String {
        let schema = schemars::schema_for!(T);
        let schema_json = serde_json::to_string_pretty(&schema).unwrap_or_else(|_| "{}".to_string());

        // On retry attempts (when there's a previous error), fall back to JSON format
        // as it's more reliable for models that struggle with TOON
        let use_json_fallback = previous_error.is_some();

        let mut prompt = if use_json_fallback {
            format!(
                r#"{}

**IMPORTANT: Return your response as valid JSON only.**

Follow this JSON schema:
```json
{}
```

Requirements:
1. Return ONLY valid JSON, no other text
2. All required fields must be present
3. Use null for optional fields with no value
4. Use empty arrays [] for arrays with no items
5. Wrap your response in ```json and ``` markers

"#,
                base_prompt, schema_json
            )
        } else {
            // Convert schema to TOON format to save tokens
            let schema_toon = toon_encode(&schema).unwrap_or_else(|_| schema_json.clone());

            format!(
                r#"{}

**Return your response in TOON format (a compact YAML-like notation).**

TOON quick reference:
- Key-value: `key: value`
- Nested objects: use 2-space indentation
- Empty arrays: omit the field or use `field: []`
- Arrays with objects: `items[count]{{field1,field2}}:` then values per line

Schema:
```toon
{}
```

Requirements:
1. Return ONLY valid TOON format, no extra text
2. All required fields must be present  
3. Use null for optional fields with no value
4. For empty arrays, omit the field entirely
5. Wrap your response in ```toon and ``` markers

"#,
                base_prompt, schema_toon
            )
        };

        if let Some(error) = previous_error {
            prompt.push_str(&format!(
                "**Previous attempt failed: {}**\nPlease return valid {} format.\n\n",
                error,
                if use_json_fallback { "JSON" } else { "TOON" }
            ));
        }

        prompt
    }

    /// Try to execute extraction once
    async fn try_extract(&self, prompt: &str, attempt: usize) -> Result<T> {
        let response = self
            .agent
            .prompt(prompt)
            .await
            .context("Failed to get response from Ollama")?;

        let parsed = self
            .parse_json_response(&response, attempt)
            .context("Failed to parse JSON from Ollama response")?;

        self.validate_json(&parsed)?;

        let result: T = serde_json::from_value(parsed.clone()).with_context(|| {
            let json_str =
                serde_json::to_string_pretty(&parsed).unwrap_or_else(|_| "invalid".to_string());
            format!(
                "Failed to deserialize JSON to target type on attempt {}. JSON structure: {}",
                attempt, json_str
            )
        })?;

        Ok(result)
    }

    /// Parse response using multiple strategies (TOON first, then JSON fallback)
    fn parse_json_response(&self, response: &str, attempt: usize) -> Result<Value> {
        // Strategy 1: Try TOON from code block (preferred)
        if let Some(toon_str) = self.extract_from_toon_code_block(response) {
            // Preprocess to fix common Ollama TOON output issues
            let preprocessed = self.preprocess_toon(&toon_str);
            if let Ok(parsed) = toon_decode::<Value>(&preprocessed) {
                return Ok(parsed);
            }
        }

        // Strategy 2: Try direct TOON parsing (if response looks like TOON)
        if response.contains(": ") && !response.trim_start().starts_with('{') {
            let preprocessed = self.preprocess_toon(response.trim());
            if let Ok(parsed) = toon_decode::<Value>(&preprocessed) {
                return Ok(parsed);
            }
        }

        // Strategy 3: Try direct JSON parsing (fallback)
        if let Ok(json) = serde_json::from_str::<Value>(response) {
            return Ok(json);
        }

        // Strategy 4: Extract from JSON markdown code blocks (fallback)
        if let Some(json_str) = self.extract_from_json_code_block(response) {
            if let Ok(parsed) = serde_json::from_str::<Value>(&json_str) {
                return Ok(parsed);
            }
        }

        // Strategy 5: Extract first JSON object (fallback)
        if let Some(json_str) = self.extract_first_json_object(response) {
            if let Ok(parsed) = serde_json::from_str::<Value>(&json_str) {
                return Ok(parsed);
            }
        }

        // Strategy 6: Clean, preprocess, and try parsing
        let cleaned = self.clean_response(response);
        let preprocessed = self.preprocess_toon(&cleaned);

        // Try TOON first on cleaned and preprocessed response
        if let Ok(parsed) = toon_decode::<Value>(&preprocessed) {
            return Ok(parsed);
        }

        // Finally try JSON
        serde_json::from_str::<Value>(&cleaned).with_context(|| {
            let preview = response.chars().take(500).collect::<String>();
            format!(
                "Failed to parse TOON/JSON from Ollama response (attempt {}). Preview (500 chars): {}",
                attempt, preview
            )
        })
    }

    /// Extract TOON from markdown code blocks
    fn extract_from_toon_code_block(&self, text: &str) -> Option<String> {
        TOON_CODE_BLOCK_REGEX
            .captures(text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract JSON from markdown code blocks (fallback)
    fn extract_from_json_code_block(&self, text: &str) -> Option<String> {
        JSON_CODE_BLOCK_REGEX
            .captures(text)
            .and_then(|cap| cap.get(1))
            .map(|m| m.as_str().to_string())
    }

    /// Extract first complete JSON object
    fn extract_first_json_object(&self, text: &str) -> Option<String> {
        let start = text.find('{')?;
        let mut depth = 0;
        let mut end = start;

        for (i, c) in text[start..].char_indices() {
            match c {
                '{' => depth += 1,
                '}' => {
                    depth -= 1;
                    if depth == 0 {
                        end = start + i + 1;
                        break;
                    }
                }
                _ => {}
            }
        }

        if depth == 0 && end > start {
            Some(text[start..end].to_string())
        } else {
            None
        }
    }

    /// Clean response text (handles both TOON and JSON markers)
    fn clean_response(&self, text: &str) -> String {
        text.trim()
            .trim_start_matches("```toon")
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string()
    }

    /// Preprocess TOON response to fix common Ollama output issues
    /// - Removes empty array declarations like "functions[0]:" or "items[0]{{field}}:"
    /// - These are invalid TOON syntax that Ollama sometimes generates
    fn preprocess_toon(&self, text: &str) -> String {
        // Remove lines with empty arrays (e.g., "functions[0]:" or "items[0]{{...}}:")
        let result = EMPTY_ARRAY_PATTERN.replace_all(text, "");
        let result = EMPTY_ARRAY_WITH_SCHEMA_PATTERN.replace_all(&result, "");
        result.to_string()
    }

    /// Validate basic JSON structure
    fn validate_json(&self, json: &Value) -> Result<()> {
        if !json.is_object() {
            anyhow::bail!("Expected JSON object, got: {}", json);
        }
        Ok(())
    }
}
