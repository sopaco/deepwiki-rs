use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::generator::agent_executor::{AgentExecuteParams, extract};
use crate::generator::context::GeneratorContext;
use crate::generator::preprocess::extractors::language_processors::LanguageProcessorManager;
use crate::types::{DirectoryDossier, DirectoryPurpose};

/// Per-file insight from LLM
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct FileInsightResult {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub summary: String,
    #[serde(default)]
    pub code_purpose: Option<crate::types::code::CodePurpose>,
    #[serde(default)]
    pub source_summary: String,
    #[serde(default)]
    pub detailed_description: String,
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub responsibilities: Vec<String>,
    #[serde(default)]
    pub interfaces: Vec<InterfaceInfoResult>,
    #[serde(default)]
    pub dependencies: Vec<DependencyResult>,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub importance_score: f64,
}

/// Simplified interface info for LLM output
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct InterfaceInfoResult {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub interface_type: String,
    #[serde(default, deserialize_with = "deserialize_vec_parameter_info_lenient")]
    pub parameters: Vec<ParameterInfoResult>,
    #[serde(default)]
    pub return_type: String,
}

/// Simplified parameter info for LLM output
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct ParameterInfoResult {
    #[serde(default)]
    pub name: String,
    #[serde(default)]
    pub param_type: String,
}

/// Simplified dependency for LLM output
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct DependencyResult {
    #[serde(default)]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_bool_lenient")]
    pub is_external: bool,
    #[serde(default)]
    pub dependency_type: String,
}

fn deserialize_vec_parameter_info_lenient<'de, D>(deserializer: D) -> Result<Vec<ParameterInfoResult>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Array(items) => Ok(items
            .into_iter()
            .filter_map(|v| {
                let obj = v.as_object()?;
                Some(ParameterInfoResult {
                    name: obj.get("name").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                    param_type: obj.get("param_type").and_then(|v| v.as_str()).unwrap_or_default().to_string(),
                })
            })
            .collect()),
        _ => Ok(Vec::new()),
    }
}

/// Directory summary response — includes LLM-assigned importance score and per-file insights
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct DirectorySummaryResponse {
    #[serde(default)]
    pub summary: String,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub importance_score: f64,
    #[serde(default)]
    pub key_files: Vec<String>,
    #[serde(default)]
    pub file_insights: Vec<FileInsightResult>,
}

fn deserialize_f64_lenient<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Number(n) => Ok(n.as_f64().unwrap_or(0.0)),
        serde_json::Value::String(s) => Ok(s.parse::<f64>().unwrap_or(0.0)),
        serde_json::Value::Bool(v) => Ok(if v { 1.0 } else { 0.0 }),
        _ => Ok(0.0),
    }
}

fn deserialize_vec_string_lenient<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => Ok(items
            .into_iter()
            .filter_map(|v| {
                match v {
                    serde_json::Value::String(s) if !s.is_empty() => Some(s),
                    serde_json::Value::Number(n) => Some(n.to_string()),
                    serde_json::Value::Bool(v) => Some(v.to_string()),
                    _ => None,
                }
            })
            .collect()),
        serde_json::Value::String(s) if !s.is_empty() => Ok(vec![s]),
        _ => Ok(Vec::new()),
    }
}

fn deserialize_bool_lenient<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Bool(v) => Ok(v),
        serde_json::Value::String(s) => Ok(s.parse::<bool>().unwrap_or(false)),
        serde_json::Value::Number(n) => Ok(n.as_f64().unwrap_or(0.0) != 0.0),
        _ => Ok(false),
    }
}

/// File content with metadata for directory summarization
#[derive(Debug, Clone)]
pub struct FileContent {
    pub name: String,
    pub path: PathBuf,
    pub content: String,
}

/// Directory summarizer — generates DirectoryDossier with summary and key_files via LLM
pub struct DirectorySummarizer {
    language_processor: LanguageProcessorManager,
}

impl DirectorySummarizer {
    pub fn new() -> Self {
        Self {
            language_processor: LanguageProcessorManager::new(),
        }
    }

