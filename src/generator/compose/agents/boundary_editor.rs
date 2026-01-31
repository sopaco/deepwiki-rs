use crate::generator::compose::memory::MemoryScope;
use crate::generator::compose::types::AgentType;
use crate::generator::context::GeneratorContext;
use crate::generator::research::memory::MemoryRetriever;
use crate::generator::research::types::{
    APIBoundary, AgentType as ResearchAgentType, BoundaryAnalysisReport, CLIBoundary,
    IntegrationSuggestion, RouterBoundary,
};
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, PromptTemplate, StepForwardAgent,
};
use anyhow::Result;
use async_trait::async_trait;

/// Boundary Interface Documentation Editor - Orchestrates boundary analysis results into standardized documentation
#[derive(Default)]
pub struct BoundaryEditor;

#[async_trait]
impl StepForwardAgent for BoundaryEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        AgentType::Boundary.to_string()
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::DOCUMENTATION.to_string()
    }

    fn should_include_timestamp(&self) -> bool {
        true
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![],
            optional_sources: vec![
                DataSource::ResearchResult(ResearchAgentType::BoundaryAnalyzer.to_string()),
                DataSource::PROJECT_STRUCTURE,
                DataSource::CODE_INSIGHTS,
                DataSource::README_CONTENT,
                // Use API and deployment docs for boundary documentation
                DataSource::knowledge_categories(vec!["api", "deployment"]),
            ],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"You are a professional software interface documentation expert, focused on generating clear, detailed boundary interface documentation. Your task is to write an interface documentation with the title `Boundary Interfaces` based on the provided research report.

## External Knowledge Integration:
You may have access to existing product description, requirements and architecture documentation from external sources.
If available:
- Cross-reference code interfaces with documented API specifications
- Use established endpoint naming and versioning conventions
- Incorporate documented authentication and authorization patterns
- Reference documented integration examples and best practices
- Validate implementation against documented API contracts
- Highlight any undocumented endpoints or changes

## Documentation Requirements
1. **Complete Interfaces**: Describe all external interfaces in detail
2. **Clear Parameters**: Each parameter must have a clear explanation
3. **Rich Examples**: Provide practical usage examples
4. **Easy to Understand**: Provide valuable references for developers
5. **Consistency**: Maintain alignment with external API documentation when available

## Output Format
- Use Markdown format
- Include appropriate heading levels
- Use code blocks to show examples
- Ensure logical and readable content"#.to_string(),

            opening_instruction: "Based on the following boundary analysis results, generate system boundary interface documentation:".to_string(),

            closing_instruction: r#"
## Documentation Requirements:
- Use standard Markdown format
- Create separate sections for each boundary type
- Include detailed parameter descriptions and usage examples
- Highlight security considerations and best practices
- Ensure clear document structure and complete content
- Validate against external API documentation if available"#
                .to_string(),

            llm_call_mode: crate::generator::step_forward_agent::LLMCallMode::Prompt,
            formatter_config: crate::generator::step_forward_agent::FormatterConfig::default(),
        }
    }

    /// Custom execute implementation that generates documentation directly without using LLM
    async fn execute(&self, context: &GeneratorContext) -> Result<Self::Output> {
        // Get boundary analysis results from memory
        let boundary_analysis = context
            .get_research(&ResearchAgentType::BoundaryAnalyzer.to_string())
            .await
            .ok_or_else(|| anyhow::anyhow!("BoundaryAnalyzer result not found"))?;

        // Parse as BoundaryAnalysisReport
        let report: BoundaryAnalysisReport = serde_json::from_value(boundary_analysis)?;

        // Generate documentation content
        let content = self.generate_boundary_documentation(&report);

        // Store to memory
        let value = serde_json::to_value(&content)?;
        context
            .store_to_memory(&self.memory_scope_key(), &self.agent_type(), value)
            .await?;

        Ok(content)
    }
}

impl BoundaryEditor {
    /// Generate boundary interface documentation
    fn generate_boundary_documentation(&self, report: &BoundaryAnalysisReport) -> String {
        let mut content = String::new();
        content.push_str("# System Boundary Interface Documentation\n\n");
        content.push_str(
            "This document describes the system's external invocation interfaces, including CLI commands, API endpoints, configuration parameters, and other boundary mechanisms.\n\n",
        );

        // Generate CLI interface documentation
        if !report.cli_boundaries.is_empty() {
            content.push_str(&self.generate_cli_documentation(&report.cli_boundaries));
        }

        // Generate API interface documentation
        if !report.api_boundaries.is_empty() {
            content.push_str(&self.generate_api_documentation(&report.api_boundaries));
        }

        // Generate Router route documentation
        if !report.router_boundaries.is_empty() {
            content.push_str(&self.generate_router_documentation(&report.router_boundaries));
        }

        // Generate integration suggestions
        if !report.integration_suggestions.is_empty() {
            content.push_str(
                &self.generate_integration_documentation(&report.integration_suggestions),
            );
        }

        // Add analysis confidence score
        content.push_str(&format!(
            "\n---\n\n**Analysis Confidence**: {:.1}/10\n",
            report.confidence_score
        ));

        content
    }

