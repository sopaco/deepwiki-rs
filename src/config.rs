use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

use crate::i18n::TargetLanguage;

/// LLM Provider type
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq)]
pub enum LLMProvider {
    #[serde(rename = "openai")]
    OpenAI,
    #[serde(rename = "moonshot")]
    Moonshot,
    #[serde(rename = "deepseek")]
    DeepSeek,
    #[serde(rename = "mistral")]
    Mistral,
    #[serde(rename = "openrouter")]
    OpenRouter,
    #[serde(rename = "anthropic")]
    Anthropic,
    #[serde(rename = "gemini")]
    Gemini,
    #[serde(rename = "ollama")]
    Ollama,
}

impl Default for LLMProvider {
    fn default() -> Self {
        Self::OpenAI
    }
}

impl std::fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LLMProvider::OpenAI => write!(f, "openai"),
            LLMProvider::Moonshot => write!(f, "moonshot"),
            LLMProvider::DeepSeek => write!(f, "deepseek"),
            LLMProvider::Mistral => write!(f, "mistral"),
            LLMProvider::OpenRouter => write!(f, "openrouter"),
            LLMProvider::Anthropic => write!(f, "anthropic"),
            LLMProvider::Gemini => write!(f, "gemini"),
            LLMProvider::Ollama => write!(f, "ollama"),
        }
    }
}

impl std::str::FromStr for LLMProvider {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "openai" => Ok(LLMProvider::OpenAI),
            "moonshot" => Ok(LLMProvider::Moonshot),
            "deepseek" => Ok(LLMProvider::DeepSeek),
            "mistral" => Ok(LLMProvider::Mistral),
            "openrouter" => Ok(LLMProvider::OpenRouter),
            "anthropic" => Ok(LLMProvider::Anthropic),
            "gemini" => Ok(LLMProvider::Gemini),
            "ollama" => Ok(LLMProvider::Ollama),
            _ => Err(format!("Unknown provider: {}", s)),
        }
    }
}

/// Application configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Config {
    /// Project name
    pub project_name: Option<String>,

    /// Project path
    pub project_path: PathBuf,

    /// Output path
    pub output_path: PathBuf,

    /// Internal working directory path (.litho)
    pub internal_path: PathBuf,

    /// Target language
    pub target_language: TargetLanguage,

    /// Whether to analyze dependencies
    pub analyze_dependencies: bool,

    /// Whether to identify core components
    pub identify_components: bool,

    /// Maximum recursion depth
    pub max_depth: u8,

    /// Core component percentage
    pub core_component_percentage: f64,

    /// Maximum file size limit (bytes)
    pub max_file_size: u64,

    /// Whether to include test files
    pub include_tests: bool,

    /// Whether to include hidden files
    pub include_hidden: bool,

    /// Directories to exclude
    pub excluded_dirs: Vec<String>,

    /// Files to exclude
    pub excluded_files: Vec<String>,

    /// File extensions to exclude
    pub excluded_extensions: Vec<String>,

    /// Only include specified file extensions
    pub included_extensions: Vec<String>,

    /// LLM model configuration
    pub llm: LLMConfig,

    /// Cache configuration
    pub cache: CacheConfig,

    /// Knowledge configuration for external documentation sources
    #[serde(default)]
    pub knowledge: KnowledgeConfig,

    /// Architecture meta description file path
    pub architecture_meta_path: Option<PathBuf>,
}

/// LLM model configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LLMConfig {
    /// LLM Provider type
    pub provider: LLMProvider,

    /// LLM API KEY (optional for local providers like Ollama)
    #[serde(default)]
    pub api_key: String,

    /// LLM API base URL
    pub api_base_url: String,

    /// Efficient model, prioritized for Litho engine's regular inference tasks
    pub model_efficient: String,

    /// Powerful model, prioritized for Litho engine's complex inference tasks, and as fallback when efficient fails
    pub model_powerful: String,

    /// Maximum tokens
    pub max_tokens: u32,

    /// Temperature (optional - some models like o3-mini don't support it)
    pub temperature: Option<f64>,

    /// Retry attempts
    pub retry_attempts: u32,

    /// Retry interval (milliseconds)
    pub retry_delay_ms: u64,

    /// Timeout duration (seconds)
    pub timeout_seconds: u64,

    pub disable_preset_tools: bool,

    pub max_parallels: usize,
}

