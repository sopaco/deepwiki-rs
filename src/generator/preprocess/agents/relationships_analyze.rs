use anyhow::Result;

use crate::generator::agent_executor::{AgentExecuteParams, extract};
use crate::generator::preprocess::memory::ScopedKeys;
use crate::types::code_releationship::RelationshipAnalysis;
use crate::{
    generator::context::GeneratorContext,
    types::{DirectoryDossier, DirectorySelection},
    utils::prompt_compressor::{CompressionConfig, PromptCompressor},
};

pub struct RelationshipsAnalyze {
    prompt_compressor: PromptCompressor,
}

impl RelationshipsAnalyze {
    pub fn new() -> Self {
        Self {
            prompt_compressor: PromptCompressor::new(CompressionConfig::default()),
        }
    }

    /// Execute relationship analysis using directory dossiers.
    /// Two-phase when index content exceeds MAX_INDEX_CHARS:
    ///   Phase 1 — selection: LLM picks important directories + files
    ///   Phase 2 — analysis: LLM generates relationship graph from selected subset
    pub async fn execute(
        &self,
        context: &GeneratorContext,
        directory_dossiers: &[DirectoryDossier],
    ) -> Result<RelationshipAnalysis> {
        // Build index (metadata only, no per-file details)
        let index_content = self.build_index_content(directory_dossiers);
        let index_size = index_content.len();
        let index_threshold = context.config.max_file_size as usize;

        // Check if we need two-phase approach
        if index_size > index_threshold {
            // Phase 1: LLM selection
            let selection = self
                .select_directories_and_files(context, directory_dossiers, &index_content)
                .await?;

            // Cache selection for reuse by other agents
            context
                .store_to_memory(
                    crate::generator::preprocess::memory::MemoryScope::PREPROCESS,
                    ScopedKeys::DIRECTORY_SELECTION,
                    &selection,
                )
                .await?;

            // Phase 2: analysis with selected subset
            let agent_params = self
                .build_analysis_params_with_selection(context, directory_dossiers, &selection)
                .await?;
            extract::<RelationshipAnalysis>(context, agent_params).await
        } else {
            // Small enough: single-phase analysis
            let agent_params = self
                .build_analysis_params(context, directory_dossiers)
                .await?;
            extract::<RelationshipAnalysis>(context, agent_params).await
        }
    }

