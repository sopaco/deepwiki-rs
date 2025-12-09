//! LLM Provider support module

use anyhow::Result;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION};
use rig::{
    agent::Agent,
    client::CompletionClient,
    completion::{Prompt, PromptError},
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

/// Unified Provider client enum
#[derive(Clone)]
pub enum ProviderClient {
    OpenAI(rig::providers::openai::Client),
    Moonshot(rig::providers::moonshot::Client),
    DeepSeek(rig::providers::deepseek::Client),
    Mistral(rig::providers::mistral::Client),
    OpenRouter(rig::providers::openrouter::Client),
    Anthropic(rig::providers::anthropic::Client),
    Gemini(rig::providers::gemini::Client),
    Ollama(rig::providers::ollama::Client<reqwest::Client>),
}

impl ProviderClient {
    /// Create corresponding provider client based on configuration
    pub fn new(config: &LLMConfig) -> Result<Self> {
        match config.provider {
            LLMProvider::OpenAI => {
                let client = rig::providers::openai::Client::builder(&config.api_key)
                    .base_url(&config.api_base_url)
                    .build();
                Ok(ProviderClient::OpenAI(client))
            }
            LLMProvider::Moonshot => {
                let client = rig::providers::moonshot::Client::builder(&config.api_key)
                    .base_url(&config.api_base_url)
                    .build();
                Ok(ProviderClient::Moonshot(client))
            }
            LLMProvider::DeepSeek => {
                let client = rig::providers::deepseek::Client::builder(&config.api_key)
                    .base_url(&config.api_base_url)
                    .build();
                Ok(ProviderClient::DeepSeek(client))
            }
            LLMProvider::Mistral => {
                let client = rig::providers::mistral::Client::builder(&config.api_key).build();
                Ok(ProviderClient::Mistral(client))
            }
            LLMProvider::OpenRouter => {
                // reference： https://docs.rig.rs/docs/integrations/model_providers/anthropic#basic-usage
                let client = rig::providers::openrouter::Client::builder(&config.api_key).build();
                Ok(ProviderClient::OpenRouter(client))
            }
            LLMProvider::Anthropic => {
                let client =
                    rig::providers::anthropic::ClientBuilder::new(&config.api_key).build()?;
                Ok(ProviderClient::Anthropic(client))
            }
            LLMProvider::Gemini => {
                let client = rig::providers::gemini::Client::builder(&config.api_key).build()?;
                Ok(ProviderClient::Gemini(client))
            }
            LLMProvider::Ollama => {
                // Create custom reqwest client with Authorization header
                let mut headers = HeaderMap::new();
                if !config.api_key.is_empty() {
                    let auth_value = format!("Bearer {}", config.api_key);
                    headers.insert(
                        AUTHORIZATION,
                        HeaderValue::from_str(&auth_value)
                            .map_err(|e| anyhow::anyhow!("Invalid API key format: {}", e))?,
                    );
                }
                let http_client = reqwest::Client::builder()
                    .default_headers(headers)
                    .build()
                    .map_err(|e| anyhow::anyhow!("Failed to build HTTP client: {}", e))?;

                let client = rig::providers::ollama::Client::builder()
                    .base_url(&config.api_base_url)
                    .with_client(http_client)
                    .build();
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
                    .completion_model(model)
                    .completions_api()
                    .into_agent_builder()
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into());
                
                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }
                
                let agent = builder.build();
                ProviderAgent::OpenAI(agent)
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
                    .completion_model(model)
                    .completions_api()
                    .into_agent_builder()
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into());
                
                if let Some(temp) = config.temperature {
                    builder = builder.temperature(temp);
                }
                
                let agent = builder
                    .tool(file_explorer.clone())
                    .tool(file_reader.clone())
                    .tool(tool_time)
                    .build();
                ProviderAgent::OpenAI(agent)
            }
            ProviderClient::Moonshot(client) => {
                let mut builder = client
                    .agent(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into());
                
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
                    .max_tokens(config.max_tokens.into());
                
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
                    .preamble(system_prompt);
                
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
                    .preamble(system_prompt);
                
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
                    .max_tokens(config.max_tokens.into());
                
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
                    .max_tokens(config.max_tokens.into());
                
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
                    .max_tokens(config.max_tokens.into());
                
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
                let extractor = client
                    .extractor_completions_api::<T>(model)
                    .preamble(system_prompt)
                    .max_tokens(config.max_tokens.into())
                    .build();
                ProviderExtractor::OpenAI(extractor)
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
                let wrapper = OllamaExtractorWrapper::new(agent, config.retry_attempts);

                ProviderExtractor::Ollama(wrapper)
            }
        }
    }
}

