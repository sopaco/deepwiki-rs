//! Time query tool

use anyhow::Result;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
#[cfg(debug_assertions)]
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

/// Time tool
#[derive(Debug, Clone)]
pub struct AgentToolTime;

/// Time query parameters
#[derive(Debug, Deserialize)]
pub struct TimeArgs {
    #[serde(rename = "format")]
    pub format: Option<String>,
}

/// Time query result
#[derive(Debug, Serialize)]
pub struct TimeResult {
    pub current_time: String,
    pub timestamp: u64,
    pub utc_time: String,
}

/// Time tool error
#[derive(Debug)]
pub struct TimeToolError;

impl std::fmt::Display for TimeToolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Time tool error")
    }
}

impl std::error::Error for TimeToolError {}

impl AgentToolTime {
    pub fn new() -> Self {
        Self
    }

    async fn get_current_time(&self, args: &TimeArgs) -> Result<TimeResult> {
        // Get current system time
        let now = SystemTime::now();
        let timestamp = now.duration_since(UNIX_EPOCH)?.as_secs();

        // Format time
        let format = args.format.as_deref().unwrap_or("%Y-%m-%d %H:%M:%S");

        // Local time
        let datetime: chrono::DateTime<chrono::Local> = now.into();
        let current_time = datetime.format(format).to_string();

        // UTC time
        let utc_datetime: chrono::DateTime<chrono::Utc> = now.into();
        let utc_time = utc_datetime.format(format).to_string();

        Ok(TimeResult {
            current_time,
            timestamp,
            utc_time,
        })
    }
}

impl Tool for AgentToolTime {
    const NAME: &'static str = "time";

    type Error = TimeToolError;
    type Args = TimeArgs;
    type Output = TimeResult;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Get current date and time information, including local time, UTC time, and timestamp.".to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "format": {
                        "type": "string",
                        "description": "Time format string (default is '%Y-%m-%d %H:%M:%S'). Supports chrono formatting syntax."
                    }
                },
                "required": []
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("   🔧 tool called...time@{:?}", args);

        tokio::time::sleep(Duration::from_secs(1)).await;

        self.get_current_time(&args)
            .await
            .map_err(|_e| TimeToolError)
    }
}
