use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::fs;
use glob::glob;

use crate::config::ChunkingConfig;

/// Metadata about processed local documentation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalDocMetadata {
    pub file_path: String,
    pub file_type: DocFileType,
    pub last_modified: String,
    pub processed_content: String,
    /// Category this document belongs to (e.g., "architecture", "database", "api")
    #[serde(default)]
    pub category: String,
    /// Agents that should receive this document
    #[serde(default)]
    pub target_agents: Vec<String>,
    /// Chunk information if this is part of a chunked document
    #[serde(default)]
    pub chunk_info: Option<ChunkInfo>,
}

/// Information about a document chunk
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChunkInfo {
    /// Chunk index (0-based)
    pub chunk_index: usize,
    /// Total number of chunks
    pub total_chunks: usize,
    /// Section title or context for this chunk
    pub section_context: String,
}

/// Supported documentation file types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DocFileType {
    Pdf,
    Markdown,
    Text,
    Sql,
    Yaml,
    Json,
}

/// Document chunker for splitting large documents
pub struct DocumentChunker {
    config: ChunkingConfig,
}

impl DocumentChunker {
    pub fn new(config: ChunkingConfig) -> Self {
        Self { config }
    }
    
    /// Check if content needs chunking based on size
    pub fn needs_chunking(&self, content: &str) -> bool {
        self.config.enabled && content.len() >= self.config.min_size_for_chunking
    }
    
    /// Chunk content based on configured strategy
    pub fn chunk_content(&self, content: &str, file_type: &DocFileType) -> Vec<DocumentChunk> {
        if !self.needs_chunking(content) {
            return vec![DocumentChunk {
                content: content.to_string(),
                chunk_index: 0,
                total_chunks: 1,
                section_context: String::new(),
            }];
        }
        
        match self.config.strategy.as_str() {
            "semantic" => self.chunk_semantic(content, file_type),
            "paragraph" => self.chunk_by_paragraph(content),
            "fixed" | _ => self.chunk_fixed_size(content),
        }
    }
    
    /// Semantic chunking - split by sections/headers (best for Markdown)
    fn chunk_semantic(&self, content: &str, file_type: &DocFileType) -> Vec<DocumentChunk> {
        match file_type {
            DocFileType::Markdown => self.chunk_markdown_by_sections(content),
            DocFileType::Sql => self.chunk_sql_by_statements(content),
            DocFileType::Yaml | DocFileType::Json => self.chunk_by_paragraph(content),
            _ => self.chunk_fixed_size(content),
        }
    }
    
    /// Chunk Markdown by headers (## or ###)
    fn chunk_markdown_by_sections(&self, content: &str) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_section = String::new();
        let mut section_stack: Vec<String> = Vec::new();
        
        for line in content.lines() {
            // Detect headers
            if line.starts_with("# ") {
                // H1 - major section boundary
                if !current_chunk.is_empty() {
                    chunks.push(DocumentChunk {
                        content: current_chunk.clone(),
                        chunk_index: chunks.len(),
                        total_chunks: 0, // Will be updated later
                        section_context: current_section.clone(),
                    });
                    current_chunk.clear();
                }
                section_stack.clear();
                section_stack.push(line[2..].trim().to_string());
                current_section = line[2..].trim().to_string();
            } else if line.starts_with("## ") {
                // H2 - check if we should split
                if current_chunk.len() >= self.config.max_chunk_size {
                    chunks.push(DocumentChunk {
                        content: current_chunk.clone(),
                        chunk_index: chunks.len(),
                        total_chunks: 0,
                        section_context: current_section.clone(),
                    });
                    current_chunk.clear();
                }
                if section_stack.len() > 1 {
                    section_stack.truncate(1);
                }
                section_stack.push(line[3..].trim().to_string());
                current_section = section_stack.join(" > ");
            } else if line.starts_with("### ") {
                // H3 - subsection
                if current_chunk.len() >= self.config.max_chunk_size {
                    chunks.push(DocumentChunk {
                        content: current_chunk.clone(),
                        chunk_index: chunks.len(),
                        total_chunks: 0,
                        section_context: current_section.clone(),
                    });
                    current_chunk.clear();
                }
                if section_stack.len() > 2 {
                    section_stack.truncate(2);
                }
                section_stack.push(line[4..].trim().to_string());
                current_section = section_stack.join(" > ");
            }
            
            current_chunk.push_str(line);
            current_chunk.push('\n');
            
            // Force split if too large
            if current_chunk.len() >= self.config.max_chunk_size + self.config.chunk_overlap {
                chunks.push(DocumentChunk {
                    content: current_chunk.clone(),
                    chunk_index: chunks.len(),
                    total_chunks: 0,
                    section_context: current_section.clone(),
                });
                // Keep overlap
                let overlap_start = current_chunk.len().saturating_sub(self.config.chunk_overlap);
                current_chunk = current_chunk[overlap_start..].to_string();
            }
        }
        