/// Cache configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CacheConfig {
    /// Whether to enable cache
    pub enabled: bool,

    /// Cache directory
    pub cache_dir: PathBuf,

    /// Cache expiration time (hours)
    pub expire_hours: u64,
}

/// Knowledge configuration for external documentation sources
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct KnowledgeConfig {
    /// Local documentation files configuration
    pub local_docs: Option<LocalDocsConfig>,
}

/// Document category for organizing external knowledge
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DocumentCategory {
    /// Category identifier (e.g., "architecture", "database", "api")
    pub name: String,

    /// Human-readable description of this category
    #[serde(default)]
    pub description: String,

    /// File paths or glob patterns for this category
    #[serde(default)]
    pub paths: Vec<String>,

    /// Which agents should receive documents from this category
    /// If empty, documents are available to all agents
    #[serde(default)]
    pub target_agents: Vec<String>,

    /// Chunking configuration for large documents in this category
    #[serde(default)]
    pub chunking: Option<ChunkingConfig>,
}

/// Configuration for document chunking
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ChunkingConfig {
    /// Enable chunking for large documents (default: true)
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Maximum chunk size in characters (default: 8000 ~2000 tokens)
    #[serde(default = "default_chunk_size")]
    pub max_chunk_size: usize,

    /// Overlap between chunks in characters (default: 200)
    #[serde(default = "default_chunk_overlap")]
    pub chunk_overlap: usize,

    /// Chunking strategy: "semantic" (by sections), "fixed" (fixed size), "paragraph"
    #[serde(default = "default_chunk_strategy")]
    pub strategy: String,

    /// Minimum document size (chars) to trigger chunking (default: 10000)
    #[serde(default = "default_min_size_for_chunking")]
    pub min_size_for_chunking: usize,
}

impl Default for ChunkingConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            max_chunk_size: 8000,
            chunk_overlap: 200,
            strategy: "semantic".to_string(),
            min_size_for_chunking: 10000,
        }
    }
}

fn default_chunk_size() -> usize {
    8000
}

fn default_chunk_overlap() -> usize {
    200
}

fn default_chunk_strategy() -> String {
    "semantic".to_string()
}

fn default_min_size_for_chunking() -> usize {
    10000
}

/// Local documentation files configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LocalDocsConfig {
    /// Whether local docs integration is enabled
    #[serde(default)]
    pub enabled: bool,

    /// Categorized document sources
    /// Each category can have its own paths and target agents
    #[serde(default)]
    pub categories: Vec<DocumentCategory>,

    /// Local cache directory for processed content
    pub cache_dir: Option<PathBuf>,

    /// Whether to re-process files if they change
    #[serde(default = "default_true")]
    pub watch_for_changes: bool,

    /// Default chunking configuration for all categories
    /// Can be overridden per category
    #[serde(default)]
    pub default_chunking: Option<ChunkingConfig>,
}

fn default_true() -> bool {
    true
}

impl Config {
    /// Load configuration from file
    pub fn from_file(path: &PathBuf) -> Result<Self> {
        let mut file =
            File::open(path).context(format!("Failed to open config file: {:?}", path))?;
        let mut content = String::new();
        file.read_to_string(&mut content)
            .context("Failed to read config file")?;

        let config: Config = toml::from_str(&content).context("Failed to parse config file")?;
        Ok(config)
    }

    /// Get project name, prioritize configured project_name, otherwise auto-infer
    pub fn get_project_name(&self) -> String {
        // Prioritize configured project name
        if let Some(ref name) = self.project_name {
            if !name.trim().is_empty() {
                return name.clone();
            }
        }

        // If not configured or empty, auto-infer
        self.infer_project_name()
    }

    /// Auto-infer project name
    fn infer_project_name(&self) -> String {
        // Try to extract project name from project configuration files
        if let Some(name) = self.extract_project_name_from_config_files() {
            return name;
        }

        // Infer from project path
        self.project_path
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string()
    }

    /// Extract project name from project configuration files
    fn extract_project_name_from_config_files(&self) -> Option<String> {
        // Try to extract from Cargo.toml (Rust project)
        if let Some(name) = self.extract_from_cargo_toml() {
            return Some(name);
        }

        // Try to extract from package.json (Node.js project)
        if let Some(name) = self.extract_from_package_json() {
            return Some(name);
        }

        // Try to extract from pyproject.toml (Python project)
        if let Some(name) = self.extract_from_pyproject_toml() {
            return Some(name);
        }

        // Try to extract from pom.xml (Java Maven project)
        if let Some(name) = self.extract_from_pom_xml() {
            return Some(name);
        }

        // Try to extract from .csproj (C# project)
        if let Some(name) = self.extract_from_csproj() {
            return Some(name);
        }

        None
    }

