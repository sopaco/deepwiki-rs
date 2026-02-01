use anyhow::{Result, anyhow};
use async_trait::async_trait;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::generator::agent_executor::{AgentExecuteParams, extract, prompt, prompt_with_tools};
use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::research::memory::MemoryRetriever;
use crate::{
    generator::context::GeneratorContext,
    types::{
        code::CodeInsight, code_releationship::RelationshipAnalysis,
        project_structure::ProjectStructure,
    },
    utils::project_structure_formatter::ProjectStructureFormatter,
    utils::prompt_compressor::{CompressionConfig, PromptCompressor},
};

/// Replace time placeholders with actual time information
/// This function replaces time placeholders in LLM responses with current actual time
pub fn replace_time_placeholders(content: &str) -> String {
    let now = chrono::Utc::now();
    content
        .replace(
            "__CURRENT_UTC_TIME__",
            &format!("{} (UTC)", now.format("%Y-%m-%d %H:%M:%S")),
        )
        .replace("__CURRENT_TIMESTAMP__", &now.timestamp().to_string())
}

/// Data source configuration - Direct data access mechanism based on Memory Key
#[derive(Debug, Clone, PartialEq)]
pub enum DataSource {
    /// Get data from Memory
    MemoryData {
        scope: &'static str,
        key: &'static str,
    },
    /// Research results from research agent
    ResearchResult(String),
    /// External knowledge from specific categories
    ExternalKnowledgeByCategory(Vec<String>),
}

impl DataSource {
    /// Predefined common data sources
    pub const PROJECT_STRUCTURE: DataSource = DataSource::MemoryData {
        scope: MemoryScope::PREPROCESS,
        key: ScopedKeys::PROJECT_STRUCTURE,
    };
    pub const CODE_INSIGHTS: DataSource = DataSource::MemoryData {
        scope: MemoryScope::PREPROCESS,
        key: ScopedKeys::CODE_INSIGHTS,
    };
    pub const DEPENDENCY_ANALYSIS: DataSource = DataSource::MemoryData {
        scope: MemoryScope::PREPROCESS,
        key: ScopedKeys::RELATIONSHIPS,
    };
    pub const README_CONTENT: DataSource = DataSource::MemoryData {
        scope: MemoryScope::PREPROCESS,
        key: ScopedKeys::ORIGINAL_DOCUMENT,
    };

    /// Create a data source for specific knowledge categories
    pub fn knowledge_categories(categories: Vec<&str>) -> DataSource {
        DataSource::ExternalKnowledgeByCategory(categories.iter().map(|s| s.to_string()).collect())
    }
}

/// Agent data configuration - Declares required data sources
#[derive(Debug, Clone)]
pub struct AgentDataConfig {
    /// Required data sources - Execution fails when missing
    pub required_sources: Vec<DataSource>,
    /// Optional data sources - Does not affect execution when missing
    pub optional_sources: Vec<DataSource>,
}

/// LLM invocation mode configuration
#[derive(Debug, Clone, PartialEq)]
pub enum LLMCallMode {
    /// Use extract method to return specific structured data
    Extract,
    /// Use prompt method to return generalized reasoning text
    #[allow(dead_code)]
    Prompt,
    /// Use prompt method with Built-in Tools to return generalized reasoning text
    PromptWithTools,
}

/// Data formatting configuration
#[derive(Debug, Clone)]
pub struct FormatterConfig {
    /// When file count exceeds the limit, only include folder information. If set to None, include all folders and files
    pub only_directories_when_files_more_than: Option<usize>,
    /// Code insights display quantity limit
    pub code_insights_limit: usize,
    /// Whether to include source code content
    pub include_source_code: bool,
    /// Dependency relationship display quantity limit
    pub dependency_limit: usize,
    /// README content truncation length
    pub readme_truncate_length: Option<usize>,
    /// Whether to enable smart compression
    pub enable_compression: bool,
    /// Compression configuration
    pub compression_config: CompressionConfig,
}

impl Default for FormatterConfig {
    fn default() -> Self {
        Self {
            code_insights_limit: 50,
            include_source_code: false,
            dependency_limit: 50,
            readme_truncate_length: Some(16384),
            enable_compression: true,
            compression_config: CompressionConfig::default(),
            only_directories_when_files_more_than: None,
        }
    }
}