    fn generate_cli_documentation(&self, cli_boundaries: &[CLIBoundary]) -> String {
        if cli_boundaries.len() == 0 {
            return String::new();
        }

        let mut content = String::new();
        content.push_str("## Command Line Interface (CLI)\n\n");

        for cli in cli_boundaries {
            content.push_str(&format!("### {}\n\n", cli.command));
            content.push_str(&format!("**Description**: {}\n\n", cli.description));
            content.push_str(&format!("**Source File**: `{}`\n\n", cli.source_location));

            if !cli.arguments.is_empty() {
                content.push_str("**Arguments**:\n\n");
                for arg in &cli.arguments {
                    let required_text = if arg.required { "required" } else { "optional" };
                    let default_text = arg
                        .default_value
                        .as_ref()
                        .map(|v| format!(" (default: `{}`)", v))
                        .unwrap_or_default();
                    content.push_str(&format!(
                        "- `{}` ({}): {} - {}{}\n",
                        arg.name, arg.value_type, required_text, arg.description, default_text
                    ));
                }
                content.push_str("\n");
            }

            if !cli.options.is_empty() {
                content.push_str("**Options**:\n\n");
                for option in &cli.options {
                    let short_text = option
                        .short_name
                        .as_ref()
                        .map(|s| format!(", {}", s))
                        .unwrap_or_default();
                    let required_text = if option.required { "required" } else { "optional" };
                    let default_text = option
                        .default_value
                        .as_ref()
                        .map(|v| format!(" (default: `{}`)", v))
                        .unwrap_or_default();
                    content.push_str(&format!(
                        "- `{}{}`({}): {} - {}{}\n",
                        option.name,
                        short_text,
                        option.value_type,
                        required_text,
                        option.description,
                        default_text
                    ));
                }
                content.push_str("\n");
            }

            if !cli.examples.is_empty() {
                content.push_str("**Usage Examples**:\n\n");
                for example in &cli.examples {
                    content.push_str(&format!("```bash\n{}\n```\n\n", example));
                }
            }
        }

        content
    }

    fn generate_api_documentation(&self, api_boundaries: &[APIBoundary]) -> String {
        if api_boundaries.len() == 0 {
            return String::new();
        }

        let mut content = String::new();
        content.push_str("## API Interfaces\n\n");

        for api in api_boundaries {
            content.push_str(&format!("### {} {}\n\n", api.method, api.endpoint));
            content.push_str(&format!("**Description**: {}\n\n", api.description));
            content.push_str(&format!("**Source File**: `{}`\n\n", api.source_location));

            if let Some(request_format) = &api.request_format {
                content.push_str(&format!("**Request Format**: {}\n\n", request_format));
            }

            if let Some(response_format) = &api.response_format {
                content.push_str(&format!("**Response Format**: {}\n\n", response_format));
            }

            if let Some(auth) = &api.authentication {
                content.push_str(&format!("**Authentication**: {}\n\n", auth));
            }
        }

        content
    }

    fn generate_router_documentation(&self, router_boundaries: &[RouterBoundary]) -> String {
        if router_boundaries.len() == 0 {
            return String::new();
        }

        let mut content = String::new();
        content.push_str("## Router Routes\n\n");

        for router in router_boundaries {
            content.push_str(&format!("### {}\n\n", router.path));
            content.push_str(&format!("**Description**: {}\n\n", router.description));
            content.push_str(&format!("**Source File**: `{}`\n\n", router.source_location));

            if !router.params.is_empty() {
                content.push_str("**Parameters**:\n\n");
                for param in &router.params {
                    content.push_str(&format!(
                        "- `{}` ({}): {}\n",
                        param.key, param.value_type, param.description
                    ));
                }
            }
        }

        content
    }

    fn generate_integration_documentation(
        &self,
        integration_suggestions: &[IntegrationSuggestion],
    ) -> String {
        if integration_suggestions.len() == 0 {
            return String::new();
        }

        let mut content = String::new();
        content.push_str("## Integration Suggestions\n\n");

        for suggestion in integration_suggestions {
            content.push_str(&format!("### {}\n\n", suggestion.integration_type));
            content.push_str(&format!("{}\n\n", suggestion.description));

            if !suggestion.example_code.is_empty() {
                content.push_str("**Example Code**:\n\n");
                content.push_str(&format!("```\n{}\n```\n\n", suggestion.example_code));
            }

            if !suggestion.best_practices.is_empty() {
                content.push_str("**Best Practices**:\n\n");
                for practice in &suggestion.best_practices {
                    content.push_str(&format!("- {}\n", practice));
                }
                content.push_str("\n");
            }
        }

        content
    }
}