/// Unified Agent enum
pub enum ProviderAgent {
    OpenAI(Agent<rig::providers::openai::CompletionModel>),
    Mistral(Agent<rig::providers::mistral::CompletionModel>),
    OpenRouter(Agent<rig::providers::openrouter::CompletionModel>),
    Anthropic(Agent<rig::providers::anthropic::completion::CompletionModel>),
    Gemini(Agent<rig::providers::gemini::completion::CompletionModel>),
    Moonshot(Agent<rig::providers::moonshot::CompletionModel>),
    DeepSeek(Agent<rig::providers::deepseek::CompletionModel>),
    Ollama(Agent<rig::providers::ollama::CompletionModel<reqwest::Client>>),
}

impl ProviderAgent {
    /// Execute prompt
    pub async fn prompt(&self, prompt: &str) -> Result<String> {
        match self {
            ProviderAgent::OpenAI(agent) => agent.prompt(prompt).await.map_err(|e| e.into()),
            ProviderAgent::Moonshot(agent) => agent.prompt(prompt).await.map_err(|e| e.into()),
            ProviderAgent::DeepSeek(agent) => agent.prompt(prompt).await.map_err(|e| e.into()),
            ProviderAgent::Mistral(agent) => agent.prompt(prompt).await.map_err(|e| e.into()),
            ProviderAgent::OpenRouter(agent) => agent.prompt(prompt).await.map_err(|e| e.into()),
            ProviderAgent::Anthropic(agent) => agent.prompt(prompt).await.map_err(|e| e.into()),
            ProviderAgent::Gemini(agent) => agent.prompt(prompt).await.map_err(|e| e.into()),
            ProviderAgent::Ollama(agent) => agent.prompt(prompt).await.map_err(|e| e.into()),
        }
    }

    /// Execute multi-turn dialogue
    pub async fn multi_turn(
        &self,
        prompt: &str,
        max_iterations: usize,
    ) -> Result<String, PromptError> {
        match self {
            ProviderAgent::OpenAI(agent) => agent.prompt(prompt).multi_turn(max_iterations).await,
            ProviderAgent::Moonshot(agent) => agent.prompt(prompt).multi_turn(max_iterations).await,
            ProviderAgent::DeepSeek(agent) => agent.prompt(prompt).multi_turn(max_iterations).await,
            ProviderAgent::Mistral(agent) => agent.prompt(prompt).multi_turn(max_iterations).await,
            ProviderAgent::OpenRouter(agent) => {
                agent.prompt(prompt).multi_turn(max_iterations).await
            }
            ProviderAgent::Anthropic(agent) => {
                agent.prompt(prompt).multi_turn(max_iterations).await
            }
            ProviderAgent::Gemini(agent) => agent.prompt(prompt).multi_turn(max_iterations).await,
            ProviderAgent::Ollama(agent) => agent.prompt(prompt).multi_turn(max_iterations).await,
        }
    }
}

/// Unified Extractor enum
pub enum ProviderExtractor<T>
where
    T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static,
{
    OpenAI(Extractor<rig::providers::openai::CompletionModel, T>),
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
