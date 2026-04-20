use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::time::Instant;

use crate::generator::preprocess::extractors::original_document_extractor;
use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::types::original_document::OriginalDocument;
use crate::{
    generator::{
        context::GeneratorContext,
        preprocess::extractors::structure_extractor::StructureExtractor,
        types::Generator,
    },
    types::{
        project_structure::ProjectStructure, CodeAndDirectoryInsights, DirectoryDossier,
        DirectoryPurpose,
    },
};

pub mod agents;
pub mod extractors;
pub mod memory;

use crate::generator::preprocess::agents::directory_summary::FileContent;
use crate::generator::preprocess::agents::relationships_analyze::RelationshipsAnalyze;

/// Preprocessing result — simplified to directory-only insights
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PreprocessingResult {
    pub original_document: OriginalDocument,
    pub project_structure: ProjectStructure,
    pub directory_dossiers: Vec<DirectoryDossier>,
    pub processing_time: f64,
}

pub struct PreProcessAgent {}

impl PreProcessAgent {
    pub fn new() -> Self {
        Self {}
    }
}

impl Generator<PreprocessingResult> for PreProcessAgent {
    async fn execute(&self, context: GeneratorContext) -> Result<PreprocessingResult> {
        let start_time = Instant::now();

        let structure_extractor = StructureExtractor::new(context.clone());
        let config = &context.config;

        println!("🔍 Starting project preprocessing phase...");

        // 1. Extract project original document materials
        println!("📁 Extracting project original document materials...");
        let original_document = original_document_extractor::extract(&context).await?;

        // 2. Extract project structure (includes all files and directories)
        println!("📁 Extracting project structure...");
        let project_structure = structure_extractor
            .extract_structure(&config.project_path)
            .await?;

        println!(
            "   🔭 Discovered {} files, {} directories",
            project_structure.total_files, project_structure.total_directories
        );

        // 3. Generate directory dossiers with LLM (reads files directly, no top-N filtering)
        println!("📂 Generating directory dossiers with LLM...");
        let directory_dossiers =
            generate_directory_dossiers(&context, &project_structure).await?;

        // 4. Generate relationship analysis based on directory dossiers
        println!("🔗 Generating relationship analysis...");
        let relationships_analyzer = RelationshipsAnalyze::new();
        let relationships = relationships_analyzer
            .execute(&context, &directory_dossiers)
            .await
            .unwrap_or_else(|e| {
                eprintln!("⚠️  Failed to generate relationships: {}, continuing without", e);
                crate::types::code_releationship::RelationshipAnalysis::default()
            });

        let processing_time = start_time.elapsed().as_secs_f64();

        println!(
            "✅ Project preprocessing completed, {} directories analyzed, took {:.2}s",
            directory_dossiers.len(),
            processing_time
        );

        // 4. Store results to Memory
        context
            .store_to_memory(
                MemoryScope::PREPROCESS,
                ScopedKeys::PROJECT_STRUCTURE,
                &project_structure,
            )
            .await?;
        context
            .store_to_memory(
                MemoryScope::PREPROCESS,
                ScopedKeys::CODE_INSIGHTS,
                &CodeAndDirectoryInsights {
                    file_insights: Vec::new(),
                    directory_insights: directory_dossiers.clone(),
                },
            )
            .await?;
        context
            .store_to_memory(
                MemoryScope::PREPROCESS,
                ScopedKeys::ORIGINAL_DOCUMENT,
                &original_document,
            )
            .await?;
        context
            .store_to_memory(
                MemoryScope::PREPROCESS,
                ScopedKeys::RELATIONSHIPS,
                &relationships,
            )
            .await?;

        Ok(PreprocessingResult {
            original_document,
            project_structure,
            directory_dossiers,
            processing_time,
        })
    }
}

/// Generate directory dossiers by reading files directly from disk.
/// Each directory's files are batched: if total content exceeds 256KB, split into
/// batches (sorted lexicographically for cache-friendly ordering) and merge results.
const MAX_BATCH_SIZE: usize = 256 * 1024;

