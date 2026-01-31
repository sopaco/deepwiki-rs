//! LLM client - Provides unified LLM service interface

use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::future::Future;

use crate::{config::Config, llm::client::utils::evaluate_befitting_model};

mod agent_builder;
mod ollama_extractor;
mod providers;
mod react;
mod react_executor;
mod summary_reasoner;
pub mod types;
pub mod utils;

pub use react::{ReActConfig, ReActResponse};

use agent_builder::AgentBuilder;
use providers::ProviderClient;
use react_executor::ReActExecutor;
use summary_reasoner::SummaryReasoner;

/// LLM client - Provides unified LLM service interface
#[derive(Clone)]
pub struct LLMClient {
    config: Config,
    client: ProviderClient,
}

impl LLMClient {
    /// Create a new LLM client
    pub fn new(config: Config) -> Result<Self> {
        let client = ProviderClient::new(&config.llm)?;
        Ok(Self { client, config })
    }

    /// Get Agent builder
    fn get_agent_builder(&self) -> AgentBuilder<'_> {
        AgentBuilder::new(&self.client, &self.config)
    }

    /// Generic retry logic for handling async operation retry mechanism
    async fn retry_with_backoff<T, F, Fut>(&self, operation: F) -> Result<T>
    where
        F: Fn() -> Fut,
        Fut: Future<Output = Result<T, anyhow::Error>>,
    {
        let llm_config = &self.config.llm;
        let max_retries = llm_config.retry_attempts;
        let retry_delay_ms = llm_config.retry_delay_ms;
        let mut retries = 0;

        loop {
            match operation().await {
                Ok(result) => return Ok(result),
                Err(err) => {
                    retries += 1;
                    eprintln!(
                        "❌ Model service call error, retrying (attempt {} / {}): {}",
                        retries, max_retries, err
                    );
                    if retries >= max_retries {
                        return Err(err);
                    }
                    tokio::time::sleep(std::time::Duration::from_millis(retry_delay_ms)).await;
                }
            }
        }
    }

    /// Data extraction method
    pub async fn extract<T>(&self, system_prompt: &str, user_prompt: &str) -> Result<T>
    where
        T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static,
    {
        let (befitting_model, fallover_model) =
            evaluate_befitting_model(&self.config.llm, system_prompt, user_prompt);

        self.extract_inner(system_prompt, user_prompt, befitting_model, fallover_model)
            .await
    }

    async fn extract_inner<T>(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        befitting_model: String,
        fallover_model: Option<String>,
    ) -> Result<T>
    where
        T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static,
    {
        let llm_config = &self.config.llm;

        let extractor =
            self.client
                .create_extractor::<T>(&befitting_model, system_prompt, llm_config);

        self.retry_with_backoff(|| async {
            match extractor.extract(user_prompt).await {
                Ok(r) => Ok(r),
                Err(e) => match fallover_model {
                    Some(ref model) => {
                        let msg = self.config.target_language.msg_ai_service_error()
                            .replacen("{}", &llm_config.retry_attempts.to_string(), 1)
                            .replacen("{}", &format!(" trying fallback model {}...{}", model, e), 1);
                        eprintln!("{}", msg);
                        let user_prompt_with_fixer = format!("{}\n\n**Notice** There was an error during my previous LLM call, error message: \"{}\". Please ensure you avoid this error this time", user_prompt, e);
                        Box::pin(self.extract_inner(
                            system_prompt,
                            &user_prompt_with_fixer,
                            model.clone(),
                            None,
                        ))
                        .await
                    }
                    None => {
                        let msg = self.config.target_language.msg_ai_service_error()
                            .replacen("{}", &llm_config.retry_attempts.to_string(), 1)
                            .replacen("{}", &e.to_string(), 1);
                        eprintln!("{}", msg);
                        Err(e.into())
                    }
                },
            }
        })
        .await
    }

    /// Intelligent dialogue method (using default ReAct configuration)
    pub async fn prompt(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let react_config = ReActConfig::default();
        let response = self
            .prompt_with_react(system_prompt, user_prompt, react_config)
            .await?;
        Ok(response.content)
    }

    /// Multi-turn dialogue using ReAct mode
    pub async fn prompt_with_react(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        react_config: ReActConfig,
    ) -> Result<ReActResponse> {
        let agent_builder = self.get_agent_builder();
        let agent = agent_builder.build_agent_with_tools(system_prompt);
        let model_name = self.config.llm.model_efficient.clone();

        let response = self
            .retry_with_backoff(|| async {
                ReActExecutor::execute(&agent, user_prompt, &react_config, &self.config.target_language, &model_name)
                    .await
                    .map_err(|e| e.into())
            })
            .await?;

        // If max iterations reached and summary reasoning enabled, attempt fallover
        if response.stopped_by_max_depth
            && react_config.enable_summary_reasoning
            && response.chat_history.is_some()
        {
            if react_config.verbose {
                println!("🔄 Activating ReAct Agent summary to direct reasoning mode...");
            }

            match self
                .try_summary_reasoning(system_prompt, user_prompt, &response)
                .await
            {
                Ok(summary_response) => {
                    if react_config.verbose {
                        println!("✅ Summary reasoning completed");
                    }
                    return Ok(summary_response);
                }
                Err(e) => {
                    if react_config.verbose {
                        let msg = self.config.target_language.msg_summary_reasoning_failed();
                        println!("{}", msg.replace("{}", &e.to_string()));
                    }
                    // When summary reasoning fails, return the original partial result
                }
            }
        }

        Ok(response)
    }

    /// Attempt summary reasoning fallover
    async fn try_summary_reasoning(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        original_response: &ReActResponse,
    ) -> Result<ReActResponse> {
        let agent_builder = self.get_agent_builder();
        let agent_without_tools = agent_builder.build_agent_without_tools(system_prompt);

        let chat_history = original_response
            .chat_history
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Missing chat history"))?;

        let summary_result = self
            .retry_with_backoff(|| async {
                SummaryReasoner::summarize_and_reason(
                    &agent_without_tools,
                    system_prompt,
                    user_prompt,
                    chat_history,
                    &original_response.tool_calls_history,
                )
                .await
                .map_err(|e| e.into())
            })
            .await?;

        Ok(ReActResponse::from_summary_reasoning(
            summary_result,
            original_response.iterations_used,
            original_response.tool_calls_history.clone(),
            chat_history.clone(),
        ))
    }

    /// Simplified single-turn dialogue method (without tools)
    pub async fn prompt_without_react(
        &self,
        system_prompt: &str,
        user_prompt: &str,
    ) -> Result<String> {
        let agent_builder = self.get_agent_builder();
        let agent = agent_builder.build_agent_without_tools(system_prompt);

        self.retry_with_backoff(|| async { agent.prompt(user_prompt).await.map_err(|e| e.into()) })
            .await
    }
}
