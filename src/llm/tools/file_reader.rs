//! File reading tool

use anyhow::Result;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};

use crate::{config::Config, utils::file_utils::is_binary_file_path};

/// File reading tool
#[derive(Debug, Clone)]
pub struct AgentToolFileReader {
    config: Config,
}

/// File reading parameters
#[derive(Debug, Deserialize)]
pub struct FileReaderArgs {
    pub file_path: String,
    pub start_line: Option<usize>,
    pub end_line: Option<usize>,
    pub max_lines: Option<usize>,
}

/// File reading result
#[derive(Debug, Serialize, Default)]
pub struct FileReaderResult {
    pub content: String,
    pub file_path: String,
    pub total_lines: usize,
    pub read_lines: usize,
    pub file_size: u64,
    pub encoding: String,
}

impl AgentToolFileReader {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    /// Resolve an agent-supplied path relative to the project root, refusing
    /// absolute paths and `..` traversal that would escape the root.
    fn resolve_within_root(&self, file_path: &str) -> Option<std::path::PathBuf> {
        let p = std::path::Path::new(file_path);
        if p.is_absolute() {
            return None;
        }
        let candidate = self.config.project_path.join(p);
        // Walk the candidate, collapsing ".." components, and verify it stays
        // inside the project root.
        let mut normalized = std::path::PathBuf::new();
        for comp in candidate.components() {
            match comp {
                std::path::Component::ParentDir => {
                    if !normalized.pop() {
                        return None;
                    }
                }
                std::path::Component::CurDir => {}
                other => normalized.push(other.as_os_str()),
            }
        }
        if normalized.starts_with(&self.config.project_path) {
            Some(normalized)
        } else {
            None
        }
    }

    async fn read_file_content(&self, args: &FileReaderArgs) -> Result<FileReaderResult> {
        let Some(file_path) = self.resolve_within_root(&args.file_path) else {
            return Ok(FileReaderResult {
                file_path: args.file_path.clone(),
                ..Default::default()
            });
        };

        if !file_path.exists() {
            return Ok(FileReaderResult {
                file_path: args.file_path.clone(),
                ..Default::default()
            });
        }

        if is_binary_file_path(&file_path) {
            return Ok(FileReaderResult {
                file_path: args.file_path.clone(),
                ..Default::default()
            });
        }

        let metadata = tokio::fs::metadata(&file_path).await?;
        let full_content = tokio::fs::read_to_string(&file_path).await?;
        let lines: Vec<&str> = full_content.lines().collect();
        let total_lines = lines.len();

        let (content, read_lines) =
            if let (Some(start), Some(end)) = (args.start_line, args.end_line) {
                let start_idx = (start.saturating_sub(1)).min(lines.len());
                let end_idx = end.min(lines.len());
                if start_idx >= end_idx {
                    return Ok(FileReaderResult {
                        file_path: args.file_path.clone(),
                        total_lines,
                        ..Default::default()
                    });
                }
                let selected_lines = &lines[start_idx..end_idx];
                (selected_lines.join("\n"), selected_lines.len())
            } else if let Some(max_lines) = args.max_lines {
                let selected_lines = &lines[..max_lines.min(lines.len())];
                (selected_lines.join("\n"), selected_lines.len())
            } else {
                // Check if file is too large, limit read lines
                let max_default_lines = 200;
                if lines.len() > max_default_lines {
                    let selected_lines = &lines[..max_default_lines];
                    (
                        format!(
                            "{}\n\n... (File too large, showing only first {} lines)",
                            selected_lines.join("\n"),
                            max_default_lines
                        ),
                        selected_lines.len(),
                    )
                } else {
                    (full_content, total_lines)
                }
            };

        Ok(FileReaderResult {
            content,
            file_path: args.file_path.clone(),
            total_lines,
            read_lines,
            file_size: metadata.len(),
            encoding: "UTF-8".to_string(),
        })
    }
}

#[derive(Debug, thiserror::Error)]
#[error("file reader tool error")]
pub struct FileReaderToolError;

impl Tool for AgentToolFileReader {
    const NAME: &'static str = "file_reader";

    type Error = FileReaderToolError;
    type Args = FileReaderArgs;
    type Output = FileReaderResult;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description: "Read source code or text-based content from the project, with support for specifying line ranges and maximum line limits. Automatically handles large files and binary files."
                .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "File path to read (relative to project root)"
                    },
                    "start_line": {
                        "type": "integer",
                        "description": "Start line number (1-based, inclusive)"
                    },
                    "end_line": {
                        "type": "integer",
                        "description": "End line number (inclusive)"
                    },
                    "max_lines": {
                        "type": "integer",
                        "description": "Maximum number of lines to read (from file start, default is 200)"
                    }
                },
                "required": ["file_path"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("   🔧 tool called...file_reader@{:?}", args);

        self.read_file_content(&args)
            .await
            .map_err(|_e| FileReaderToolError)
    }
}
