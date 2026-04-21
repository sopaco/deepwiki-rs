//! ReAct (Reasoning and Acting) pattern related types and configuration

use rig::completion::Message;

/// ReAct mode configuration
#[derive(Debug, Clone)]
pub struct ReActConfig {
    /// Maximum number of iterations
    pub max_iterations: usize,
    /// Whether to enable verbose logging
    pub verbose: bool,
    /// Whether to enable summary reasoning fallover mechanism
    pub enable_summary_reasoning: bool,
    /// Concurrency level for tool execution
    pub concurrency: usize,
}

impl Default for ReActConfig {
    fn default() -> Self {
        Self {
            max_iterations: 10,
            verbose: cfg!(debug_assertions),
            enable_summary_reasoning: true,
            concurrency: 4,
        }
    }
}

/// ReAct response result
#[derive(Debug, Clone)]
pub struct ReActResponse {
    /// Final response content
    pub content: String,
    /// Actual number of iterations used
    pub iterations_used: usize,
    /// Whether stopped due to reaching max iterations
    pub stopped_by_max_depth: bool,
    /// Tool call history
    pub tool_calls_history: Vec<String>,
    /// Chat history (only included when max depth reached)
    pub chat_history: Option<Vec<Message>>,
}

impl ReActResponse {
    /// Create a new ReAct response
    pub fn new(
        content: String,
        iterations_used: usize,
        stopped_by_max_depth: bool,
        tool_calls_history: Vec<String>,
        chat_history: Option<Vec<Message>>,
    ) -> Self {
        Self {
            content,
            iterations_used,
            stopped_by_max_depth,
            tool_calls_history,
            chat_history,
        }
    }

    /// Create successfully completed response
    pub fn success(content: String, iterations_used: usize) -> Self {
        Self::new(content, iterations_used, false, Vec::new(), None)
    }

    /// Create response generated through summary reasoning
    pub fn from_summary_reasoning(
        content: String,
        max_depth: usize,
        tool_calls_history: Vec<String>,
        chat_history: Vec<Message>,
    ) -> Self {
        Self::new(
            content,
            max_depth,
            true,
            tool_calls_history,
            Some(chat_history),
        )
    }
}