    /// Generate a DirectoryDossier for a single directory using LLM.
    /// Files should be batched by caller: if total content exceeds 256KB, summarize
    /// in batches and merge results.
    pub async fn summarize_directory(
        &self,
        context: &GeneratorContext,
        dir_info: &crate::types::DirectoryInfo,
        files: &[FileContent],
        progress: Option<(usize, usize)>,
    ) -> Result<DirectoryDossier> {
        let prompt_sys =
            "You are a professional software architecture analyst skilled at summarizing code directories."
                .to_string();
        let prompt_user = self.build_summary_prompt(dir_info, files, 1, 1);

        let cache_scope = "directory_summary";
        let log_tag = dir_info.path.to_string_lossy().to_string();

        let response: DirectorySummaryResponse = extract(
            context,
            AgentExecuteParams {
                prompt_sys,
                prompt_user,
                cache_scope: cache_scope.to_string(),
                log_tag,
                progress,
            },
        )
        .await?;

        Ok(DirectoryDossier {
            path: dir_info.path.clone(),
            name: dir_info.name.clone(),
            purpose: classify_directory_purpose(&dir_info.name),
            file_count: dir_info.file_count,
            subdirectory_count: dir_info.subdirectory_count,
            importance_score: response.importance_score.clamp(0.0, 1.0),
            summary: response.summary,
            key_files: response.key_files,
            file_insights: response
                .file_insights
                .into_iter()
                .map(|fi| {
                    let name = fi.name.clone();
                    let path = files
                        .iter()
                        .find(|f| f.name == name)
                        .map(|f| f.path.clone())
                        .unwrap_or_default();
                    crate::types::FileInsight {
                        name,
                        file_path: path,
                        summary: fi.summary.clone(),
                        code_purpose: fi.code_purpose.unwrap_or(crate::types::code::CodePurpose::Other),
                        source_summary: fi.source_summary.clone(),
                        detailed_description: fi.detailed_description.clone(),
                        responsibilities: fi.responsibilities.clone(),
                        interfaces: fi.interfaces.iter().map(|i| crate::types::code::InterfaceInfo {
                            name: i.name.clone(),
                            interface_type: i.interface_type.clone(),
                            visibility: String::new(),
                            parameters: i.parameters.iter().map(|p| crate::types::code::ParameterInfo {
                                name: p.name.clone(),
                                param_type: p.param_type.clone(),
                                is_optional: false,
                                description: None,
                            }).collect(),
                            return_type: if i.return_type.is_empty() { None } else { Some(i.return_type.clone()) },
                            description: None,
                        }).collect(),
                        dependencies: fi.dependencies.iter().map(|d| crate::types::code::Dependency {
                            name: d.name.clone(),
                            path: None,
                            is_external: d.is_external,
                            line_number: None,
                            dependency_type: d.dependency_type.clone(),
                            version: None,
                        }).collect(),
                        importance_score: fi.importance_score,
                    }
                })
                .collect(),
        })
    }

