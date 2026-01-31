use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::research::types::{AgentType, BoundaryAnalysisReport};
use crate::generator::{
    context::GeneratorContext,
    step_forward_agent::{
        AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
    },
};
use crate::types::code::{CodeInsight, CodePurpose};
use anyhow::{Result, anyhow};
use async_trait::async_trait;

/// Boundary Interface Analyzer - Responsible for analyzing the external call boundaries of the system, including CLI, API, configuration interfaces, etc.
#[derive(Default, Clone)]
pub struct BoundaryAnalyzer;

#[async_trait]
impl StepForwardAgent for BoundaryAnalyzer {
    type Output = BoundaryAnalysisReport;

    fn agent_type(&self) -> String {
        AgentType::BoundaryAnalyzer.to_string()
    }

    fn agent_type_enum(&self) -> Option<AgentType> {
        Some(AgentType::BoundaryAnalyzer)
    }

    fn memory_scope_key(&self) -> String {
        crate::generator::research::memory::MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::PROJECT_STRUCTURE,
                DataSource::DEPENDENCY_ANALYSIS,
                DataSource::ResearchResult(AgentType::SystemContextResearcher.to_string()),
            ],
            // Use API and deployment docs for boundary analysis
            optional_sources: vec![DataSource::knowledge_categories(vec!["api", "deployment"])],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt:
                r#"You are a professional system boundary interface analyst, focused on identifying and analyzing external call boundaries of software systems.

Your task is to identify and analyze based on the provided boundary-related code:
1. CLI Command Line Interface - commands, parameters, options, usage examples
2. API Interface - HTTP endpoints, request/response formats, authentication methods
3. Router Routes - page router routes, URL paths, route parameters
4. Integration Suggestions - best practices and example code

You may have access to existing product description, requirements and architecture documentation from external sources.
If available:
- Cross-reference code endpoints with documented API specifications
- Validate authentication and authorization mechanisms
- Use established API versioning and naming conventions
- Reference documented integration patterns and examples
- Identify any undocumented endpoints or missing documentation

Focus on:
- Extract boundary information from Entry, Api, Controller, Router type code
- Analyze interface definitions, parameter structures, dependency relationships in the code
- Identify mechanisms and methods for external systems to call this system
- Provide practical integration guidance and security recommendations

Please return the analysis results in structured JSON format."#
                    .to_string(),

            opening_instruction: "Analyze the system's boundary interfaces based on the following boundary-related code and project information:".to_string(),

            closing_instruction: r#"
## Analysis Requirements:
- Focus on Entry, Api, Controller, Config, Router type code
- Extract specific boundary information from code structure and interface definitions
- Generate practical usage examples and integration suggestions
- Identify potential security risks and provide mitigation strategies
- Ensure analysis results are accurate, complete, and practical
- If a certain type of boundary interface does not exist, the corresponding array can be empty"#
                .to_string(),

            llm_call_mode: LLMCallMode::Extract,
            formatter_config: FormatterConfig {
                include_source_code: true, // Boundary analysis requires viewing source code details
                code_insights_limit: 100,  // Increase code insights limit to ensure no boundary code is missed
                only_directories_when_files_more_than: Some(500), // Appropriate limit to avoid information overload
                ..FormatterConfig::default()
            },
        }
    }

    /// Provide custom boundary code analysis content
    async fn provide_custom_prompt_content(
        &self,
        context: &GeneratorContext,
    ) -> Result<Option<String>> {
        // 1. Filter boundary-related code insights
        let boundary_insights = self.filter_boundary_code_insights(context).await?;

        if boundary_insights.is_empty() {
            return Ok(Some(
                "### Boundary-Related Code Insights\nNo obvious boundary interface-related code found.\n\n".to_string(),
            ));
        }

        // 2. Format boundary code insights
        let formatted_content = self.format_boundary_insights(&boundary_insights);

        Ok(Some(formatted_content))
    }

    /// Post-processing - output analysis summary
    fn post_process(
        &self,
        result: &BoundaryAnalysisReport,
        _context: &GeneratorContext,
    ) -> Result<()> {
        println!("✅ Boundary interface analysis completed:");
        println!("   - CLI commands: {} items", result.cli_boundaries.len());
        println!("   - API interfaces: {} items", result.api_boundaries.len());
        println!("   - Router routes: {} items", result.router_boundaries.len());
        println!("   - Integration suggestions: {} items", result.integration_suggestions.len());
        println!("   - Confidence: {:.1}/10", result.confidence_score);

        Ok(())
    }
}

