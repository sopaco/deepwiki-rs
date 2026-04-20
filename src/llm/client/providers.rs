//! LLM Provider support module

use anyhow::Result;
use rig::{
    agent::Agent,
    client::CompletionClient,
    completion::Prompt,
    extractor::Extractor,
    providers::gemini::completion::gemini_api_types::{AdditionalParameters, GenerationConfig},
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    config::{LLMConfig, LLMProvider},
    llm::tools::time::AgentToolTime,
};

use super::ollama_extractor::OllamaExtractorWrapper;
use super::openai_compatible_extractor::OpenAICompatibleExtractorWrapper;

/// Unified Provider client enum
#[derive(Clone)]
pub enum ProviderClient {
    OpenAI(rig::providers::openai::CompletionsClient),
    Moonshot(rig::providers::moonshot::Client),
    DeepSeek(rig::providers::deepseek::Client),
    Mistral(rig::providers::mistral::Client),
    OpenRouter(rig::providers::openrouter::Client),
    Anthropic(rig::providers::anthropic::Client),
    Gemini(rig::providers::gemini::Client),
    Ollama(rig::providers::ollama::Client),
}

impl ProviderClient {
    /// Create corresponding provider client based on configuration
    pub fn new(config: &LLMConfig) -> Result<Self> {
        match config.provider {
            LLMProvider::OpenAI => {
                let client = if config.api_base_url != "https://api.openai.com/v1" {
                    rig::providers::openai::Client::builder()
                        .api_key(&config.api_key)
                        .base_url(&config.api_base_url)
                        .build()?
                        .completions_api()
                } else {
                    rig::providers::openai::Client::new(&config.api_key)?
                        .completions_api()
                };
                Ok(ProviderClient::OpenAI(client))
            }
            LLMProvider::Moonshot => {
                let client = rig::providers::moonshot::Client::builder()
                    .api_key(&config.api_key)
                    .base_url(&config.api_base_url)
                    .build()?;
                Ok(ProviderClient::Moonshot(client))
            }
            LLMProvider::DeepSeek => {
                let client = rig::providers::deepseek::Client::builder()
                    .api_key(&config.api_key)
                    .base_url(&config.api_base_url)
                    .build()?;
                Ok(ProviderClient::DeepSeek(client))
            }
            LLMProvider::Mistral => {
                let client = rig::providers::mistral::Client::new(&config.api_key)?;
                Ok(ProviderClient::Mistral(client))
            }
            LLMProvider::OpenRouter => {
                let client = rig::providers::openrouter::Client::new(&config.api_key)?;
                Ok(ProviderClient::OpenRouter(client))
            }
            LLMProvider::Anthropic => {
                let client = if config.api_base_url != "https://api.anthropic.com" {
                    rig::providers::anthropic::Client::builder()
                        .api_key(&config.api_key)
                        .base_url(&config.api_base_url)
                        .build()?
                } else {
                    rig::providers::anthropic::Client::new(&config.api_key)?
                };
                Ok(ProviderClient::Anthropic(client))
            }
            LLMProvider::Gemini => {
                let client = rig::providers::gemini::Client::new(&config.api_key)?;
                Ok(ProviderClient::Gemini(client))
            }
            LLMProvider::Ollama => {
                let client = rig::providers::ollama::Client::builder()
                    .api_key(rig::client::Nothing)
                    .base_url(&config.api_base_url)
                    .build()?;
                Ok(ProviderClient::Ollama(client))
            }
        }
    }