        // Add remaining content
        if !current_chunk.trim().is_empty() {
            chunks.push(DocumentChunk {
                content: current_chunk,
                chunk_index: chunks.len(),
                total_chunks: 0,
                section_context: current_section,
            });
        }
        
        // Update total_chunks
        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total;
        }
        
        chunks
    }
    
    /// Chunk SQL by statement boundaries (CREATE, ALTER, etc.)
    fn chunk_sql_by_statements(&self, content: &str) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        let mut current_context = String::new();
        
        // SQL statement keywords that typically start new logical blocks
        let statement_keywords = ["CREATE", "ALTER", "DROP", "INSERT", "UPDATE", "DELETE", 
                                   "GRANT", "REVOKE", "-- ==", "-- --"];
        
        for line in content.lines() {
            let upper_line = line.to_uppercase();
            
            // Check if this line starts a new statement
            let is_new_statement = statement_keywords.iter()
                .any(|kw| upper_line.trim_start().starts_with(kw));
            
            if is_new_statement && current_chunk.len() >= self.config.max_chunk_size {
                chunks.push(DocumentChunk {
                    content: current_chunk.clone(),
                    chunk_index: chunks.len(),
                    total_chunks: 0,
                    section_context: current_context.clone(),
                });
                current_chunk.clear();
            }
            
            // Extract context from CREATE statements
            if upper_line.contains("CREATE TABLE") || upper_line.contains("CREATE VIEW") {
                if let Some(name) = Self::extract_sql_object_name(line) {
                    current_context = name;
                }
            }
            
            current_chunk.push_str(line);
            current_chunk.push('\n');
        }
        
        if !current_chunk.trim().is_empty() {
            chunks.push(DocumentChunk {
                content: current_chunk,
                chunk_index: chunks.len(),
                total_chunks: 0,
                section_context: current_context,
            });
        }
        
        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total;
        }
        
        chunks
    }
    
    /// Extract object name from SQL CREATE statement
    fn extract_sql_object_name(line: &str) -> Option<String> {
        let upper = line.to_uppercase();
        if let Some(pos) = upper.find("CREATE TABLE") {
            let rest = &line[pos + 12..];
            return Self::extract_first_word(rest);
        }
        if let Some(pos) = upper.find("CREATE VIEW") {
            let rest = &line[pos + 11..];
            return Self::extract_first_word(rest);
        }
        None
    }
    
    fn extract_first_word(s: &str) -> Option<String> {
        s.trim()
            .split(|c: char| c.is_whitespace() || c == '(' || c == '[')
            .next()
            .map(|w| w.trim_matches(|c| c == '"' || c == '\'' || c == '`' || c == '[' || c == ']').to_string())
            .filter(|w| !w.is_empty())
    }
    
    /// Chunk by paragraphs (double newlines)
    fn chunk_by_paragraph(&self, content: &str) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let mut current_chunk = String::new();
        
        // Split by double newlines (paragraphs)
        let paragraphs: Vec<&str> = content.split("\n\n").collect();
        
        for para in paragraphs {
            if current_chunk.len() + para.len() > self.config.max_chunk_size && !current_chunk.is_empty() {
                chunks.push(DocumentChunk {
                    content: current_chunk.clone(),
                    chunk_index: chunks.len(),
                    total_chunks: 0,
                    section_context: String::new(),
                });
                // Keep overlap from end of previous chunk
                let overlap_start = current_chunk.len().saturating_sub(self.config.chunk_overlap);
                current_chunk = current_chunk[overlap_start..].to_string();
            }
            
            if !current_chunk.is_empty() {
                current_chunk.push_str("\n\n");
            }
            current_chunk.push_str(para);
        }
        
        if !current_chunk.trim().is_empty() {
            chunks.push(DocumentChunk {
                content: current_chunk,
                chunk_index: chunks.len(),
                total_chunks: 0,
                section_context: String::new(),
            });
        }
        
        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total;
        }
        
        chunks
    }
    
    /// Fixed-size chunking with overlap
    fn chunk_fixed_size(&self, content: &str) -> Vec<DocumentChunk> {
        let mut chunks = Vec::new();
        let chars: Vec<char> = content.chars().collect();
        let mut start = 0;
        
        while start < chars.len() {
            let end = (start + self.config.max_chunk_size).min(chars.len());
            let chunk_content: String = chars[start..end].iter().collect();
            
            chunks.push(DocumentChunk {
                content: chunk_content,
                chunk_index: chunks.len(),
                total_chunks: 0,
                section_context: format!("Part {}", chunks.len() + 1),
            });
            
            // Move start, accounting for overlap
            start = end.saturating_sub(self.config.chunk_overlap);
            if start >= end {
                break;
            }
        }
        
        let total = chunks.len();
        for chunk in &mut chunks {
            chunk.total_chunks = total;
        }
        
        chunks
    }
}

