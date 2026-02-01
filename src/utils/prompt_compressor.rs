use anyhow::Result;
use serde::{Deserialize, Serialize};

use crate::generator::agent_executor::{AgentExecuteParams, prompt};
use crate::generator::context::GeneratorContext;
use crate::utils::token_estimator::{TokenEstimation, TokenEstimator};

/// Prompt compressor for compressing overly long prompt content
pub struct PromptCompressor {
    token_estimator: TokenEstimator,
    compression_config: CompressionConfig,
}

/// Compression configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionConfig {
    /// Token threshold that triggers compression
    pub compression_threshold: usize,
    /// Target compression ratio (0.0-1.0)
    pub target_compression_ratio: f64,
    /// Whether compression is enabled
    pub enabled: bool,
    /// Types of key information to preserve during compression
    pub preserve_patterns: Vec<PreservePattern>,
}

/// Key information patterns to preserve
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreservePattern {
    /// Function signatures
    FunctionSignatures,
    /// Type definitions
    TypeDefinitions,
    /// Import statements
    ImportStatements,
    /// Interface definitions
    InterfaceDefinitions,
    /// Error handling
    ErrorHandling,
    /// Configuration related
    Configuration,
}

impl Default for CompressionConfig {
    fn default() -> Self {
        Self {
            compression_threshold: 65536, // Reduced to 64K to prevent token overflow
            target_compression_ratio: 0.5, // More aggressive compression to 50%
            enabled: true,
            preserve_patterns: vec![
                PreservePattern::FunctionSignatures,
                PreservePattern::TypeDefinitions,
                PreservePattern::ImportStatements,
                PreservePattern::InterfaceDefinitions,
            ],
        }
    }
}

/// Compression result
#[derive(Debug, Clone)]
pub struct CompressionResult {
    /// Compressed content
    pub compressed_content: String,
    /// Original token count
    pub original_tokens: usize,
    /// Compressed token count
    pub compressed_tokens: usize,
    /// Actual compression ratio
    #[allow(dead_code)]
    pub compression_ratio: f64,
    /// Whether compression was performed
    pub was_compressed: bool,
    /// Compression summary information
    pub compression_summary: String,
}

impl PromptCompressor {
    pub fn new(config: CompressionConfig) -> Self {
        Self {
            token_estimator: TokenEstimator::new(),
            compression_config: config,
        }
    }

    /// Check and compress prompt content if needed
    pub async fn compress_if_needed(
        &self,
        context: &GeneratorContext,
        content: &str,
        content_type: &str,
    ) -> Result<CompressionResult> {
        if !self.compression_config.enabled {
            return Ok(self.create_no_compression_result(content));
        }

        let estimation = self.token_estimator.estimate_tokens(content);

        if estimation.estimated_tokens <= self.compression_config.compression_threshold {
            return Ok(self.create_no_compression_result(content));
        }

        // Check cache
        let cache_manager = context.cache_manager.read().await;
        if let Ok(Some(cached_result)) = cache_manager
            .get_compression_cache(content, content_type)
            .await
        {
            let msg = context.config.target_language.msg_cache_compression_hit().replace("{}", content_type);
            println!("{}", msg);
            let compressed_estimation = self.token_estimator.estimate_tokens(&cached_result);
            let actual_ratio =
                compressed_estimation.estimated_tokens as f64 / estimation.estimated_tokens as f64;

            return Ok(CompressionResult {
                compressed_content: cached_result,
                original_tokens: estimation.estimated_tokens,
                compressed_tokens: compressed_estimation.estimated_tokens,
                compression_ratio: actual_ratio,
                was_compressed: true,
                compression_summary: format!(
                    "Cached compression result: {}tokens -> {}tokens, compression ratio {:.1}%",
                    estimation.estimated_tokens,
                    compressed_estimation.estimated_tokens,
                    (1.0 - actual_ratio) * 100.0
                ),
            });
        }
        drop(cache_manager);

        println!(
            "   🗜️  Detected oversized content [{}]: {} tokens, starting intelligent compression...",
            content_type, estimation.estimated_tokens
        );

        let result = self
            .perform_compression(context, content, content_type, estimation)
            .await?;

        // Cache compression result
        if result.was_compressed {
            let cache_manager = context.cache_manager.write().await;
            let _ = cache_manager
                .set_compression_cache(content, content_type, result.compressed_content.clone())
                .await;
        }

        Ok(result)
    }