/// Prompt template configuration
#[derive(Debug, Clone)]
pub struct PromptTemplate {
    /// System prompt
    pub system_prompt: String,
    /// Opening instructional statement
    pub opening_instruction: String,
    /// Closing emphasis instruction
    pub closing_instruction: String,
    /// LLM invocation mode
    pub llm_call_mode: LLMCallMode,
    /// Data formatting configuration
    pub formatter_config: FormatterConfig,
}

/// Generic data formatter
pub struct DataFormatter {
    config: FormatterConfig,
    prompt_compressor: Option<PromptCompressor>,
}

impl DataFormatter {
    pub fn new(config: FormatterConfig) -> Self {
        let prompt_compressor = if config.enable_compression {
            Some(PromptCompressor::new(config.compression_config.clone()))
        } else {
            None
        };

        Self {
            config,
            prompt_compressor,
        }
    }

    /// Format project structure information
    pub fn format_project_structure(&self, structure: &ProjectStructure) -> String {
        let config = &self.config;
        if let Some(files_limit) = config.only_directories_when_files_more_than {
            // If exceeds limit, use simplified project structure (only show directories)
            if structure.total_files > files_limit {
                return ProjectStructureFormatter::format_as_directory_tree(structure);
            }
        }

        // Otherwise use complete project structure information
        ProjectStructureFormatter::format_as_tree(structure)
    }

