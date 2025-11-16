//! LLM客户端 - 提供统一的LLM服务接口

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

/// LLM客户端 - 提供统一的LLM服务接口
#[derive(Clone)]
pub struct LLMClient {
    config: Config,
    client: ProviderClient,
}

impl LLMClient {
    /// 创建新的LLM客户端
    pub fn new(config: Config) -> Result<Self> {
        let client = ProviderClient::new(&config.llm)?;
        Ok(Self { client, config })
    }

    /// 获取Agent构建器
    fn get_agent_builder(&self) -> AgentBuilder<'_> {
        AgentBuilder::new(&self.client, &self.config)
    }

    /// 通用重试逻辑，用于处理异步操作的重试机制
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
                        "❌ 调用模型服务出错，重试中 (第 {} / {}次尝试): {}",
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

    /// 数据提取方法
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
                        eprintln!(
                            "❌ 调用模型服务出错，尝试 {} 次均失败，尝试使用备选模型{}...{}",
                            llm_config.retry_attempts, model, e
                        );
                        let user_prompt_with_fixer = format!("{}\n\n**注意事项**此前我调用大模型过程时存在错误，错误信息为“{}”，你注意你这一次要规避这个错误", user_prompt, e);
                        Box::pin(self.extract_inner(
                            system_prompt,
                            &user_prompt_with_fixer,
                            model.clone(),
                            None,
                        ))
                        .await
                    }
                    None => {
                        eprintln!(
                            "❌ 调用模型服务出错，尝试 {} 次均失败...{}",
                            llm_config.retry_attempts, e
                        );
                        Err(e.into())
                    }
                },
            }
        })
        .await
    }

    /// 智能对话方法（使用默认ReAct配置）
    pub async fn prompt(&self, system_prompt: &str, user_prompt: &str) -> Result<String> {
        let react_config = ReActConfig::default();
        let response = self
            .prompt_with_react(system_prompt, user_prompt, react_config)
            .await?;
        Ok(response.content)
    }

    /// 使用ReAct模式进行多轮对话
    pub async fn prompt_with_react(
        &self,
        system_prompt: &str,
        user_prompt: &str,
        react_config: ReActConfig,
    ) -> Result<ReActResponse> {
        let agent_builder = self.get_agent_builder();
        let agent = agent_builder.build_agent_with_tools(system_prompt);

        let response = self
            .retry_with_backoff(|| async {
                ReActExecutor::execute(&agent, user_prompt, &react_config)
                    .await
                    .map_err(|e| e.into())
            })
            .await?;

        // 如果达到最大迭代次数且启用了总结推理，则尝试fallover
        if response.stopped_by_max_depth
            && react_config.enable_summary_reasoning
            && response.chat_history.is_some()
        {
            if react_config.verbose {
                println!("🔄 启动ReAct Agent总结转直接推理模式...");
            }

            match self
                .try_summary_reasoning(system_prompt, user_prompt, &response)
                .await
            {
                Ok(summary_response) => {
                    if react_config.verbose {
                        println!("✅ 总结推理完成");
                    }
                    return Ok(summary_response);
                }
                Err(e) => {
                    if react_config.verbose {
                        println!("⚠️  总结推理失败，返回原始部分结果...{}", e);
                    }
                    // 总结推理失败时，返回原始的部分结果
                }
            }
        }

        Ok(response)
    }

    /// 尝试总结推理fallover
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
            .ok_or_else(|| anyhow::anyhow!("缺少对话历史"))?;

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

    /// 简化的单轮对话方法（不使用工具）
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
