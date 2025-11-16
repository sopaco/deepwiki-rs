//! Ollama Structured Output Wrapper
//!
//! Ollama does not support native structured output (unlike OpenAI), so this module
//! provides a wrapper to parse JSON from Ollama's text responses and validate against schemas

use anyhow::{Context, Result};
use regex::Regex;
use rig::{agent::Agent, completion::Prompt};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::LazyLock;

/// JSON code block regex pattern
static JSON_CODE_BLOCK_REGEX: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"```(?:json)?\s*(\{[\s\S]*?\})\s*```").unwrap());

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
    pub fn new(agent: Agent<rig::providers::ollama::CompletionModel<reqwest::Client>>, max_retries: u32) -> Self {
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

    /// Build enhanced prompt with schema and instructions
    fn build_prompt(&self, base_prompt: &str, previous_error: Option<&str>) -> String {
        let schema = schemars::schema_for!(T);
        let schema_json = serde_json::to_string_pretty(&schema)
            .unwrap_or_else(|_| "{}".to_string());

        let mut prompt = format!(
            "{}\n\n**CRITICAL: YOU MUST RETURN VALID JSON**\n\nYou MUST return the result as a valid JSON object that strictly follows this schema:\n\n```json\n{}\n```\n\n",
            base_prompt, schema_json
        );

        prompt.push_str("Requirements:\n");
        prompt.push_str("1. Return pure JSON object, do not add any extra text\n");
        prompt.push_str("2. All required fields must be present\n");
        prompt.push_str("3. Field types must match schema exactly\n");
        prompt.push_str("4. Arrays and nested objects must be correctly formatted\n\n");

        if let Some(error) = previous_error {
            prompt.push_str(&format!(
                "**Previous attempt failed with error: {}**\nPlease fix these issues and regenerate.\n\n",
                error
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
            let json_str = serde_json::to_string_pretty(&parsed)
                .unwrap_or_else(|_| "invalid".to_string());
            format!(
                "Failed to deserialize JSON to target type on attempt {}. JSON structure: {}",
                attempt, json_str
            )
        })?;

        Ok(result)
    }

    /// Parse JSON response using multiple strategies
    fn parse_json_response(&self, response: &str, attempt: usize) -> Result<Value> {
        // Strategy 1: Try direct parsing
        if let Ok(json) = serde_json::from_str::<Value>(response) {
            return Ok(json);
        }

        // Strategy 2: Extract from markdown code blocks
        if let Some(json_str) = self.extract_from_code_block(response) {
            if let Ok(parsed) = serde_json::from_str::<Value>(&json_str) {
                return Ok(parsed);
            }
        }

        // Strategy 3: Extract first JSON object
        if let Some(json_str) = self.extract_first_json_object(response) {
            if let Ok(parsed) = serde_json::from_str::<Value>(&json_str) {
                return Ok(parsed);
            }
        }

        // Strategy 4: Clean and try parsing
        let cleaned = self.clean_response(response);
        serde_json::from_str::<Value>(&cleaned).with_context(|| {
            let preview = response.chars().take(200).collect::<String>();
            format!(
                "Failed to parse JSON from Ollama response (attempt {}). Response preview: {}",
                attempt, preview
            )
        })
    }

    /// Extract JSON from markdown code blocks
    fn extract_from_code_block(&self, text: &str) -> Option<String> {
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

    /// Clean response text
    fn clean_response(&self, text: &str) -> String {
        text.trim()
            .trim_start_matches("```json")
            .trim_start_matches("```")
            .trim_end_matches("```")
            .trim()
            .to_string()
    }

    /// Validate basic JSON structure
    fn validate_json(&self, json: &Value) -> Result<()> {
        if !json.is_object() {
            anyhow::bail!("Expected JSON object, got: {}", json);
        }
        Ok(())
    }
}