    /// Summarize a batch of files and merge into a single response.
    /// Used when total content exceeds 256KB and needs to be split.
    pub async fn summarize_batch(
        &self,
        context: &GeneratorContext,
        dir_info: &crate::types::DirectoryInfo,
        batches: &[Vec<FileContent>],
        progress: Option<(usize, usize)>,
    ) -> Result<DirectoryDossier> {
        let mut all_key_files = Vec::new();
        let mut all_summaries = Vec::new();
        let mut all_file_insights: Vec<crate::types::FileInsight> = Vec::new();
        let mut total_importance: f64 = 0.0;

        for (batch_idx, batch) in batches.iter().enumerate() {
            let prompt_sys =
                "You are a professional software architecture analyst skilled at summarizing code directories."
                    .to_string();
            let prompt_user = self.build_summary_prompt(dir_info, batch, batch_idx + 1, batches.len());

            let response: DirectorySummaryResponse = extract(
                context,
                AgentExecuteParams {
                    prompt_sys,
                    prompt_user,
                    cache_scope: "directory_summary".to_string(),
                    log_tag: dir_info.path.to_string_lossy().to_string(),
                    progress,
                },
            )
            .await?;

            total_importance += response.importance_score;
            if !response.summary.is_empty() {
                all_summaries.push(response.summary);
            }
            all_key_files.extend(response.key_files);

            // Collect per-file insights
            for fi in response.file_insights {
                let file_path = batch
                    .iter()
                    .find(|f| f.name == fi.name)
                    .map(|f| f.path.clone())
                    .unwrap_or_default();
                all_file_insights.push(crate::types::FileInsight {
                    name: fi.name.clone(),
                    file_path,
                    summary: fi.summary.clone(),
                    code_purpose: fi.code_purpose.unwrap_or(crate::types::code::CodePurpose::Other),
                    source_summary: fi.source_summary.clone(),
                    detailed_description: fi.detailed_description.clone(),
                    responsibilities: fi.responsibilities.clone(),
                    interfaces: fi.interfaces.iter().map(|i| crate::types::code::InterfaceInfo {
                        name: i.name.clone(),
                        interface_type: i.interface_type.clone(),
                        visibility: String::new(),
                        parameters: i.parameters.iter().map(|p| crate::types::code::ParameterInfo {
                            name: p.name.clone(),
                            param_type: p.param_type.clone(),
                            is_optional: false,
                            description: None,
                        }).collect(),
                        return_type: if i.return_type.is_empty() { None } else { Some(i.return_type.clone()) },
                        description: None,
                    }).collect(),
                    dependencies: fi.dependencies.iter().map(|d| crate::types::code::Dependency {
                        name: d.name.clone(),
                        path: None,
                        is_external: d.is_external,
                        line_number: None,
                        dependency_type: d.dependency_type.clone(),
                        version: None,
                    }).collect(),
                    importance_score: fi.importance_score,
                });
            }
        }

        // Deduplicate key files (first occurrence wins)
        let mut seen = std::collections::HashSet::new();
        let key_files: Vec<String> = all_key_files
            .into_iter()
            .filter(|f| seen.insert(f.clone()))
            .take(5)
            .collect();

        let importance_score = if !batches.is_empty() {
            total_importance / batches.len() as f64
        } else {
            0.0
        };

        let summary = if all_summaries.len() > 1 {
            format!(
                "This directory contains multiple file groups. Key aspects: {}",
                all_summaries.join(" ")
            )
        } else {
            all_summaries.into_iter().next().unwrap_or_default()
        };

        Ok(DirectoryDossier {
            path: dir_info.path.clone(),
            name: dir_info.name.clone(),
            purpose: classify_directory_purpose(&dir_info.name),
            file_count: dir_info.file_count,
            subdirectory_count: dir_info.subdirectory_count,
            importance_score: importance_score.clamp(0.0, 1.0),
            summary,
            key_files,
            file_insights: all_file_insights,
        })
    }