    /// Create Agent
    pub fn create_agent(
        &self,
        model: &str,
        system_prompt: &str,
        config: &LLMConfig,
    ) -> ProviderAgent {
        match self {
            ProviderClient::OpenAI(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into());

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder.build();
                ProviderAgent::OpenAI {
                    agent,
                    base_url: config.api_base_url.clone(),
                    model: model.to_string(),
                    api_key: config.api_key.clone(),
                }
            }
            ProviderClient::Moonshot(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder.build();
                ProviderAgent::Moonshot(agent)
            }
            ProviderClient::DeepSeek(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder.build();
                ProviderAgent::DeepSeek(agent)
            }
            ProviderClient::Mistral(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder.build();
                ProviderAgent::Mistral(agent)
            }
            ProviderClient::OpenRouter(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder.build();
                ProviderAgent::OpenRouter(agent)
            }
            ProviderClient::Anthropic(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into());

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder.build();
                ProviderAgent::Anthropic(agent)
            }
            ProviderClient::Gemini(client) => {
                let gen_cfg = GenerationConfig::default();
                let cfg = AdditionalParameters::default().with_config(gen_cfg);

                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into());

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder
                    .additional_params(serde_json::to_value(cfg).unwrap())
                    .build();
                ProviderAgent::Gemini(agent)
            }
            ProviderClient::Ollama(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into());

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder.build();
                ProviderAgent::Ollama(agent)
            }
        }
    }

    /// Create Agent with tools
    pub fn create_agent_with_tools(
        &self,
        model: &str,
        system_prompt: &str,
        config: &LLMConfig,
        file_explorer: &crate::llm::tools::file_explorer::AgentToolFileExplorer,
        file_reader: &crate::llm::tools::file_reader::AgentToolFileReader,
    ) -> ProviderAgent {
        let tool_time = AgentToolTime::new();

        match self {
            ProviderClient::OpenAI(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .default_max_turns(config.max_turns);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder
                    .tool(file_explorer.clone())
                    .tool(file_reader.clone())
                    .tool(tool_time)
                    .build();
                ProviderAgent::OpenAI {
                    agent,
                    base_url: config.api_base_url.clone(),
                    model: model.to_string(),
                    api_key: config.api_key.clone(),
                }
            }
            ProviderClient::Moonshot(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .default_max_turns(config.max_turns);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder
                    .tool(file_explorer.clone())
                    .tool(file_reader.clone())
                    .tool(tool_time)
                    .build();
                ProviderAgent::Moonshot(agent)
            }
            ProviderClient::DeepSeek(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .default_max_turns(config.max_turns);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder
                    .tool(file_explorer.clone())
                    .tool(file_reader.clone())
                    .tool(tool_time)
                    .build();
                ProviderAgent::DeepSeek(agent)
            }
            ProviderClient::Mistral(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .default_max_turns(config.max_turns);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder
                    .tool(file_explorer.clone())
                    .tool(file_reader.clone())
                    .tool(tool_time)
                    .build();
                ProviderAgent::Mistral(agent)
            }
            ProviderClient::OpenRouter(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .default_max_turns(config.max_turns);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder
                    .tool(file_explorer.clone())
                    .tool(file_reader.clone())
                    .tool(tool_time)
                    .build();
                ProviderAgent::OpenRouter(agent)
            }
            ProviderClient::Anthropic(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .default_max_turns(config.max_turns);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder
                    .tool(file_explorer.clone())
                    .tool(file_reader.clone())
                    .tool(tool_time)
                    .build();
                ProviderAgent::Anthropic(agent)
            }
            ProviderClient::Gemini(client) => {
                let gen_cfg = GenerationConfig::default();
                let cfg = AdditionalParameters::default().with_config(gen_cfg);

                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .default_max_turns(config.max_turns);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder
                    .tool(file_explorer.clone())
                    .tool(file_reader.clone())
                    .tool(tool_time)
                    .additional_params(serde_json::to_value(cfg).unwrap())
                    .build();
                ProviderAgent::Gemini(agent)
            }
            ProviderClient::Ollama(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .default_max_turns(config.max_turns);

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder
                    .tool(file_explorer.clone())
                    .tool(file_reader.clone())
                    .tool(tool_time)
                    .build();
                ProviderAgent::Ollama(agent)
            }
        }
    }

    /// Create Extractor
    pub fn create_extractor<T>(
        &self,
        model: &str,
        system_prompt: &str,
        config: &LLMConfig,
    ) -> ProviderExtractor<T>
    where
        T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static,
    {
        match self {
            ProviderClient::OpenAI(client) => {
                // Create agent for OpenAI-compatible provider
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into());

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder.build();

                // Wrap with OpenAICompatibleExtractorWrapper for HTTP fallback
                let wrapper = OpenAICompatibleExtractorWrapper::new(
                    agent,
                    config.retry_attempts,
                    config.api_base_url.clone(),
                    model.to_string(),
                    config.api_key.clone(),
                );

                ProviderExtractor::OpenAI(wrapper)
            }
            ProviderClient::Moonshot(client) => {
                let extractor = client
                    .extractor::<T>(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .build();
                ProviderExtractor::Moonshot(extractor)
            }
            ProviderClient::DeepSeek(client) => {
                let extractor = client
                    .extractor::<T>(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .build();
                ProviderExtractor::DeepSeek(extractor)
            }
            ProviderClient::Mistral(client) => {
                let extractor = client
                    .extractor::<T>(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .build();
                ProviderExtractor::Mistral(extractor)
            }
            ProviderClient::OpenRouter(client) => {
                let extractor = client
                    .extractor::<T>(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .build();
                ProviderExtractor::OpenRouter(extractor)
            }
            ProviderClient::Anthropic(client) => {
                let extractor = client
                    .extractor::<T>(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .build();
                ProviderExtractor::Anthropic(extractor)
            }
            ProviderClient::Gemini(client) => {
                let gen_cfg = GenerationConfig::default();
                let cfg = AdditionalParameters::default().with_config(gen_cfg);

                let extractor = client
                    .extractor::<T>(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .additional_params(serde_json::to_value(cfg).unwrap())
                    .build();
                ProviderExtractor::Gemini(extractor)
            }
            ProviderClient::Ollama(client) => {
                // Create standard agent for Ollama
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into());

                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }

                let agent = builder.build();

                // Wrap with OllamaExtractorWrapper to handle structured output
                // Pass base_url and model for HTTP fallback when rig fails
                let wrapper = OllamaExtractorWrapper::with_config(
                    agent,
                    config.retry_attempts,
                    config.api_base_url.clone(),
                    model.to_string(),
                );

                ProviderExtractor::Ollama(wrapper)
            }
        }
    }
}

/// Unified Agent enum
pub enum ProviderAgent {
    OpenAI {
        agent: Agent<rig::providers::openai::completion::CompletionModel>,
        base_url: String,
        model: String,
        api_key: String,
    },
    Mistral(Agent<rig::providers::mistral::CompletionModel>),
    OpenRouter(Agent<rig::providers::openrouter::CompletionModel>),
    Anthropic(Agent<rig::providers::anthropic::completion::CompletionModel>),
    Gemini(Agent<rig::providers::gemini::completion::CompletionModel>),
    Moonshot(Agent<rig::providers::moonshot::CompletionModel>),
    DeepSeek(Agent<rig::providers::deepseek::CompletionModel>),
    Ollama(Agent<rig::providers::ollama::CompletionModel>),
}

impl ProviderAgent {
    /// Execute prompt with HTTP fallback for OpenAI-compatible providers
    pub async fn prompt(&self, prompt: &str, concurrency: usize) -> Result<String> {
        match self {
            ProviderAgent::OpenAI { agent, base_url, model, api_key } => {
                // Try rig agent first with concurrency
                match agent.prompt(prompt).with_tool_concurrency(concurrency).await {
                    Ok(result) => Ok(result),
                    Err(e) => {
                        let error_msg = format!("{:?}", e);
                        // Check if it's an API response parsing error
                        if error_msg.contains("ApiResponse")
                            || error_msg.contains("untagged enum")
                            || error_msg.contains("JsonError")
                        {
                            // Fallback to direct HTTP call
                            Self::prompt_via_http(base_url, model, api_key, prompt).await
                        } else {
                            Err(e.into())
                        }
                    }
                }
            }
            ProviderAgent::Moonshot(agent) => {
                agent.prompt(prompt).with_tool_concurrency(concurrency).await.map_err(|e| e.into())
            }
            ProviderAgent::DeepSeek(agent) => {
                agent.prompt(prompt).with_tool_concurrency(concurrency).await.map_err(|e| e.into())
            }
            ProviderAgent::Mistral(agent) => {
                agent.prompt(prompt).with_tool_concurrency(concurrency).await.map_err(|e| e.into())
            }
            ProviderAgent::OpenRouter(agent) => {
                agent.prompt(prompt).with_tool_concurrency(concurrency).await.map_err(|e| e.into())
            }
            ProviderAgent::Anthropic(agent) => {
                agent.prompt(prompt).with_tool_concurrency(concurrency).await.map_err(|e| e.into())
            }
            ProviderAgent::Gemini(agent) => {
                agent.prompt(prompt).with_tool_concurrency(concurrency).await.map_err(|e| e.into())
            }
            ProviderAgent::Ollama(agent) => {
                agent.prompt(prompt).with_tool_concurrency(concurrency).await.map_err(|e| e.into())
            }
        }
    }

    /// Direct HTTP call to OpenAI-compatible API
    async fn prompt_via_http(base_url: &str, model: &str, api_key: &str, prompt: &str) -> Result<String> {
        let client = reqwest::Client::new();

        let request_body = serde_json::json!({
            "model": model,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.7,
            "max_tokens": 4096
        });

        let response = client
            .post(format!("{}/chat/completions", base_url.trim_end_matches('/')))
            .header("Authorization", format!("Bearer {}", api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .timeout(std::time::Duration::from_secs(120))
            .send()
            .await
            .map_err(|e| anyhow::anyhow!("HTTP request failed: {}", e))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            anyhow::bail!("OpenAI-compatible API HTTP error {}: {}", status, body);
        }

        let json: serde_json::Value = response
            .json()
            .await
            .map_err(|e| anyhow::anyhow!("Failed to parse HTTP response: {}", e))?;

        let content = json
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|c| c.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|c| c.as_str())
            .ok_or_else(|| anyhow::anyhow!("Invalid OpenAI API response format"))?;

        Ok(content.to_string())
    }
}

/// Unified Extractor enum
pub enum ProviderExtractor<T>
where
    T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static,
{
    OpenAI(OpenAICompatibleExtractorWrapper<T>),
    Mistral(Extractor<rig::providers::mistral::CompletionModel, T>),
    OpenRouter(Extractor<rig::providers::openrouter::CompletionModel, T>),
    Anthropic(Extractor<rig::providers::anthropic::completion::CompletionModel, T>),
    Gemini(Extractor<rig::providers::gemini::completion::CompletionModel, T>),
    Moonshot(Extractor<rig::providers::moonshot::CompletionModel, T>),
    DeepSeek(Extractor<rig::providers::deepseek::CompletionModel, T>),
    Ollama(OllamaExtractorWrapper<T>),
}

impl<T> ProviderExtractor<T>
where
    T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static,
{
    /// Execute extraction
    pub async fn extract(&self, prompt: &str) -> Result<T> {
        match self {
            ProviderExtractor::OpenAI(extractor) => {
                extractor.extract(prompt).await.map_err(|e| e.into())
            }
            ProviderExtractor::Moonshot(extractor) => {
                extractor.extract(prompt).await.map_err(|e| e.into())
            }
            ProviderExtractor::DeepSeek(extractor) => {
                extractor.extract(prompt).await.map_err(|e| e.into())
            }
            ProviderExtractor::Mistral(extractor) => {
                extractor.extract(prompt).await.map_err(|e| e.into())
            }
            ProviderExtractor::OpenRouter(extractor) => {
                extractor.extract(prompt).await.map_err(|e| e.into())
            }
            ProviderExtractor::Anthropic(extractor) => {
                extractor.extract(prompt).await.map_err(|e| e.into())
            }
            ProviderExtractor::Gemini(extractor) => {
                extractor.extract(prompt).await.map_err(|e| e.into())
            }
            ProviderExtractor::Ollama(extractor) => {
                extractor.extract(prompt).await.map_err(|e| e.into())
            }
        }
    }
}
