//! File system exploration tool

use anyhow::Result;
use rig::tool::Tool;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
#[cfg(debug_assertions)]
use std::time::Duration;
use walkdir::WalkDir;

use crate::config::Config;
use crate::types::FileInfo;
use crate::utils::file_utils::is_test_file;

/// File exploration tool
#[derive(Debug, Clone)]
pub struct AgentToolFileExplorer {
    config: Config,
}

/// File exploration parameters
#[derive(Debug, Deserialize)]
pub struct FileExplorerArgs {
    pub action: String, // "list_directory", "find_files", "get_file_info"
    pub path: Option<String>,
    pub pattern: Option<String>,
    pub recursive: Option<bool>,
    pub max_files: Option<usize>,
}

/// File exploration result
#[derive(Debug, Serialize, Default)]
pub struct FileExplorerResult {
    pub files: Vec<FileInfo>,
    pub directories: Vec<String>,
    pub total_count: usize,
    pub insights: Vec<String>,
    pub file_types: HashMap<String, usize>,
}

impl AgentToolFileExplorer {
    pub fn new(config: Config) -> Self {
        Self { config }
    }

    async fn list_directory(&self, args: &FileExplorerArgs) -> Result<FileExplorerResult> {
        let target_path = if let Some(path) = &args.path {
            self.config.project_path.join(path)
        } else {
            self.config.project_path.clone()
        };

        if !target_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!("Path does not exist: {}", target_path.display())],
                ..Default::default()
            });
        }

        let recursive = args.recursive.unwrap_or(false);
        let max_files = args.max_files.unwrap_or(100);
        let mut files = Vec::new();
        let mut directories = Vec::new();
        let mut file_types = HashMap::new();

        if recursive {
            // Recursive traversal, limit depth to 3
            for entry in WalkDir::new(&target_path).max_depth(3) {
                if files.len() >= max_files {
                    break;
                }

                let entry = entry?;
                let path = entry.path();

                if self.is_ignored(path) {
                    continue;
                }

                if entry.file_type().is_file() {
                    let file_info = self.create_file_info(path)?;
                    if let Some(ext) = &file_info.extension {
                        *file_types.entry(ext.clone()).or_insert(0) += 1;
                    }
                    files.push(file_info);
                } else if entry.file_type().is_dir() && path != target_path {
                    let relative_path = path
                        .strip_prefix(&self.config.project_path)
                        .unwrap_or(path)
                        .to_string_lossy()
                        .to_string();
                    directories.push(relative_path);
                }
            }
        } else {
            // Non-recursive, only list current directory
            for entry in std::fs::read_dir(&target_path)? {
                if files.len() >= max_files {
                    break;
                }

                let entry = entry?;
                let path = entry.path();

                if self.is_ignored(&path) {
                    continue;
                }

                if entry.file_type()?.is_file() {
                    let file_info = self.create_file_info(&path)?;
                    if let Some(ext) = &file_info.extension {
                        *file_types.entry(ext.clone()).or_insert(0) += 1;
                    }
                    files.push(file_info);
                } else if entry.file_type()?.is_dir() {
                    let relative_path = path
                        .strip_prefix(&self.config.project_path)
                        .unwrap_or(&path)
                        .to_string_lossy()
                        .to_string();
                    directories.push(relative_path);
                }
            }
        }

        let insights = self.generate_insights(&files, &directories, &file_types);

        Ok(FileExplorerResult {
            total_count: files.len(),
            files,
            directories,
            insights,
            file_types,
        })
    }

    async fn find_files(&self, args: &FileExplorerArgs) -> Result<FileExplorerResult> {
        let pattern = args
            .pattern
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("find_files action requires pattern parameter"))?;

        let search_path = if let Some(path) = &args.path {
            self.config.project_path.join(path)
        } else {
            self.config.project_path.clone()
        };

        if !search_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!("Search path does not exist: {}", search_path.display())],
                ..Default::default()
            });
        }

        let max_files = args.max_files.unwrap_or(100);
        let mut files = Vec::new();
        let mut file_types = HashMap::new();

        // Use walkdir for recursive search, limit depth to 5
        for entry in WalkDir::new(&search_path).max_depth(5) {
            if files.len() >= max_files {
                break;
            }

            let entry = entry?;
            let path = entry.path();

            if !entry.file_type().is_file() || self.is_ignored(path) {
                continue;
            }

            let file_name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");

            // Simple pattern matching
            if self.matches_pattern(file_name, pattern) {
                let file_info = self.create_file_info(path)?;
                if let Some(ext) = &file_info.extension {
                    *file_types.entry(ext.clone()).or_insert(0) += 1;
                }
                files.push(file_info);
            }
        }

        let insights = vec![
            format!("Search pattern: {}", pattern),
            format!("Search path: {}", search_path.display()),
            format!("Found {} matching files", files.len()),
        ];

        Ok(FileExplorerResult {
            total_count: files.len(),
            files,
            directories: Vec::new(),
            insights,
            file_types,
        })
    }

    async fn get_file_info(&self, args: &FileExplorerArgs) -> Result<FileExplorerResult> {
        let file_path = args
            .path
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("get_file_info action requires path parameter"))?;

        let target_path = self.config.project_path.join(file_path);

        if !target_path.exists() {
            return Ok(FileExplorerResult {
                insights: vec![format!("File does not exist: {}", target_path.display())],
                ..Default::default()
            });
        }

        if !target_path.is_file() {
            return Ok(FileExplorerResult {
                insights: vec![format!("Path is not a file: {}", target_path.display())],
                ..Default::default()
            });
        }

        if self.is_ignored(&target_path) {
            return Ok(FileExplorerResult {
                insights: vec![format!("File is ignored: {}", target_path.display())],
                ..Default::default()
            });
        }

        let file_info = self.create_file_info(&target_path)?;
        let mut file_types = HashMap::new();
        if let Some(ext) = &file_info.extension {
            file_types.insert(ext.clone(), 1);
        }

        let insights = vec![
            format!("File path: {}", file_info.path.display()),
            format!("File size: {} bytes", file_info.size),
            format!(
                "File extension: {}",
                file_info.extension.as_deref().unwrap_or("none")
            ),
            format!("Importance score: {:.2}", file_info.importance_score),
            format!(
                "Last modified: {}",
                file_info.last_modified.as_deref().unwrap_or("unknown")
            ),
        ];

        Ok(FileExplorerResult {
            total_count: 1,
            files: vec![file_info],
            directories: Vec::new(),
            insights,
            file_types,
        })
    }

    fn is_ignored(&self, path: &Path) -> bool {
        let path_str = path.to_string_lossy().to_lowercase();
        let file_name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("")
            .to_lowercase();

        // Check excluded directories
        for excluded_dir in &self.config.excluded_dirs {
            if path_str.contains(&excluded_dir.to_lowercase()) {
                return true;
            }
        }

        // Check excluded files
        for excluded_file in &self.config.excluded_files {
            if excluded_file.contains('*') {
                // Simple wildcard matching
                let pattern = excluded_file.replace('*', "");
                if file_name.contains(&pattern.to_lowercase()) {
                    return true;
                }
            } else if file_name == excluded_file.to_lowercase() {
                return true;
            }
        }

        // Check excluded extensions
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            if self
                .config
                .excluded_extensions
                .contains(&extension.to_lowercase())
            {
                return true;
            }
        }

        // Check included extensions (if specified)
        if !self.config.included_extensions.is_empty() {
            if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
                if !self
                    .config
                    .included_extensions
                    .contains(&extension.to_lowercase())
                {
                    return true;
                }
            } else {
                return true; // No extension and included list is specified
            }
        }

        // Check test files (if not including test files)
        if !self.config.include_tests && is_test_file(path) {
            return true;
        }

        // Check hidden files
        if !self.config.include_hidden && file_name.starts_with('.') {
            return true;
        }

        // Check file size
        if let Ok(metadata) = std::fs::metadata(path) {
            if metadata.len() > self.config.max_file_size {
                return true;
            }
        }

        false
    }

    fn create_file_info(&self, path: &Path) -> Result<FileInfo> {
        let metadata = std::fs::metadata(path)?;

        let name = path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string());

        let relative_path = path
            .strip_prefix(&self.config.project_path)
            .unwrap_or(path)
            .to_path_buf();

        let last_modified = metadata
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|duration| duration.as_secs().to_string());

        // Calculate simple importance score
        let importance_score = self.calculate_importance_score(path, &metadata);

        Ok(FileInfo {
            path: relative_path,
            name,
            size: metadata.len(),
            extension,
            is_core: importance_score > 0.5,
            importance_score,
            complexity_score: 0.0, // Temporarily set to 0, can be extended later
            last_modified,
        })
    }

    fn calculate_importance_score(&self, path: &Path, metadata: &std::fs::Metadata) -> f64 {
        let mut score: f64 = 0.0;

        // Weight based on file location
        let path_str = path.to_string_lossy().to_lowercase();
        if path_str.contains("src") || path_str.contains("lib") {
            score += 0.3;
        }
        if path_str.contains("main") || path_str.contains("index") {
            score += 0.2;
        }
        if path_str.contains("config") || path_str.contains("setup") {
            score += 0.1;
        }

        // Weight based on file size
        let size = metadata.len();
        if size > 1000 && size < 50000 {
            score += 0.2;
        }

        // Weight based on file type
        if let Some(extension) = path.extension().and_then(|e| e.to_str()) {
            match extension.to_lowercase().as_str() {
                // Main programming languages
                "rs" | "py" | "java" | "kt" | "cpp" | "c" | "go" | "rb" | "php" | "m" | "swift"
                | "dart" | "cs" => score += 0.3,
                // React special files
                "jsx" | "tsx" => score += 0.3,
                // JavaScript/TypeScript ecosystem
                "js" | "ts" | "mjs" | "cjs" => score += 0.3,
                // Frontend framework files
                "vue" | "svelte" => score += 0.3,
                // Mini Apps
                "wxml" | "ttml" | "ksml" => score += 0.3,
                // Configuration files
                "toml" | "yaml" | "yml" | "json" | "xml" | "ini" | "env" => score += 0.1,
                // Build and package management files
                "gradle" | "pom" | "csproj" | "sln" => score += 0.15,
                "package" => score += 0.15,
                "lock" => score += 0.05,
                // Style files
                "css" | "scss" | "sass" | "less" | "styl" | "wxss" => score += 0.1,
                // Template files
                "html" | "htm" | "hbs" | "mustache" | "ejs" => score += 0.1,
                _ => {}
            }
        }

        score.min(1.0)
    }

    fn matches_pattern(&self, file_name: &str, pattern: &str) -> bool {
        if pattern.contains('*') {
            // Simple wildcard matching
            let parts: Vec<&str> = pattern.split('*').collect();
            if parts.len() == 2 {
                let prefix = parts[0];
                let suffix = parts[1];
                return file_name.starts_with(prefix) && file_name.ends_with(suffix);
            }
        }

        // Contains matching
        file_name.to_lowercase().contains(&pattern.to_lowercase())
    }

    fn generate_insights(
        &self,
        files: &[FileInfo],
        directories: &[String],
        file_types: &HashMap<String, usize>,
    ) -> Vec<String> {
        let mut insights = Vec::new();

        insights.push(format!(
            "Found {} files and {} directories",
            files.len(),
            directories.len()
        ));

        if !file_types.is_empty() {
            let mut type_summary = String::new();
            for (ext, count) in file_types.iter() {
                if !type_summary.is_empty() {
                    type_summary.push_str(", ");
                }
                type_summary.push_str(&format!("{}: {}", ext, count));
            }
            insights.push(format!("File type distribution: {}", type_summary));
        }

        let total_size: u64 = files.iter().map(|f| f.size).sum();
        if total_size > 0 {
            insights.push(format!("Total file size: {} bytes", total_size));
        }

        let core_files: Vec<_> = files.iter().filter(|f| f.is_core).collect();
        if !core_files.is_empty() {
            insights.push(format!("Core files count: {}", core_files.len()));
        }

        insights
    }
}

