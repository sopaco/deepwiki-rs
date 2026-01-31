//! ReAct executor - Responsible for executing ReAct pattern multi-turn dialogue logic

use anyhow::Result;
use rig::completion::{AssistantContent, Message, PromptError};

use crate::i18n::TargetLanguage;
use super::react::{ReActConfig, ReActResponse};
use super::providers::ProviderAgent;

/// ReAct executor
pub struct ReActExecutor;

impl ReActExecutor {
    /// Execute ReAct loop logic
    pub async fn execute(
        agent: &ProviderAgent,
        user_prompt: &str,
        config: &ReActConfig,
        target_language: &TargetLanguage,
        model_name: &str,
    ) -> Result<ReActResponse> {
        if config.verbose {
            println!(
                "   ♻️ Activating ReAct Agent mode, max iterations: {}",
                config.max_iterations
            );
        }

        let mut tool_calls_history = Vec::new();

        match agent.multi_turn(user_prompt, config.max_iterations).await {
            Ok(response) => {
                if config.verbose {
                    println!("   ✅ ReAct Agent task completed");
                }

                Ok(ReActResponse::success(response, config.max_iterations))
            }
            Err(PromptError::MaxDepthError {
                max_depth,
                chat_history,
                prompt: _,
            }) => {
                if config.verbose {
                    let msg = target_language.msg_max_iterations();
                    println!("{}", msg.replace("{}", &max_depth.to_string()));
                }

                if config.return_partial_on_max_depth {
                    let (content, tool_calls) = Self::extract_partial_result(&chat_history);
                    tool_calls_history.extend(tool_calls);

                    Ok(ReActResponse::max_depth_reached_with_history(
                        format!(
                            "{}\n\n[Notice: Interrupted due to reaching max iterations ({})]",
                            content, max_depth
                        ),
                        max_depth,
                        tool_calls_history,
                        chat_history.to_vec(),
                    ))
                } else {
                    Err(anyhow::anyhow!(
                        "ReAct Agent task incomplete due to reaching max iterations ({})",
                        max_depth
                    ))
                }
            }
            Err(e) => {
                if config.verbose {
                    println!("   ❌ ReAct Agent error: {:?}", e);
                }
                Err(anyhow::anyhow!("ReAct Agent task execution failed (model: {}): {}", model_name, e))
            }
        }
    }

    /// Extract partial result from chat history
    fn extract_partial_result(chat_history: &[Message]) -> (String, Vec<String>) {
        let mut tool_calls = Vec::new();

        // Try to extract the last assistant response from chat history
        let last_assistant_message = chat_history
            .iter()
            .rev()
            .find_map(|msg| {
                if let Message::Assistant { content, .. } = msg {
                    // 提取文本内容
                    let text_content = content
                        .iter()
                        .filter_map(|c| {
                            if let AssistantContent::Text(text) = c {
                                Some(text.text.clone())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("\n");

                    if !text_content.is_empty() {
                        Some(text_content)
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| {
                "ReAct Agent interrupted due to reaching max iterations, unable to obtain complete response.".to_string()
            });

        // Extract tool call information from chat history
        for msg in chat_history {
            if let Message::Assistant { content, .. } = msg {
                for c in content.iter() {
                    if let AssistantContent::ToolCall(tool_call) = c {
                        tool_calls.push(format!(
                            "{}({})",
                            tool_call.function.name, tool_call.function.arguments
                        ));
                    }
                }
            }
        }

        (last_assistant_message, tool_calls)
    }
}
