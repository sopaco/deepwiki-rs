use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::fs;
use std::collections::{HashMap, HashSet};
use chrono::{DateTime, Utc};

use crate::config::{Config, LocalDocsConfig};
use crate::integrations::local_docs::{LocalDocsProcessor, LocalDocMetadata};

/// Metadata about synced knowledge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeMetadata {
    pub last_synced: DateTime<Utc>,
    pub local_docs: Vec<LocalDocMetadata>,
    /// Documents organized by category
    #[serde(default)]
    pub categories: HashMap<String, Vec<LocalDocMetadata>>,
}

/// Syncs external knowledge sources to local cache
pub struct KnowledgeSyncer {
    config: Config,
}

impl KnowledgeSyncer {
    /// Create a new knowledge syncer
    pub fn new(config: Config) -> Result<Self> {
        Ok(Self { config })
    }

    /// Sync all configured knowledge sources
    pub async fn sync_all(&self) -> Result<()> {
        let target_lang = self.config.target_language.display_name();
        println!("🔄 Syncing external knowledge sources (target language: {})...", target_lang);

        let mut synced_any = false;

        if let Some(ref local_docs_config) = self.config.knowledge.local_docs {
            if local_docs_config.enabled {
                self.sync_local_docs(local_docs_config).await?;
                synced_any = true;
            } else {
                println!("ℹ️  Local docs integration is disabled");
            }
        }

        if !synced_any {
            println!("ℹ️  No knowledge sources are configured");
        }

        println!("✅ Knowledge sync completed");
        Ok(())
    }

    /// Sync local documentation files
    async fn sync_local_docs(&self, config: &LocalDocsConfig) -> Result<()> {
        println!("\n📄 Processing local documentation files...");

        let cache_dir = config
            .cache_dir
            .clone()
            .unwrap_or_else(|| {
                self.config
                    .internal_path
                    .join("knowledge")
                    .join("local_docs")
            });

        fs::create_dir_all(&cache_dir).context("Failed to create local docs cache directory")?;

        let mut all_docs = Vec::new();
        let mut categories_map: HashMap<String, Vec<LocalDocMetadata>> = HashMap::new();
        let mut processed_count = 0;
        let mut chunked_count = 0;

        // Get default chunking config
        let default_chunking = config.default_chunking.clone();
        let project_root = self.config.project_path.as_path();

        // Process categorized documents
        for category in &config.categories {
            println!("\n  📁 Processing category: {} ({})", category.name, category.description);
            
            let files = LocalDocsProcessor::expand_glob_patterns(&category.paths, Some(project_root));
            
            // Determine chunking config for this category
            let chunking_config = category.chunking.as_ref().or(default_chunking.as_ref());
            
            for file_path in files {
                match LocalDocsProcessor::process_file_with_chunking(
                    &file_path,
                    &category.name,
                    &category.target_agents,
                    chunking_config,
                ) {
                    Ok(doc_metas) => {
                        let is_chunked = doc_metas.len() > 1;
                        if is_chunked {
                            println!("    ✓ [{}] {} (chunked into {} parts)", 
                                category.name, file_path.display(), doc_metas.len());
                            chunked_count += 1;
                        } else {
                            println!("    ✓ [{}] {}", category.name, file_path.display());
                        }
                        
                        for doc_meta in doc_metas {
                            // Add to category-specific map
                            categories_map
                                .entry(category.name.clone())
                                .or_default()
                                .push(doc_meta.clone());
                            
                            // Also add to all_docs for combined access
                            all_docs.push(doc_meta);
                        }
                        processed_count += 1;
                    }
                    Err(e) => {
                        eprintln!("    ✗ Failed to process {}: {}", file_path.display(), e);
                    }
                }
            }
        }

        // Save metadata
        let metadata = KnowledgeMetadata {
            last_synced: Utc::now(),
            local_docs: all_docs,
            categories: categories_map,
        };

        let metadata_file = cache_dir.join("_metadata.json");
        let metadata_json =
            serde_json::to_string_pretty(&metadata).context("Failed to serialize metadata")?;
        fs::write(&metadata_file, metadata_json).context("Failed to write metadata")?;

        if chunked_count > 0 {
            println!("✅ Processed {} files ({} chunked into multiple parts)", processed_count, chunked_count);
        } else {
            println!("✅ Processed {} local documentation files", processed_count);
        }
        Ok(())
    }