/// A chunk of document content
#[derive(Debug, Clone)]
pub struct DocumentChunk {
    pub content: String,
    pub chunk_index: usize,
    pub total_chunks: usize,
    pub section_context: String,
}

/// Local documentation processor
pub struct LocalDocsProcessor;

impl LocalDocsProcessor {
    /// Extract text content from a PDF file
    pub fn extract_pdf_text(pdf_path: &Path) -> Result<String> {
        let bytes = fs::read(pdf_path)
            .with_context(|| format!("Failed to read PDF file: {:?}", pdf_path))?;

        let text = pdf_extract::extract_text_from_mem(&bytes)
            .with_context(|| format!("Failed to extract text from PDF: {:?}", pdf_path))?;

        Ok(text)
    }

    /// Read markdown file content
    pub fn read_markdown(md_path: &Path) -> Result<String> {
        fs::read_to_string(md_path)
            .with_context(|| format!("Failed to read Markdown file: {:?}", md_path))
    }

    /// Read text file content
    pub fn read_text(txt_path: &Path) -> Result<String> {
        fs::read_to_string(txt_path)
            .with_context(|| format!("Failed to read text file: {:?}", txt_path))
    }
    
    /// Read SQL file content with schema header
    pub fn read_sql(sql_path: &Path) -> Result<String> {
        let content = fs::read_to_string(sql_path)
            .with_context(|| format!("Failed to read SQL file: {:?}", sql_path))?;
        
        // Add a header to help LLM understand this is database schema
        Ok(format!("-- Database Schema Definition\n-- File: {}\n\n{}", 
            sql_path.file_name().unwrap_or_default().to_string_lossy(),
            content
        ))
    }
    
    /// Read YAML file content (for OpenAPI specs, K8s configs, etc.)
    pub fn read_yaml(yaml_path: &Path) -> Result<String> {
        fs::read_to_string(yaml_path)
            .with_context(|| format!("Failed to read YAML file: {:?}", yaml_path))
    }
    
    /// Read JSON file content (for OpenAPI specs, configs, etc.)
    pub fn read_json(json_path: &Path) -> Result<String> {
        fs::read_to_string(json_path)
            .with_context(|| format!("Failed to read JSON file: {:?}", json_path))
    }
    
    /// Process a documentation file with chunking support
    /// Returns multiple LocalDocMetadata entries if the document is chunked
    pub fn process_file_with_chunking(
        file_path: &Path,
        category: &str,
        target_agents: &[String],
        chunking_config: Option<&ChunkingConfig>,
    ) -> Result<Vec<LocalDocMetadata>> {
        let file_type = Self::detect_file_type(file_path)?;
        
        let raw_content = match file_type {
            DocFileType::Pdf => Self::extract_pdf_text(file_path)?,
            DocFileType::Markdown => Self::read_markdown(file_path)?,
            DocFileType::Text => Self::read_text(file_path)?,
            DocFileType::Sql => Self::read_sql(file_path)?,
            DocFileType::Yaml => Self::read_yaml(file_path)?,
            DocFileType::Json => Self::read_json(file_path)?,
        };

        let metadata = fs::metadata(file_path)?;
        let last_modified = format!("{:?}", metadata.modified()?);
        let file_path_str = file_path.to_string_lossy().to_string();
        
        // Determine if we should chunk
        let config = chunking_config.cloned().unwrap_or_default();
        let chunker = DocumentChunker::new(config);
        
        if !chunker.needs_chunking(&raw_content) {
            // No chunking needed - return single document
            return Ok(vec![LocalDocMetadata {
                file_path: file_path_str,
                file_type,
                last_modified,
                processed_content: raw_content,
                category: category.to_string(),
                target_agents: target_agents.to_vec(),
                chunk_info: None,
            }]);
        }
        
        // Chunk the content
        let chunks = chunker.chunk_content(&raw_content, &file_type);
        
        // Create metadata for each chunk
        let docs: Vec<LocalDocMetadata> = chunks
            .into_iter()
            .map(|chunk| LocalDocMetadata {
                file_path: file_path_str.clone(),
                file_type: file_type.clone(),
                last_modified: last_modified.clone(),
                processed_content: chunk.content,
                category: category.to_string(),
                target_agents: target_agents.to_vec(),
                chunk_info: Some(ChunkInfo {
                    chunk_index: chunk.chunk_index,
                    total_chunks: chunk.total_chunks,
                    section_context: chunk.section_context,
                }),
            })
            .collect();
        
        Ok(docs)
    }
    