async fn generate_directory_dossiers(
    context: &GeneratorContext,
    project_structure: &ProjectStructure,
) -> Result<Vec<DirectoryDossier>> {
    use crate::generator::preprocess::agents::directory_summary::DirectorySummarizer;

    let summarizer = DirectorySummarizer::new();
    let config = &context.config;
    let mut dossiers = Vec::new();
    let total_dirs = project_structure.directories.len();

    for (idx, dir) in project_structure.directories.iter().enumerate() {
        // Read all files in this directory from disk
        let mut files = read_directory_files(&dir.path, config)?;

        if files.is_empty() {
            continue;
        }

        // Sort lexicographically for cache-friendly batching
        files.sort_by(|a, b| a.name.cmp(&b.name));

        // Calculate total size
        let total_size: usize = files.iter().map(|f| f.content.len()).sum();

        if total_size <= MAX_BATCH_SIZE {
            // Single batch
            match summarizer
                .summarize_directory(context, dir, &files, Some((idx + 1, total_dirs)))
                .await
            {
                Ok(dossier) => dossiers.push(dossier),
                Err(e) => {
                    eprintln!(
                        "⚠️  Failed to summarize directory {}: {}, using fallback",
                        dir.name, e
                    );
                    dossiers.push(fallback_dossier(dir));
                }
            }
        } else {
            // Multiple batches: split by file boundaries, keep lexicographic order within each batch
            let batches = split_into_batches(&files, MAX_BATCH_SIZE);
            if batches.len() == 1 {
                // Edge case: single file exceeds 256KB
                match summarizer
                    .summarize_directory(context, dir, &files, Some((idx + 1, total_dirs)))
                    .await
                {
                    Ok(dossier) => dossiers.push(dossier),
                    Err(e) => {
                        eprintln!(
                            "⚠️  Failed to summarize directory {}: {}, using fallback",
                            dir.name, e
                        );
                        dossiers.push(fallback_dossier(dir));
                    }
                }
            } else {
                match summarizer
                    .summarize_batch(context, dir, &batches, Some((idx + 1, total_dirs)))
                    .await
                {
                    Ok(dossier) => dossiers.push(dossier),
                    Err(e) => {
                        eprintln!(
                            "⚠️  Failed to summarize directory {} (batch mode): {}, using fallback",
                            dir.name, e
                        );
                        dossiers.push(fallback_dossier(dir));
                    }
                }
            }
        }
    }

    Ok(dossiers)
}

/// Read all files in a directory, respecting config exclusions and max_file_size.
fn read_directory_files(
    dir_path: &std::path::PathBuf,
    config: &crate::config::Config,
) -> Result<Vec<FileContent>> {
    use crate::utils::file_utils::is_binary_file_path;

    let mut files = Vec::new();

    if let Ok(entries) = std::fs::read_dir(dir_path) {
        for entry in entries.filter_map(|e| e.ok()) {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            // Skip binary files
            if is_binary_file_path(&path) {
                continue;
            }

            // Get file name for exclusion checks
            let file_name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_lowercase();

            // Skip excluded files (wildcard and exact match)
            let should_skip_file = config.excluded_files.iter().any(|excluded| {
                if excluded.contains('*') {
                    let pattern = excluded.replace('*', "").to_lowercase();
                    file_name.contains(&pattern)
                } else {
                    file_name == excluded.to_lowercase()
                }
            });
            if should_skip_file {
                continue;
            }

            // Skip excluded extensions
            if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
                if config.excluded_extensions.contains(&ext.to_lowercase()) {
                    continue;
                }
            }

            if let Ok(metadata) = std::fs::metadata(&path) {
                let size = metadata.len() as usize;
                // Skip files larger than max_file_size
                if size > config.max_file_size as usize {
                    continue;
                }
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let name = path
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy()
                        .to_string();
                    // Truncate per-file at 256KB for prompt
                    let truncated = if content.chars().count() > 256 * 1024 {
                        content.chars().take(256 * 1024).collect()
                    } else {
                        content
                    };
                    files.push(FileContent {
                        name,
                        path,
                        content: truncated,
                    });
                }
            }
        }
    }

    Ok(files)
}

/// Split files into batches, each batch's total content <= max_size.
/// Files are kept in lexicographic order within each batch.
fn split_into_batches(files: &[FileContent], max_size: usize) -> Vec<Vec<FileContent>> {
    let mut batches = Vec::new();
    let mut current_batch = Vec::new();
    let mut current_size = 0usize;

    for file in files {
        if current_size + file.content.len() > max_size && !current_batch.is_empty() {
            batches.push(std::mem::take(&mut current_batch));
            current_size = 0;
        }
        current_size += file.content.len();
        current_batch.push(file.clone());
    }

    if !current_batch.is_empty() {
        batches.push(current_batch);
    }

    batches
}

fn fallback_dossier(dir: &crate::types::DirectoryInfo) -> DirectoryDossier {
    DirectoryDossier {
        path: dir.path.clone(),
        name: dir.name.clone(),
        purpose: DirectoryPurpose::Other,
        file_count: dir.file_count,
        subdirectory_count: dir.subdirectory_count,
        importance_score: 0.0,
        summary: String::new(),
        key_files: Vec::new(),
        file_insights: Vec::new(),
    }
}