    /// Check if knowledge needs to be re-synced
    pub fn should_sync(&self) -> Result<bool> {
        // Check if local docs need syncing
        if let Some(ref local_docs_config) = self.config.knowledge.local_docs {
            if !local_docs_config.enabled {
                return Ok(false);
            }

            let cache_dir = local_docs_config
                .cache_dir
                .clone()
                .unwrap_or_else(|| {
                    self.config
                        .internal_path
                        .join("knowledge")
                        .join("local_docs")
                });

            let metadata_file = cache_dir.join("_metadata.json");

            // Always sync local docs if cache doesn't exist or if watch_for_changes is true
            if !metadata_file.exists() {
                return Ok(true);
            }

            if local_docs_config.watch_for_changes {
                // Check if any source file has been modified since last sync
                let metadata_content = fs::read_to_string(&metadata_file)?;
                let metadata: KnowledgeMetadata = serde_json::from_str(&metadata_content)?;

                let mut cached_files: HashSet<PathBuf> = HashSet::new();
                for doc in &metadata.local_docs {
                    let cached_path = Path::new(&doc.file_path);
                    cached_files.insert(Self::normalize_path(cached_path));
                }

                let mut current_files: HashSet<PathBuf> = HashSet::new();
                let project_root = self.config.project_path.as_path();
                for category in &local_docs_config.categories {
                    let files = LocalDocsProcessor::expand_glob_patterns(&category.paths, Some(project_root));
                    for file_path in files {
                        current_files.insert(Self::normalize_path(&file_path));
                    }
                }

                // Detect new or removed files quickly
                if current_files.symmetric_difference(&cached_files).next().is_some() {
                    return Ok(true);
                }
                
                
                
                
                // Check if any source file has been modified
                for doc in &metadata.local_docs {
                    let source_path = PathBuf::from(&doc.file_path);
                    if source_path.exists() {
                        if let Ok(file_metadata) = fs::metadata(&source_path) {
                            if let Ok(modified) = file_metadata.modified() {
                                // Convert SystemTime to DateTime<Utc>
                                let modified_datetime: DateTime<Utc> = modified.into();
                                // Compare with cached modification time
                                if modified_datetime > metadata.last_synced {
                                    return Ok(true);
                                }
                            }
                        }
                    }
                }
                return Ok(false);
            }
        }

        Ok(false)
    }
    
    fn normalize_path(path: &Path) -> PathBuf {
        fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
    }
    
    /// Load cached knowledge for a specific category
    pub fn load_cached_knowledge_by_category(
        &self,
        category: &str,
        agent_filter: Option<&str>,
    ) -> Result<Option<String>> {
        let local_docs_config = match &self.config.knowledge.local_docs {
            Some(cfg) if cfg.enabled => cfg,
            _ => return Ok(None),
        };

        let cache_dir = local_docs_config
            .cache_dir
            .clone()
            .unwrap_or_else(|| {
                self.config
                    .internal_path
                    .join("knowledge")
                    .join("local_docs")
            });

        let metadata_file = cache_dir.join("_metadata.json");
        if !metadata_file.exists() {
            return Ok(None);
        }

        let metadata_content = fs::read_to_string(&metadata_file)?;
        let metadata: KnowledgeMetadata = serde_json::from_str(&metadata_content)?;

        // Get documents for the specified category
        let Some(docs) = metadata.categories.get(category) else {
            return Ok(None);
        };

        let filtered_docs: Vec<LocalDocMetadata> = docs
            .iter()
            .cloned()
            .filter(|doc| Self::doc_visible_to_agent(doc, agent_filter))
            .collect();

        if filtered_docs.is_empty() {
            return Ok(None);
        }

        let target_lang = self.config.target_language.display_name();
        let header = format!(
            "# {} Documentation ({})\n\nCategory: {}\nLast processed: {}\nDocuments in category: {}\n\n",
            Self::format_category_name(category),
            target_lang,
            category,
            metadata.last_synced.format("%Y-%m-%d %H:%M:%S UTC"),
            filtered_docs.len()
        );

        let formatted = LocalDocsProcessor::format_for_llm_with_options(
            &filtered_docs,
            Some(&header),
            false,
        );

        Ok(Some(formatted))
    }
    
    /// Format category name for display
    fn format_category_name(category: &str) -> String {
        match category {
            "architecture" => "Architecture".to_string(),
            "database" => "Database".to_string(),
            "deployment" => "Deployment & Infrastructure".to_string(),
            "api" => "API".to_string(),
            "adr" => "Architecture Decision Records".to_string(),
            "workflow" => "Workflow & Business Process".to_string(),
            "general" => "General".to_string(),
            other => other.chars().next().map(|c| c.to_uppercase().to_string()).unwrap_or_default() 
                + &other.chars().skip(1).collect::<String>(),
        }
    }

    fn doc_visible_to_agent(doc: &LocalDocMetadata, agent_filter: Option<&str>) -> bool {
        match agent_filter {
            None => true,
            Some(agent) => {
                if doc.target_agents.is_empty() {
                    true
                } else {
                    doc.target_agents.iter().any(|configured| configured == agent)
                }
            }
        }
    }
}