    /// Format code insights information
    pub fn format_code_insights(&self, insights: &[CodeInsight]) -> String {
        let config = &self.config;

        // First sort by importance score
        let mut sorted_insights: Vec<_> = insights.iter().collect();
        sorted_insights.sort_by(|a, b| {
            b.code_dossier
                .importance_score
                .partial_cmp(&a.code_dossier.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let mut content = String::from("### Source Code Insights Summary\n");
        for (i, insight) in sorted_insights
            .iter()
            .take(self.config.code_insights_limit)
            .enumerate()
        {
            content.push_str(&format!(
                "{}. File `{}`, purpose type is `{}`, importance: {:.2}\n",
                i + 1,
                insight.code_dossier.file_path.to_string_lossy(),
                insight.code_dossier.code_purpose,
                insight.code_dossier.importance_score
            ));
            if !insight.detailed_description.is_empty() {
                content.push_str(&format!("   Detailed description: {}\n", &insight.detailed_description));
            }
            if config.include_source_code {
                content.push_str(&format!(
                    "   Source code details: ```code\n{}\n\n",
                    &insight.code_dossier.source_summary
                ));
            }
        }
        content.push_str("\n");
        content
    }

    /// Format README content
    pub fn format_readme_content(&self, readme: &str) -> String {
        let content = if let Some(limit) = self.config.readme_truncate_length {
            if readme.len() > limit {
                format!("{}...(truncated)", &readme[..limit])
            } else {
                readme.to_string()
            }
        } else {
            readme.to_string()
        };
        format!(
            "### Previous README Content (Manually entered information, may not be accurate, for reference only)\n{}\n\n",
            content
        )
    }

    /// Format dependency relationship analysis
    pub fn format_dependency_analysis(&self, deps: &RelationshipAnalysis) -> String {
        let mut content = String::from("### Dependency Relationship Analysis\n");

        // Sort by dependency strength, prioritize important dependencies
        let mut sorted_deps: Vec<_> = deps.core_dependencies.iter().collect();
        sorted_deps.sort_by(|a, b| {
            // Can sort based on dependency type importance
            let a_priority = self.get_dependency_priority(&a.dependency_type);
            let b_priority = self.get_dependency_priority(&b.dependency_type);
            b_priority.cmp(&a_priority)
        });

        for rel in sorted_deps.iter().take(self.config.dependency_limit) {
            content.push_str(&format!(
                "{} -> {} ({})\n",
                rel.from,
                rel.to,
                rel.dependency_type.as_str()
            ));
        }
        content.push_str("\n");
        content
    }

    /// Emergency content truncation when compression fails
    fn emergency_truncate(&self, content: &str, content_type: &str) -> Result<String> {
        // For code insights, truncate more aggressively
        let truncate_ratio = if content_type == "Code Insights" {
            0.2 // Keep only 20% of code insights
        } else {
            0.4 // Keep 40% of other content
        };

        let target_len = (content.len() as f64 * truncate_ratio) as usize;

        if content.len() <= target_len + 100 {
            // Content is already small enough
            return Ok(content.to_string());
        }

        // Find a good truncation point at the end of a line
        let truncated: String = content
            .chars()
            .take(target_len)
            .collect();

        // Find the last newline character to avoid breaking mid-line
        let safe_end = truncated.rfind('\n').unwrap_or(target_len);
        let result = if safe_end > 100 {
            format!("{}\n\n[Content truncated due to size limitations]",
                    &truncated[..safe_end])
        } else {
            format!("{}\n\n[Content truncated due to size limitations]",
                    truncated)
        };

        println!("   🚨 Emergency truncation for [{}]: reduced from {} to {} characters",
                content_type, content.len(), result.len());

        Ok(result)
    }

    /// Get dependency type priority
    fn get_dependency_priority(
        &self,
        dep_type: &crate::types::code_releationship::DependencyType,
    ) -> u8 {
        use crate::types::code_releationship::DependencyType;
        match dep_type {
            DependencyType::Import => 10,
            DependencyType::FunctionCall => 8,
            DependencyType::Inheritance => 9,
            DependencyType::Composition => 7,
            DependencyType::DataFlow => 6,
            DependencyType::Module => 5,
        }
    }

    /// Format research results
    pub fn format_research_results(&self, results: &HashMap<String, serde_json::Value>) -> String {
        let mut content = String::from("### Existing Research Results\n");
        for (key, value) in results {
            content.push_str(&format!(
                "#### {}：\n{}\n\n",
                key,
                serde_json::to_string_pretty(value).unwrap_or_default()
            ));
        }
        content
    }

    /// Smart content compression (if enabled and needed)
    pub async fn compress_content_if_needed(
        &self,
        context: &GeneratorContext,
        content: &str,
        content_type: &str,
    ) -> Result<String> {
        if let Some(compressor) = &self.prompt_compressor {
            match compressor
                .compress_if_needed(context, content, content_type)
                .await
            {
                Ok(compression_result) => {
                    if compression_result.was_compressed {
                        println!("   📊 {}", compression_result.compression_summary);
                    }
                    Ok(compression_result.compressed_content)
                }
                Err(e) => {
                    // If compression fails, try to truncate content to a reasonable size
                    println!("   ⚠️ Compression failed for [{}]: {}, attempting emergency truncation", content_type, e);
                    self.emergency_truncate(content, content_type)
                }
            }
        } else {
            Ok(content.to_string())
        }
    }
}

/// Standard research Agent Prompt builder
pub struct GeneratorPromptBuilder {
    template: PromptTemplate,
    formatter: DataFormatter,
}

impl GeneratorPromptBuilder {
    pub fn new(template: PromptTemplate) -> Self {
        let formatter = DataFormatter::new(template.formatter_config.clone());
        Self {
            template,
            formatter,
        }
    }

    /// Build standard prompt (system prompt and user prompt)
    /// Added custom_content parameter for inserting custom content
    /// Added include_timestamp parameter to control whether to include timestamp information
    /// Added agent_filter parameter for filtering external knowledge by target agent
    pub async fn build_prompts(
        &self,
        context: &GeneratorContext,
        data_sources: &[DataSource],
        custom_content: Option<String>,
        include_timestamp: bool,
        agent_filter: Option<&str>,
    ) -> Result<(String, String)> {
        let system_prompt = self.template.system_prompt.clone();
        let user_prompt = self
            .build_standard_user_prompt(context, data_sources, custom_content, include_timestamp, agent_filter)
            .await?;
        Ok((system_prompt, user_prompt))
    }

    /// Build standard user prompt
    /// Added custom_content parameter
    /// Added include_timestamp parameter to control whether to include timestamp information
    /// Added agent_filter parameter for filtering external knowledge by target agent
    async fn build_standard_user_prompt(
        &self,
        context: &GeneratorContext,
        data_sources: &[DataSource],
        custom_content: Option<String>,
        include_timestamp: bool,
        agent_filter: Option<&str>,
    ) -> Result<String> {
        let mut prompt = String::new();

        // Opening instructional statement
        prompt.push_str(&self.template.opening_instruction);
        prompt.push_str("\n\n");

        // Add current time information based on parameter (using placeholders)
        if include_timestamp {
            prompt.push_str(
                "## Current Time Information\nGeneration time: __CURRENT_UTC_TIME__\nTimestamp: __CURRENT_TIMESTAMP__\n\n"
            );
        }

        // Research materials reference section
        prompt.push_str("## Research Materials Reference\n");

        // Insert custom content (if any)
        if let Some(custom) = custom_content {
            prompt.push_str(&custom);
            prompt.push_str("\n");
        }

        // Collect and format various data sources
        let mut research_results = HashMap::new();

        for source in data_sources {
            match source {
                DataSource::MemoryData { scope, key } => match *key {
                    ScopedKeys::PROJECT_STRUCTURE => {
                        if let Some(structure) = context
                            .get_from_memory::<ProjectStructure>(scope, key)
                            .await
                        {
                            let formatted = self.formatter.format_project_structure(&structure);
                            let compressed = self
                                .formatter
                                .compress_content_if_needed(context, &formatted, "Project Structure")
                                .await?;
                            prompt.push_str(&compressed);
                        }
                    }
                    ScopedKeys::CODE_INSIGHTS => {
                        if let Some(insights) = context
                            .get_from_memory::<Vec<CodeInsight>>(scope, key)
                            .await
                        {
                            let formatted = self.formatter.format_code_insights(&insights);
                            let compressed = self
                                .formatter
                                .compress_content_if_needed(context, &formatted, "Code Insights")
                                .await?;
                            prompt.push_str(&compressed);
                        }
                    }
                    ScopedKeys::ORIGINAL_DOCUMENT => {
                        if let Some(readme) = context.get_from_memory::<String>(scope, key).await {
                            let formatted = self.formatter.format_readme_content(&readme);
                            let compressed = self
                                .formatter
                                .compress_content_if_needed(context, &formatted, "README Document")
                                .await?;
                            prompt.push_str(&compressed);
                        }
                    }
                    ScopedKeys::RELATIONSHIPS => {
                        if let Some(deps) = context
                            .get_from_memory::<RelationshipAnalysis>(scope, key)
                            .await
                        {
                            let formatted = self.formatter.format_dependency_analysis(&deps);
                            let compressed = self
                                .formatter
                                .compress_content_if_needed(context, &formatted, "Dependencies")
                                .await?;
                            prompt.push_str(&compressed);
                        }
                    }
                    _ => {}
                },
                DataSource::ResearchResult(agent_type) => {
                    if let Some(result) = context.get_research(agent_type).await {
                        research_results.insert(agent_type.clone(), result);
                    }
                }
                DataSource::ExternalKnowledgeByCategory(categories) => {
                    // Load external knowledge from specific categories
                    let category_refs: Vec<&str> = categories.iter().map(|s| s.as_str()).collect();
                    if let Some(knowledge) = context
                        .load_external_knowledge_by_categories(&category_refs, agent_filter)
                        .await
                    {
                        let cat_names = categories.join(", ");
                        let formatted = format!("### External Knowledge ({})\n{}\n\n", cat_names, knowledge);
                        let compressed = self
                            .formatter
                            .compress_content_if_needed(context, &formatted, &format!("Knowledge: {}", cat_names))
                            .await?;
                        prompt.push_str(&compressed);
                    }
                }
            }
        }

        // Add research results
        if !research_results.is_empty() {
            let formatted = self.formatter.format_research_results(&research_results);
            let compressed = self
                .formatter
                .compress_content_if_needed(context, &formatted, "Research Results")
                .await?;
            prompt.push_str(&compressed);
        }

        // Closing emphasis instruction
        prompt.push_str(&self.template.closing_instruction);

        // Final detection and compression again
        self.formatter
            .compress_content_if_needed(context, &prompt, "StepForwardAgent_prompt_full")
            .await
    }
}

/// Minimal Agent trait - Greatly simplifies agent implementation
#[async_trait]
pub trait StepForwardAgent: Send + Sync {
    /// Agent output type - Must support JSON serialization
    type Output: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static;

    /// Agent type identifier
    fn agent_type(&self) -> String;

    /// Get the AgentType enum variant (optional, for research agents only)
    fn agent_type_enum(&self) -> Option<crate::generator::research::types::AgentType> {
        None
    }

    fn memory_scope_key(&self) -> String;

    /// Data source configuration
    fn data_config(&self) -> AgentDataConfig;

    /// Prompt template configuration
    fn prompt_template(&self) -> PromptTemplate;

    /// Optional post-processing hook
    fn post_process(&self, _result: &Self::Output, _context: &GeneratorContext) -> Result<()> {
        Ok(())
    }

    /// Optional custom prompt content provider hook
    /// Returns custom prompt content, will be inserted into the research materials reference section of standard prompt
    async fn provide_custom_prompt_content(&self, _context: &GeneratorContext) -> Result<Option<String>> {
        Ok(None)
    }

    /// Whether to include timestamp information in prompt
    /// Defaults to false, only specific agents (such as editor agents in compose directory) need to override as true
    fn should_include_timestamp(&self) -> bool {
        false
    }

    /// Default implementation of execute method - Fully standardized with automatic data validation
    async fn execute(&self, context: &GeneratorContext) -> Result<Self::Output> {
        // 1. Get data configuration
        let config = self.data_config();
        let agent_type_value = self.agent_type();

        // 2. Check if required data sources are available (automatic validation)
        for source in &config.required_sources {
            match source {
                DataSource::MemoryData { scope, key } => {
                    if !context.has_memory_data(scope, key).await {
                        return Err(anyhow!("Required data source {}:{} is not available", scope, key));
                    }
                }
                DataSource::ResearchResult(agent_type) => {
                    if context.get_research(agent_type).await.is_none() {
                        return Err(anyhow!("Required research result {} is not available", agent_type));
                    }
                }
                DataSource::ExternalKnowledgeByCategory(_) => {
                    // External knowledge is optional by nature, don't fail if not available
                }
            }
        }

        // 3. Collect all data sources (required + optional)
        let all_sources = [config.required_sources, config.optional_sources].concat();

        // 4. Build prompt using standard template and adjust according to target language
        let template = self.prompt_template();

        // Add language instruction based on configured target language
        let language_instruction = context.config.target_language.prompt_instruction();

        let prompt_builder = GeneratorPromptBuilder::new(template.clone());

        // Get custom prompt content
        let custom_content = self.provide_custom_prompt_content(context).await?;

        // Check if timestamp needs to be included
        let include_timestamp = self.should_include_timestamp();

        let (system_prompt, user_prompt) = prompt_builder
            .build_prompts(context, &all_sources, custom_content, include_timestamp, Some(agent_type_value.as_str()))
            .await?;

        let system_prompt = format!("{}\n\n{}", system_prompt, language_instruction);
        let user_prompt = format!("{}\n\n{}", user_prompt, language_instruction);

        // 5. Select LLM invocation method based on configuration
        // Use localized agent name for log_tag if available
        let log_tag = if let Some(agent_enum) = self.agent_type_enum() {
            agent_enum.display_name(&context.config.target_language)
        } else {
            agent_type_value.clone()
        };

        let params = AgentExecuteParams {
            prompt_sys: system_prompt,
            prompt_user: user_prompt,
            cache_scope: format!("{}/{}", self.memory_scope_key(), agent_type_value.as_str()),
            log_tag,
        };

        let result_value = match template.llm_call_mode {
            LLMCallMode::Extract => {
                let result: Self::Output = extract(context, params).await?;
                serde_json::to_value(&result)?
            }
            LLMCallMode::Prompt => {
                let result_text: String = prompt(context, params).await?;
                // Replace time placeholders
                let processed_text = replace_time_placeholders(&result_text);
                serde_json::to_value(&processed_text)?
            }
            LLMCallMode::PromptWithTools => {
                let result_text: String = prompt_with_tools(context, params).await?;
                // Replace time placeholders
                let processed_text = replace_time_placeholders(&result_text);
                serde_json::to_value(&processed_text)?
            }
        };

        // 6. Store results
        context
            .store_to_memory(
                &self.memory_scope_key(),
                agent_type_value.as_str(),
                result_value.clone(),
            )
            .await?;

        // 7. Execute post-processing
        if let Ok(typed_result) = serde_json::from_value::<Self::Output>(result_value) {
            self.post_process(&typed_result, context)?;
            // Use localized agent name if available
            let agent_name = if let Some(agent_enum) = self.agent_type_enum() {
                agent_enum.display_name(&context.config.target_language)
            } else {
                agent_type_value.clone()
            };
            println!("✅ Sub-Agent [{}] execution completed", agent_name);
            Ok(typed_result)
        } else {
            Err(anyhow::format_err!(""))
        }
    }
}