#[derive(Debug, thiserror::Error)]
#[error("file explorer tool error")]
pub struct FileExplorerToolError;

impl Tool for AgentToolFileExplorer {
    const NAME: &'static str = "file_explorer";

    type Error = FileExplorerToolError;
    type Args = FileExplorerArgs;
    type Output = FileExplorerResult;

    async fn definition(&self, _prompt: String) -> rig::completion::ToolDefinition {
        rig::completion::ToolDefinition {
            name: Self::NAME.to_string(),
            description:
                "Explore project file structure, list directory contents, find specific file patterns. Supports recursive search and file filtering."
                    .to_string(),
            parameters: serde_json::json!({
                "type": "object",
                "properties": {
                    "action": {
                        "type": "string",
                        "enum": ["list_directory", "find_files", "get_file_info"],
                        "description": "Action type to execute: list_directory (list directory), find_files (find files), get_file_info (get file info)"
                    },
                    "path": {
                        "type": "string",
                        "description": "Target path (relative to project root)"
                    },
                    "pattern": {
                        "type": "string",
                        "description": "File search pattern (for find_files operation)"
                    },
                    "recursive": {
                        "type": "boolean",
                        "description": "Whether to recursively search subdirectories (default false)"
                    },
                    "max_files": {
                        "type": "integer",
                        "description": "Maximum number of files to return (default 100)"
                    }
                },
                "required": ["action"]
            }),
        }
    }

    async fn call(&self, args: Self::Args) -> Result<Self::Output, Self::Error> {
        println!("   🔧 tool called...file_reader@{:?}", args);

        tokio::time::sleep(Duration::from_secs(1)).await;

        match args.action.as_str() {
            "list_directory" => self
                .list_directory(&args)
                .await
                .map_err(|_e| FileExplorerToolError),
            "find_files" => self
                .find_files(&args)
                .await
                .map_err(|_e| FileExplorerToolError),
            "get_file_info" => self
                .get_file_info(&args)
                .await
                .map_err(|_e| FileExplorerToolError),
            _ => Err(FileExplorerToolError),
        }
    }
}