    /// Expand glob patterns to actual file paths
    pub fn expand_glob_patterns(patterns: &[String], base_path: Option<&Path>) -> Vec<std::path::PathBuf> {
        let mut files = Vec::new();
        
        for pattern in patterns {
            let pattern_path = Path::new(pattern);
            let full_pattern = if pattern_path.is_absolute() {
                pattern.clone()
            } else if let Some(base) = base_path {
                base.join(pattern_path).to_string_lossy().to_string()
            } else {
                pattern.clone()
            };
            
            match glob(&full_pattern) {
                Ok(paths) => {
                    for entry in paths.flatten() {
                        if entry.is_file() {
                            // Only include supported file types
                            if let Some(ext) = entry.extension().and_then(|e| e.to_str()) {
                                match ext.to_lowercase().as_str() {
                                    // Documentation files
                                    "pdf" | "md" | "markdown" | "txt" | "text" |
                                    // Database schema files
                                    "sql" |
                                    // API specs and config files
                                    "yaml" | "yml" | "json" => {
                                        files.push(entry);
                                    }
                                    _ => {} // Skip unsupported file types
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    eprintln!("  ⚠️  Invalid glob pattern '{}': {}", pattern, e);
                }
            }
        }
        
        files
    }

    /// Detect file type from extension
    fn detect_file_type(file_path: &Path) -> Result<DocFileType> {
        let extension = file_path
            .extension()
            .and_then(|e| e.to_str())
            .ok_or_else(|| anyhow::anyhow!("No file extension found"))?;

        match extension.to_lowercase().as_str() {
            "pdf" => Ok(DocFileType::Pdf),
            "md" | "markdown" => Ok(DocFileType::Markdown),
            "txt" | "text" => Ok(DocFileType::Text),
            "sql" => Ok(DocFileType::Sql),
            "yaml" | "yml" => Ok(DocFileType::Yaml),
            "json" => Ok(DocFileType::Json),
            _ => Err(anyhow::anyhow!("Unsupported file type: {}", extension)),
        }
    }

    /// Format documentation content for LLM with custom header and options
    pub fn format_for_llm_with_options(
        docs: &[LocalDocMetadata],
        custom_header: Option<&str>,
        include_category: bool,
    ) -> String {
        let mut formatted = String::new();
        
        // Add header
        if let Some(header) = custom_header {
            formatted.push_str(header);
        } else {
            formatted.push_str("# Local Technical Documentation\n\n");
        }

        for doc in docs.iter() {
            let filename = Path::new(&doc.file_path)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| doc.file_path.clone());
            
            // Handle chunked documents
            let title = if let Some(ref chunk_info) = doc.chunk_info {
                if chunk_info.total_chunks > 1 {
                    if chunk_info.section_context.is_empty() {
                        format!("{} (Part {}/{})", filename, chunk_info.chunk_index + 1, chunk_info.total_chunks)
                    } else {
                        format!("{} - {} (Part {}/{})", filename, chunk_info.section_context, chunk_info.chunk_index + 1, chunk_info.total_chunks)
                    }
                } else {
                    filename
                }
            } else {
                filename
            };
            
            formatted.push_str(&format!("\n---\n\n## {}\n\n", title));
            formatted.push_str(&format!("**Source:** {}\n", doc.file_path));
            
            if include_category && !doc.category.is_empty() {
                formatted.push_str(&format!("**Category:** {}\n", doc.category));
            }
            
            // Add chunk context if present
            if let Some(ref chunk_info) = doc.chunk_info {
                if chunk_info.total_chunks > 1 {
                    formatted.push_str(&format!("**Chunk:** {}/{}\n", chunk_info.chunk_index + 1, chunk_info.total_chunks));
                    if !chunk_info.section_context.is_empty() {
                        formatted.push_str(&format!("**Section:** {}\n", chunk_info.section_context));
                    }
                }
            }
            
            formatted.push_str(&format!("**Type:** {:?}\n\n", doc.file_type));
            formatted.push_str(&doc.processed_content);
            formatted.push_str("\n\n");
        }

        formatted
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_file_type() {
        assert_eq!(
            LocalDocsProcessor::detect_file_type(Path::new("doc.pdf")).unwrap(),
            DocFileType::Pdf
        );
        assert_eq!(
            LocalDocsProcessor::detect_file_type(Path::new("readme.md")).unwrap(),
            DocFileType::Markdown
        );
        assert_eq!(
            LocalDocsProcessor::detect_file_type(Path::new("notes.txt")).unwrap(),
            DocFileType::Text
        );
    }
}