    /// Perform actual compression operation
    async fn perform_compression(
        &self,
        context: &GeneratorContext,
        content: &str,
        content_type: &str,
        original_estimation: TokenEstimation,
    ) -> Result<CompressionResult> {
        let target_tokens = ((original_estimation.estimated_tokens as f64
            * self.compression_config.target_compression_ratio)
            as usize)
            .min(self.compression_config.compression_threshold);

        let compression_prompt =
            self.build_compression_prompt(content, content_type, target_tokens);

        let params = AgentExecuteParams {
            prompt_sys:
                "You are a professional content simplification expert, skilled at extracting and preserving key information while significantly reducing content length. Focus on preserving only the most critical information and eliminate all redundancies."
                    .to_string(),
            prompt_user: compression_prompt,
            cache_scope: format!("prompt_compression_{}", content_type),
            log_tag: format!("Context-Compression-{}", content_type),
        };

        // Check if content is already too large for compression
        if original_estimation.estimated_tokens > 150000 {
            return Err(anyhow::anyhow!(
                "Content too large for compression ({} tokens), maximum supported is 150000 tokens",
                original_estimation.estimated_tokens
            ));
        }

        let compressed_content = prompt(context, params).await?;
        let compressed_estimation = self.token_estimator.estimate_tokens(&compressed_content);

        let actual_ratio = compressed_estimation.estimated_tokens as f64
            / original_estimation.estimated_tokens as f64;

        println!(
            "   ✅ Compression complete: {} tokens -> {} tokens (compression ratio: {:.1}%)",
            original_estimation.estimated_tokens,
            compressed_estimation.estimated_tokens,
            (1.0 - actual_ratio) * 100.0
        );

        Ok(CompressionResult {
            compressed_content,
            original_tokens: original_estimation.estimated_tokens,
            compressed_tokens: compressed_estimation.estimated_tokens,
            compression_ratio: actual_ratio,
            was_compressed: true,
            compression_summary: format!(
                "Original {} tokens compressed to {} tokens, compression ratio {:.1}%",
                original_estimation.estimated_tokens,
                compressed_estimation.estimated_tokens,
                (1.0 - actual_ratio) * 100.0
            ),
        })
    }

    /// Build compression prompt
    fn build_compression_prompt(
        &self,
        content: &str,
        content_type: &str,
        target_tokens: usize,
    ) -> String {
        let preserve_instructions = self.build_preserve_instructions();

        format!(
            r#"Please intelligently optimize the following {} content to reduce word count, with the goal of compressing the content to no more than {} tokens.

## CRITICAL Requirements:
1. Preserve ONLY the most essential information and core logic
2. Remove ALL redundant descriptions, verbose explanations, and duplicate information
3. Use extremely concise expressions with bullet points when possible
4. Eliminate unnecessary examples and verbose explanations
5. {}

## Original Content:
{}

## Simplified Content:
Output only the condensed information, with zero additional comments or explanations."#,
            content_type, target_tokens, preserve_instructions, content
        )
    }

    /// Build preserve instructions
    fn build_preserve_instructions(&self) -> String {
        let mut instructions = Vec::new();

        for pattern in &self.compression_config.preserve_patterns {
            let instruction = match pattern {
                PreservePattern::FunctionSignatures => "Preserve all function signatures and method definitions",
                PreservePattern::TypeDefinitions => "Preserve all type definitions and data structures",
                PreservePattern::ImportStatements => "Preserve important import and dependency declarations",
                PreservePattern::InterfaceDefinitions => "Preserve all interface definitions",
                PreservePattern::ErrorHandling => "Preserve error handling related logic",
                PreservePattern::Configuration => "Preserve configuration related information",
            };
            instructions.push(instruction);
        }

        instructions.join("\n")
    }

    /// Create uncompressed result
    fn create_no_compression_result(&self, content: &str) -> CompressionResult {
        let estimation = self.token_estimator.estimate_tokens(content);

        CompressionResult {
            compressed_content: content.to_string(),
            original_tokens: estimation.estimated_tokens,
            compressed_tokens: estimation.estimated_tokens,
            compression_ratio: 1.0,
            was_compressed: false,
            compression_summary: format!("Content not compressed, token count: {}", estimation.estimated_tokens),
        }
    }
}
