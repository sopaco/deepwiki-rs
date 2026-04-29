//! Summary reasoning module - Fallover mechanism when ReAct mode reaches max iterations

use anyhow::Result;
use rig::completion::Message;

use super::providers::ProviderAgent;

/// Summary reasoner
pub struct SummaryReasoner;

impl SummaryReasoner {
    /// Summarize and reason based on ReAct chat history and tool call records
    pub async fn summarize_and_reason(
        agent_without_tools: &ProviderAgent,
        original_system_prompt: &str,
        original_user_prompt: &str,
        chat_history: &[Message],
        tool_calls_history: &[String],
    ) -> Result<String> {
        // Build summary reasoning prompt
        let summary_prompt = Self::build_summary_prompt(
            original_system_prompt,
            original_user_prompt,
            chat_history,
            tool_calls_history,
        );

        // Use agent without tools for single-turn reasoning
        let result = agent_without_tools.prompt(&summary_prompt, 1).await?;
        
        Ok(result)
    }

    /// Build summary reasoning prompt
    fn build_summary_prompt(
        original_system_prompt: &str,
        original_user_prompt: &str,
        chat_history: &[Message],
        tool_calls_history: &[String],
    ) -> String {
        let mut prompt = String::new();
        
        // Add original system prompt
        prompt.push_str("# Original Task Background\n");
        prompt.push_str(original_system_prompt);
        prompt.push_str("\n\n");
        
        // Add original user question
        prompt.push_str("# Original User Question\n");
        prompt.push_str(original_user_prompt);
        prompt.push_str("\n\n");
        
        // Add tool call history
        if !tool_calls_history.is_empty() {
            prompt.push_str("# Executed Tool Call Records\n");
            for (index, tool_call) in tool_calls_history.iter().enumerate() {
                prompt.push_str(&format!("{}. {}\n", index + 1, tool_call));
            }
            prompt.push_str("\n");
        }
        
        // Add detailed conversation history information
        let conversation_details = Self::extract_detailed_conversation_info(chat_history);
        if !conversation_details.is_empty() {
            prompt.push_str("# Detailed Conversation History and Tool Results\n");
            prompt.push_str(&conversation_details);
            prompt.push_str("\n\n");
        }
        
        // Add summary reasoning instructions
        prompt.push_str("# Summary Reasoning Task\n");
        prompt.push_str("Based on the above information, although the multi-turn reasoning process was truncated due to reaching max iterations, please provide a complete and valuable answer to the original user question based on the available context, tool call records, and conversation history. Please comprehensively analyze the obtained information and provide the best solution or answer.\n\n");
        prompt.push_str("Note:\n");
        prompt.push_str("1. Please reason based on available information, do not fabricate non-existent content\n");
        prompt.push_str("2. If information is insufficient to fully answer the question, please state the known parts and indicate aspects that need further understanding\n");
        prompt.push_str("3. Please provide specific and actionable suggestions or solutions\n");
        prompt.push_str("4. Make full use of the executed tool calls and their results to form the answer\n");
        
        prompt
    }
    
    /// Extract more detailed conversation information, including tool calls and related context
    fn extract_detailed_conversation_info(chat_history: &[Message]) -> String {
        let mut details = String::new();
        
        for (index, message) in chat_history.iter().enumerate() {
            if index == 0 { // Skip the first user input (original user prompt), as it's already included above
                continue;
            }
            match message {
                Message::System { content } => {
                    details.push_str(&format!("## System Message [Round {}]\n", index + 1));
                    details.push_str(&format!("{}\n\n", content));
                }
                Message::User { content } => {
                    // Handle user messages in more detail
                    details.push_str(&format!("## User Input [Round {}]\n", index + 1));
                    details.push_str(&format!("{:#?}\n\n", content));
                }
                Message::Assistant { content, .. } => {
                    details.push_str(&format!("## Assistant Response [Round {}]\n", index + 1));
                    
                    // Handle text content and tool calls separately
                    let mut has_content = false;
                    
                    for item in content.iter() {
                        match item {
                            rig::completion::AssistantContent::Text(text) => {
                                if !text.text.is_empty() {
                                    details.push_str(&format!("**Text Reply:** {}\n\n", text.text));
                                    has_content = true;
                                }
                            }
                            rig::completion::AssistantContent::ToolCall(tool_call) => {
                                details.push_str(&format!(
                                    "**Tool Call:** `{}` \nArguments: `{}`\n\n",
                                    tool_call.function.name, 
                                    tool_call.function.arguments
                                ));
                                has_content = true;
                            }
                            rig::completion::AssistantContent::Reasoning(reasoning) => {
                                let reasoning_text = reasoning.display_text();
                                if !reasoning_text.is_empty() {
                                    details.push_str(&format!("**Reasoning Process:** {}\n\n", reasoning_text));
                                    has_content = true;
                                }
                            }
                            rig::completion::AssistantContent::Image(_) => {
                                // Skip image content in summary
                            }
                        }
                    }
                    
                    if !has_content {
                        details.push_str("No specific content\n\n");
                    }
                }
            }
        }
        
        details
    }
}