    fn build_summary_prompt(
        &self,
        dir_info: &crate::types::DirectoryInfo,
        files: &[FileContent],
        batch_idx: usize,
        total_batches: usize,
    ) -> String {
        // Sort files lexicographically for cache-friendly batching
        let mut sorted_files = files.to_vec();
        sorted_files.sort_by(|a, b| a.name.cmp(&b.name));

        let file_list: String = sorted_files
            .iter()
            .map(|f| {
                let truncated = if f.content.chars().count() > 500 {
                    format!("{}...", f.content.chars().take(500).collect::<String>())
                } else {
                    f.content.clone()
                };

                // Pre-extract interfaces and dependencies via language processor
                let path = &f.path;
                let interfaces = self
                    .language_processor
                    .extract_interfaces(path, &f.content);
                let dependencies = self
                    .language_processor
                    .extract_dependencies(path, &f.content);
                let complexity = self
                    .language_processor
                    .calculate_complexity_metrics(&f.content);

                let interfaces_str = if interfaces.is_empty() {
                    String::new()
                } else {
                    interfaces
                        .iter()
                        .take(20)
                        .map(|i| {
                            let params = i
                                .parameters
                                .iter()
                                .map(|p| format!("{}: {}", p.name, p.param_type))
                                .collect::<Vec<_>>()
                                .join(", ");
                            let ret = i
                                .return_type
                                .as_ref()
                                .map(|r| format!(" -> {}", r))
                                .unwrap_or_default();
                            format!("  - {}: {}({}){}", i.name, i.interface_type, params, ret)
                        })
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let dependencies_str = if dependencies.is_empty() {
                    String::new()
                } else {
                    dependencies
                        .iter()
                        .take(15)
                        .map(|d| format!("  - {} ({})", d.name, d.dependency_type))
                        .collect::<Vec<_>>()
                        .join("\n")
                };

                let metrics_str = format!(
                    "  lines: {}, functions: {}, classes: {}, complexity: {:.1}",
                    complexity.lines_of_code,
                    complexity.number_of_functions,
                    complexity.number_of_classes,
                    complexity.cyclomatic_complexity
                );

                let block = if interfaces_str.is_empty() && dependencies_str.is_empty() {
                    format!("=== {} ===\n{}\n", f.name, truncated)
                } else {
                    format!(
                        "=== {} ===\nMetrics:\n{}\n\nInterfaces:\n{}\n\nDependencies:\n{}\n\nSource (truncated to 500 chars):\n{}\n",
                        f.name,
                        metrics_str,
                        if interfaces_str.is_empty() { "  (none)".to_string() } else { interfaces_str },
                        if dependencies_str.is_empty() { "  (none)".to_string() } else { dependencies_str },
                        truncated
                    )
                };
                block
            })
            .collect::<Vec<_>>()
            .join("\n");

        let batch_note = if total_batches > 1 {
            format!(
                " (batch {}/{}, processing a subset of files due to size)",
                batch_idx, total_batches
            )
        } else {
            String::new()
        };

        format!(
            r#"Analyze the directory "{}"{} and generate directory-level and per-file insights.

Directory info:
- Name: {}
- Files in this directory: {}
- Subdirectories: {}
- This is batch {}/{} of {} total batches (files are sorted lexicographically)

Files content:
{}

Rate the importance of this directory based on:
1. Business value - does it contain core business logic, APIs, or data layer?
2. Code concentration - is it a hub with many imports/exports?
3. Infrastructure role - is it a core package, main entry, or config layer?
IMPORTANT: Backend directories (*.py, *.go, *.rs, *.java, *.kt, etc.) should be rated higher than frontend directories (*.ts, *.js, *.tsx, *.vue, *.jsx, etc.) when business value is comparable.

Output JSON with:
- "summary": 2-3 sentence description of this directory's role and how the files work together
- "importance_score": directory importance score (0.0-1.0), higher = more important to the project
- "key_files": names of the up-to-5 most important files in this directory
- "file_insights": array of per-file insights, each with:
  - "name": file name
  - "summary": 1-2 sentence description of what this file does
  - "code_purpose": one of Entry, Agent, Page, Widget, SpecificFeature, Model, Types, Tool, Util, Config, Middleware, Plugin, Router, Database, Api, Controller, Service, Module, Lib, Test, Doc, Dao, Context (infer from file extension and content)
  - "importance_score": file importance score (0.0-1.0)
  - "detailed_description": 2-3 sentence detailed description of this file's role
  - "source_summary": 2-3 sentence summary of the file's source code content (for database files: describe schema/tables; for code files: describe main functions/purposes)
  - "responsibilities": array of 2-5 key responsibilities this file handles
  - "interfaces": array of key functions/methods exposed by this file, each with name, interface_type, parameters (array of objects with name and param_type fields), return_type
  - "dependencies": array of key dependencies this file imports/uses, each with name, is_external (bool), dependency_type (import|use|include|require)
{{"summary": "...", "importance_score": 0.85, "key_files": ["file1.rs"], "file_insights": [{{"name": "file1.rs", "summary": "Main entry point...", "code_purpose": "Entry", "importance_score": 0.95, "detailed_description": "Entry point.", "source_summary": "Defines main.", "responsibilities": ["init"], "interfaces": [{{"name": "main", "interface_type": "function", "parameters": [], "return_type": "void"}}], "dependencies": [{{"name": "app", "is_external": false, "dependency_type": "import"}}]}}, ...]}}

IMPORTANT: Output valid JSON only, no markdown fences."#,
            dir_info.name,
            batch_note,
            dir_info.name,
            dir_info.name,
            dir_info.file_count,
            dir_info.subdirectory_count,
            batch_idx,
            total_batches,
            file_list
        )
    }
}

fn classify_directory_purpose(name: &str) -> DirectoryPurpose {
    let n = name.to_lowercase();
    if n == "src" || n == "lib" || n == "internal" || n == "core" || n == "pkg" {
        DirectoryPurpose::Core
    } else if n == "cmd" || n == "main" || n == "entry" {
        DirectoryPurpose::Core
    } else if n.contains("config") || n == "conf" || n == "cfg" {
        DirectoryPurpose::Config
    } else if n.contains("api") || n == "rpc" || n == "grpc" {
        DirectoryPurpose::Api
    } else if n.contains("database") || n == "db" || n.contains("migrations") || n.contains("schema")
    {
        DirectoryPurpose::Database
    } else if n == "frontend" || n == "ui" || n == "web" || n == "client" || n == "views" {
        DirectoryPurpose::Frontend
    } else if n == "test" || n == "tests" || n == "spec" || n == "__tests__" {
        DirectoryPurpose::Test
    } else if n == "scripts" || n == "tools" || n == "bin" {
        DirectoryPurpose::Tool
    } else if n == "docs" || n == "doc" || n == "documentation" {
        DirectoryPurpose::Docs
    } else {
        DirectoryPurpose::Other
    }
}
