//! ReAct executor - Responsible for executing ReAct pattern logic

use anyhow::Result;

use super::react::{ReActConfig, ReActResponse};
use super::providers::ProviderAgent;

/// ReAct executor
pub struct ReActExecutor;

impl ReActExecutor {
    /// Execute ReAct loop logic
    /// Note: In rig-core 0.34, multi_turn was removed. We use prompt for single-turn
    /// and the agent handles tool calls internally.
    pub async fn execute(
        agent: &ProviderAgent,
        user_prompt: &str,
        config: &ReActConfig,
        model_name: &str,
    ) -> Result<ReActResponse> {
        if config.verbose {
            println!(
                "   ♻️ Activating Agent mode (model: {})",
                model_name
            );
        }

        match agent.prompt(user_prompt, config.concurrency).await {
            Ok(response) => {
                if config.verbose {
                    println!("   ✅ Agent task completed");
                }

                Ok(ReActResponse::success(response, config.max_iterations))
            }
            Err(e) => {
                if config.verbose {
                    println!("   ❌ Agent error: {:?}", e);
                }
                Err(anyhow::anyhow!("Agent task execution failed (model: {}): {}", model_name, e))
            }
        }
    }
}