    /// Extract project name from Cargo.toml
    pub fn extract_from_cargo_toml(&self) -> Option<String> {
        let cargo_path = self.project_path.join("Cargo.toml");
        if !cargo_path.exists() {
            return None;
        }

        match std::fs::read_to_string(&cargo_path) {
            Ok(content) => {
                // Find name under [package] section
                let mut in_package_section = false;
                for line in content.lines() {
                    let line = line.trim();
                    if line == "[package]" {
                        in_package_section = true;
                        continue;
                    }
                    if line.starts_with('[') && in_package_section {
                        in_package_section = false;
                        continue;
                    }
                    if in_package_section && line.starts_with("name") && line.contains("=") {
                        if let Some(name_part) = line.split('=').nth(1) {
                            let name = name_part.trim().trim_matches('"').trim_matches('\'');
                            if !name.is_empty() {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }
            Err(_) => return None,
        }
        None
    }

    /// Extract project name from package.json
    pub fn extract_from_package_json(&self) -> Option<String> {
        let package_path = self.project_path.join("package.json");
        if !package_path.exists() {
            return None;
        }

        match std::fs::read_to_string(&package_path) {
            Ok(content) => {
                // Simple JSON parsing, find "name": "..."
                for line in content.lines() {
                    let line = line.trim();
                    if line.starts_with("\"name\"") && line.contains(":") {
                        if let Some(name_part) = line.split(':').nth(1) {
                            let name = name_part
                                .trim()
                                .trim_matches(',')
                                .trim_matches('"')
                                .trim_matches('\'');
                            if !name.is_empty() {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }
            Err(_) => return None,
        }
        None
    }

    /// Extract project name from pyproject.toml
    pub fn extract_from_pyproject_toml(&self) -> Option<String> {
        let pyproject_path = self.project_path.join("pyproject.toml");
        if !pyproject_path.exists() {
            return None;
        }

        match std::fs::read_to_string(&pyproject_path) {
            Ok(content) => {
                // Find name under [project] or [tool.poetry]
                let mut in_project_section = false;
                let mut in_poetry_section = false;

                for line in content.lines() {
                    let line = line.trim();
                    if line == "[project]" {
                        in_project_section = true;
                        in_poetry_section = false;
                        continue;
                    }
                    if line == "[tool.poetry]" {
                        in_poetry_section = true;
                        in_project_section = false;
                        continue;
                    }
                    if line.starts_with('[') && (in_project_section || in_poetry_section) {
                        in_project_section = false;
                        in_poetry_section = false;
                        continue;
                    }
                    if (in_project_section || in_poetry_section)
                        && line.starts_with("name")
                        && line.contains("=")
                    {
                        if let Some(name_part) = line.split('=').nth(1) {
                            let name = name_part.trim().trim_matches('"').trim_matches('\'');
                            if !name.is_empty() {
                                return Some(name.to_string());
                            }
                        }
                    }
                }
            }
            Err(_) => return None,
        }
        None
    }

    /// Extract project name from .csproj
    fn extract_from_csproj(&self) -> Option<String> {
        // Find all .csproj files
        if let Ok(entries) = std::fs::read_dir(&self.project_path) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|e| e.to_str()) == Some("csproj") {
                    if let Ok(content) = std::fs::read_to_string(&path) {
                        // Extract project name from filename (remove .csproj extension)
                        if let Some(file_stem) = path.file_stem() {
                            if let Some(name) = file_stem.to_str() {
                                return Some(name.to_string());
                            }
                        }

                        // Try to extract <AssemblyName> or <PackageId> from XML
                        for line in content.lines() {
                            let line = line.trim();
                            if line.starts_with("<AssemblyName>") && line.ends_with("</AssemblyName>") {
                                let name = line
                                    .trim_start_matches("<AssemblyName>")
                                    .trim_end_matches("</AssemblyName>");
                                if !name.is_empty() {
                                    return Some(name.to_string());
                                }
                            }
                            if line.starts_with("<PackageId>") && line.ends_with("</PackageId>") {
                                let name = line
                                    .trim_start_matches("<PackageId>")
                                    .trim_end_matches("</PackageId>");
                                if !name.is_empty() {
                                    return Some(name.to_string());
                                }
                            }
                        }
                    }
                }
            }
        }
        None
    }

    /// Extract project name from pom.xml
    fn extract_from_pom_xml(&self) -> Option<String> {
        let pom_path = self.project_path.join("pom.xml");
        if !pom_path.exists() {
            return None;
        }

        match std::fs::read_to_string(&pom_path) {
            Ok(content) => {
                // Simple XML parsing, find <artifactId> or <name>
                let lines: Vec<&str> = content.lines().collect();
                for line in lines {
                    let line = line.trim();
                    // Prioritize <name> tag
                    if line.starts_with("<name>") && line.ends_with("</name>") {
                        let name = line
                            .trim_start_matches("<name>")
                            .trim_end_matches("</name>");
                        if !name.is_empty() {
                            return Some(name.to_string());
                        }
                    }
                    // Use <artifactId> tag as fallback
                    if line.starts_with("<artifactId>") && line.ends_with("</artifactId>") {
                        let name = line
                            .trim_start_matches("<artifactId>")
                            .trim_end_matches("</artifactId>");
                        if !name.is_empty() {
                            return Some(name.to_string());
                        }
                    }
                }
            }
            Err(_) => return None,
        }
        None
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            project_name: None,
            project_path: PathBuf::from("."),
            output_path: PathBuf::from("./litho.docs"),
            internal_path: PathBuf::from("./.litho"),
            target_language: TargetLanguage::default(),
            analyze_dependencies: true,
            identify_components: true,
            max_depth: 10,
            core_component_percentage: 20.0,
            max_file_size: 64 * 1024, // 64KB
            include_tests: false,
            include_hidden: false,
            excluded_dirs: vec![
                ".litho".to_string(),
                "litho.docs".to_string(),
                "target".to_string(),
                "node_modules".to_string(),
                ".git".to_string(),
                "build".to_string(),
                "dist".to_string(),
                "venv".to_string(),
                ".svelte-kit".to_string(),
                "__pycache__".to_string(),
                "__tests__".to_string(),
                "__mocks__".to_string(),
                "__fixtures__".to_string(),
            ],
            excluded_files: vec![
                "litho.toml".to_string(),
                "*.litho".to_string(),
                "*.log".to_string(),
                "*.tmp".to_string(),
                "*.cache".to_string(),
                "bun.lock".to_string(),
                "package-lock.json".to_string(),
                "yarn.lock".to_string(),
                "pnpm-lock.yaml".to_string(),
                "Cargo.lock".to_string(),
                ".gitignore".to_string(),
                "*.tpl".to_string(),
                "*.md".to_string(),
                "*.txt".to_string(),
                ".env".to_string(),
            ],
            excluded_extensions: vec![
                "jpg".to_string(),
                "jpeg".to_string(),
                "png".to_string(),
                "gif".to_string(),
                "bmp".to_string(),
                "ico".to_string(),
                "mp3".to_string(),
                "mp4".to_string(),
                "avi".to_string(),
                "pdf".to_string(),
                "zip".to_string(),
                "tar".to_string(),
                "exe".to_string(),
                "dll".to_string(),
                "so".to_string(),
                "archive".to_string(),
            ],
            included_extensions: vec![],
            architecture_meta_path: None,
            llm: LLMConfig::default(),
            cache: CacheConfig::default(),
            knowledge: KnowledgeConfig::default(),
        }
    }
}

impl Default for LLMConfig {
    fn default() -> Self {
        Self {
            provider: LLMProvider::default(),
            api_key: std::env::var("LITHO_LLM_API_KEY").unwrap_or_default(),
            api_base_url: String::from("https://api-inference.modelscope.cn/v1"),
            model_efficient: String::from("ZhipuAI/GLM-5"),
            model_powerful: String::from("Qwen/Qwen3-Next-80B-A3B-Instruct"),
            max_tokens: 131072,
            temperature: Some(0.1),
            retry_attempts: 3,
            retry_delay_ms: 5000,
            timeout_seconds: 300,
            disable_preset_tools: false,
            max_parallels: 3,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            cache_dir: PathBuf::from(".litho/cache"),
            expire_hours: 8760,
        }
    }
}