    /// Build lightweight index: directory path, purpose, score, summary, and per-file names + scores.
    fn build_index_content(&self, dossiers: &[DirectoryDossier]) -> String {
        dossiers
            .iter()
            .map(|d| {
                let files_summary = d
                    .file_insights
                    .iter()
                    .map(|fi| format!("  - {} (score: {:.2}, purpose: {:?})", fi.name, fi.importance_score, fi.code_purpose))
                    .collect::<Vec<_>>()
                    .join("\n");

                format!(
                    "### {} | path: {} | purpose: {:?} | importance: {:.2}\nSummary: {}\nFiles:\n{}",
                    d.name,
                    d.path.to_string_lossy(),
                    d.purpose,
                    d.importance_score,
                    d.summary,
                    files_summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    /// Phase 1: ask LLM which directories and files are architecturally significant.
    async fn select_directories_and_files(
        &self,
        context: &GeneratorContext,
        _directory_dossiers: &[DirectoryDossier],
        index_content: &str,
    ) -> Result<DirectorySelection> {
        let prompt_sys = r#"You are a software architecture analyst selecting key directories and files for relationship analysis.

You MUST return valid JSON only (no markdown, no code fences):
{
  "selected_directories": ["path1", "path2", ...],
  "selected_files": [
    {"dir_path": "path1", "file_names": ["file1.rs", "file2.rs"]},
    {"dir_path": "path2", "file_names": ["file3.rs"]}
  ]
}

Rules:
- Select directories that represent distinct architectural concerns (apis, core, models, services, etc.)
- Prefer directories with high architectural significance over generic utility dirs
- For each selected directory, pick the 3-5 most important files (highest score or most central to the architecture)
- Limit to 20 directories maximum; 5-10 is preferred
- Use relative paths matching exactly those in the index"#
            .to_string();

        let compression_result = self
            .prompt_compressor
            .compress_if_needed(context, index_content, "Directory Selection Index")
            .await?;

        if compression_result.was_compressed {
            println!(
                "   ✅ Selection index compressed: {} -> {} tokens",
                compression_result.original_tokens, compression_result.compressed_tokens
            );
        }

        let prompt_user = format!(
            r#"From the directory index below, select the most architecturally significant directories and files for a relationship graph analysis.

## Directory Index
{}

Output JSON selecting the key directories and per-directory file selection."#,
            compression_result.compressed_content
        );

        let agent_params = AgentExecuteParams {
            prompt_sys,
            prompt_user,
            cache_scope: "directory_selection".to_string(),
            log_tag: "Directory Selection".to_string(),
            progress: None,
        };

        extract::<DirectorySelection>(context, agent_params).await
    }

    async fn build_analysis_params(
        &self,
        context: &GeneratorContext,
        directory_dossiers: &[DirectoryDossier],
    ) -> Result<AgentExecuteParams> {
        let prompt_sys = r#"You are a professional software architecture analyst.

You MUST return valid JSON only (no markdown, no code fences, no prose before/after JSON).
The JSON MUST match this exact schema and field names:
{
  "core_dependencies": [
    {
      "from": "string",
      "to": "string",
      "dependency_type": "Import|FunctionCall|Inheritance|Composition|DataFlow|Module",
      "importance": 1,
      "description": "string (optional)"
    }
  ],
  "architecture_layers": [
    {
      "name": "string",
      "components": ["string"],
      "level": 1
    }
  ],
  "key_insights": ["string"]
}

Constraints:
- Never omit top-level keys. Always include all three arrays.
- Use plain strings for textual fields; never objects/arrays for those fields.
- Use integer values for "importance" and "level".
- Keep values concise and architecture-focused.
"#
            .to_string();

        let dossiers_content = self.build_dossiers_content(directory_dossiers);

        let compression_result = self
            .prompt_compressor
            .compress_if_needed(context, &dossiers_content, "Directory Dossiers")
            .await?;

        if compression_result.was_compressed {
            println!(
                "   ✅ Compression complete: {} -> {} tokens",
                compression_result.original_tokens, compression_result.compressed_tokens
            );
        }
        let compressed_content = compression_result.compressed_content;

        let prompt_user = format!(
            r#"Analyze the overall architectural relationship graph of this project based on the directory dossiers below.

Output requirements (strict):
- Return JSON only.
- Do not use markdown code blocks.
- Do not include explanations outside JSON.
- Use exactly the allowed enum labels: Import, FunctionCall, Inheritance, Composition, DataFlow, Module.
- If uncertain, use Module as dependency_type.

## Directory Dossiers
{}

## Analysis Requirements:
Generate a project-level dependency relationship graph, focusing on:
1. Cross-directory module dependencies and data flows
2. Architectural hierarchy (which directories are core, which are peripheral)
3. Key integration points between directories
4. Potential architectural issues or circular dependencies"#,
            compressed_content
        );

        Ok(AgentExecuteParams {
            prompt_sys,
            prompt_user,
            cache_scope: "ai_relationships_insights".to_string(),
            log_tag: "Dependency Relationship Analysis".to_string(),
            progress: None,
        })
    }

    /// Build analysis params using LLM-provided selection to filter dossiers.
    async fn build_analysis_params_with_selection(
        &self,
        context: &GeneratorContext,
        directory_dossiers: &[DirectoryDossier],
        selection: &DirectorySelection,
    ) -> Result<AgentExecuteParams> {
        let prompt_sys = r#"You are a professional software architecture analyst.

You MUST return valid JSON only (no markdown, no code fences, no prose before/after JSON).
The JSON MUST match this exact schema and field names:
{
  "core_dependencies": [
    {
      "from": "string",
      "to": "string",
      "dependency_type": "Import|FunctionCall|Inheritance|Composition|DataFlow|Module",
      "importance": 1,
      "description": "string (optional)"
    }
  ],
  "architecture_layers": [
    {
      "name": "string",
      "components": ["string"],
      "level": 1
    }
  ],
  "key_insights": ["string"]
}

Constraints:
- Never omit top-level keys. Always include all three arrays.
- Use plain strings for textual fields; never objects/arrays for those fields.
- Use integer values for "importance" and "level".
- Keep values concise and architecture-focused.
"#
            .to_string();

        // Build selected subset of dossiers + file_insights
        let selected_dir_set: std::collections::HashSet<_> =
            selection.selected_directories.iter().collect();
        let selected_files_map: std::collections::HashMap<_, _> = selection
            .selected_files
            .iter()
            .map(|sf| (&sf.dir_path, &sf.file_names))
            .collect();

        let filtered: Vec<_> = directory_dossiers
            .iter()
            .filter(|d| selected_dir_set.contains(&d.path))
            .map(|dossier| {
                let file_names = selected_files_map.get(&dossier.path);
                let file_insights: Vec<_> = dossier
                    .file_insights
                    .iter()
                    .filter(|fi| {
                        file_names
                            .map(|names| names.contains(&fi.name))
                            .unwrap_or(false)
                    })
                    .collect();
                let files_str = file_insights
                    .iter()
                    .map(|fi| format!("  - {}: {}", fi.name, fi.summary))
                    .collect::<Vec<_>>()
                    .join("\n");
                format!(
                    "### {} (purpose: {:?}, importance: {:.2})\nDirectory summary: {}\nPer-file insights:\n{}",
                    dossier.name,
                    dossier.purpose,
                    dossier.importance_score,
                    dossier.summary,
                    files_str
                )
            })
            .collect();

        let dossiers_content = filtered.join("\n\n");

        let compression_result = self
            .prompt_compressor
            .compress_if_needed(context, &dossiers_content, "Directory Dossiers")
            .await?;

        if compression_result.was_compressed {
            println!(
                "   ✅ Compression (selected): {} -> {} tokens",
                compression_result.original_tokens, compression_result.compressed_tokens
            );
        }
        let compressed_content = compression_result.compressed_content;

        let prompt_user = format!(
            r#"Analyze the overall architectural relationship graph of this project based on the LLM-selected directories and files below.

Output requirements (strict):
- Return JSON only.
- Do not use markdown code blocks.
- Do not include explanations outside JSON.
- Use exactly the allowed enum labels: Import, FunctionCall, Inheritance, Composition, DataFlow, Module.
- If uncertain, use Module as dependency_type.

## Selected Directory Dossiers
{}

## Analysis Requirements:
Generate a project-level dependency relationship graph, focusing on:
1. Cross-directory module dependencies and data flows
2. Architectural hierarchy (which directories are core, which are peripheral)
3. Key integration points between directories
4. Potential architectural issues or circular dependencies"#,
            compressed_content
        );

        Ok(AgentExecuteParams {
            prompt_sys,
            prompt_user,
            cache_scope: "ai_relationships_insights_selected".to_string(),
            log_tag: "Dependency Relationship Analysis (selected)".to_string(),
            progress: None,
        })
    }

    fn build_dossiers_content(&self, dossiers: &[DirectoryDossier]) -> String {
        let filtered: Vec<_> = dossiers
            .iter()
            .filter(|d| d.importance_score >= 0.5 || !d.key_files.is_empty())
            .collect();
        filtered
            .into_iter()
            .map(|dossier| {
                let file_insights = dossier
                    .file_insights
                    .iter()
                    .map(|fi| format!("  - {}: {}", fi.name, fi.summary))
                    .collect::<Vec<_>>()
                    .join("\n");

                let key_files_str = if dossier.key_files.is_empty() {
                    String::new()
                } else {
                    format!(", key_files: {:?}", dossier.key_files)
                };

                format!(
                    "### {} (purpose: {:?}, importance: {:.2}{})\nDirectory summary: {}\nPer-file insights:\n{}",
                    dossier.name,
                    dossier.purpose,
                    dossier.importance_score,
                    key_files_str,
                    dossier.summary,
                    file_insights
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    }
}