impl BoundaryAnalyzer {
    /// Filter boundary-related code insights
    async fn filter_boundary_code_insights(
        &self,
        context: &GeneratorContext,
    ) -> Result<Vec<CodeInsight>> {
        let all_insights = context
            .get_from_memory::<Vec<CodeInsight>>(MemoryScope::PREPROCESS, ScopedKeys::CODE_INSIGHTS)
            .await
            .ok_or_else(|| anyhow!("CODE_INSIGHTS not found in PREPROCESS memory"))?;

        // Filter boundary-related code
        let boundary_insights: Vec<CodeInsight> = all_insights
            .into_iter()
            .filter(|insight| {
                matches!(
                    insight.code_dossier.code_purpose,
                    CodePurpose::Entry
                        | CodePurpose::Api
                        | CodePurpose::Config
                        | CodePurpose::Router
                        | CodePurpose::Controller
                )
            })
            .collect();

        // Sort by importance, take top 50 most important
        let mut sorted_insights = boundary_insights;
        sorted_insights.sort_by(|a, b| {
            b.code_dossier
                .importance_score
                .partial_cmp(&a.code_dossier.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        sorted_insights.truncate(50);

        // Group by type and count
        let mut entry_count = 0;
        let mut api_count = 0;
        let mut config_count = 0;
        let mut router_count = 0;

        for insight in &sorted_insights {
            match insight.code_dossier.code_purpose {
                CodePurpose::Entry => entry_count += 1,
                CodePurpose::Api => api_count += 1,
                CodePurpose::Config => config_count += 1,
                CodePurpose::Router => router_count += 1,
                CodePurpose::Controller => api_count += 1,
                _ => {}
            }
        }

        println!(
            "📊 Boundary code distribution: Entry({}) API/Controller({}) Config({}) Router({})",
            entry_count, api_count, config_count, router_count
        );

        Ok(sorted_insights)
    }

    /// Format boundary code insights - specialized formatting logic
    fn format_boundary_insights(&self, insights: &[CodeInsight]) -> String {
        let mut content = String::from("### Boundary-Related Code Insights\n");

        // Group by CodePurpose for display
        let mut entry_codes = Vec::new();
        let mut api_codes = Vec::new();
        let mut config_codes = Vec::new();
        let mut router_codes = Vec::new();

        for insight in insights {
            match insight.code_dossier.code_purpose {
                CodePurpose::Entry => entry_codes.push(insight),
                CodePurpose::Api => api_codes.push(insight),
                CodePurpose::Controller => api_codes.push(insight),
                CodePurpose::Config => config_codes.push(insight),
                CodePurpose::Router => router_codes.push(insight),
                _ => {}
            }
        }

        if !entry_codes.is_empty() {
            content.push_str("#### Entry Point Code (Entry)\n");
            content.push_str("These code usually contain CLI command definitions, main function entry points, etc.:\n\n");
            for insight in entry_codes {
                self.add_boundary_insight_item(&mut content, insight);
            }
        }

        if !api_codes.is_empty() {
            content.push_str("#### API/Controller Code (API/Controller)\n");
            content.push_str("These code usually contain HTTP endpoints, API routes, controller logic, etc.:\n\n");
            for insight in api_codes {
                self.add_boundary_insight_item(&mut content, insight);
            }
        }

        if !config_codes.is_empty() {
            content.push_str("#### Configuration-Related Code (Config)\n");
            content.push_str("These code usually contain configuration structures, parameter definitions, environment variables, etc.:\n\n");
            for insight in config_codes {
                self.add_boundary_insight_item(&mut content, insight);
            }
        }

        if !router_codes.is_empty() {
            content.push_str("#### Router-Related Code (Router)\n");
            content.push_str("These code usually contain route definitions, middleware, request handling, etc.:\n\n");
            for insight in router_codes {
                self.add_boundary_insight_item(&mut content, insight);
            }
        }

        content.push_str("\n");
        content
    }

    /// Add single boundary code insight item
    fn add_boundary_insight_item(&self, content: &mut String, insight: &CodeInsight) {
        content.push_str(&format!(
            "**File**: `{}` (Importance: {:.2}, Purpose: {:?})\n",
            insight.code_dossier.file_path.to_string_lossy(),
            insight.code_dossier.importance_score,
            insight.code_dossier.code_purpose
        ));

        if !insight.detailed_description.is_empty() {
            content.push_str(&format!("- **Description**: {}\n", insight.detailed_description));
        }

        if !insight.responsibilities.is_empty() {
            content.push_str(&format!("- **Responsibilities**: {:?}\n", insight.responsibilities));
        }

        if !insight.interfaces.is_empty() {
            content.push_str(&format!("- **Interfaces**: {:?}\n", insight.interfaces));
        }

        if !insight.dependencies.is_empty() {
            content.push_str(&format!("- **Dependencies**: {:?}\n", insight.dependencies));
        }

        if !insight.code_dossier.source_summary.is_empty() {
            content.push_str(&format!(
                "- **Source Summary**:\n```\n{}\n```\n",
                insight.code_dossier.source_summary
            ));
        }

        content.push_str("\n");
    }
}
