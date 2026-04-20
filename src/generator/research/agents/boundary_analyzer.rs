use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::research::types::{AgentType, BoundaryAnalysisReport};
use crate::generator::{
    context::GeneratorContext,
    step_forward_agent::{
        AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
    },
};
use crate::types::code::CodePurpose;
use crate::types::{CodeAndDirectoryInsights, FileInsight};
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
                r#"You are a professional system boundary interface analyst. Your task is to identify and analyze external call boundaries of software systems.

## What to Look For:

### CLI Commands (cli_boundaries)
Look in Entry-type files for:
- Command-line argument parsing (e.g., argparse, commander, clap, yargs)
- Main function parameters
- Process.argv usage
- Environment variable reading
- Configuration file loading
- Any program startup options

### API Interfaces (api_boundaries)  
Look in Api/Controller-type files for:
- HTTP route handlers
- REST endpoints
- GraphQL resolvers
- RPC method definitions
- Webhook handlers

### Router Routes (router_boundaries)
Look in Router-type files for:
- URL path definitions
- Route parameters
- Page routing logic
- Middleware chains

### Configuration (can be documented as CLI or Integration)
Look in Config-type files for:
- Configuration parameters
- Environment variables
- Feature flags
- Startup options

## Important:
- Even if code doesn't have explicit CLI/API definitions, extract what you can from entry points and config files
- Document how users interact with the system (command line, config files, etc.)
- If you find configuration parameters, document them as CLI boundaries or integration suggestions
- NEVER leave all arrays empty if you have Entry or Config code - at minimum document the startup/configuration interface

You MUST return a valid JSON object:
{
  "cli_boundaries": [...],
  "api_boundaries": [...],
  "router_boundaries": [...],
  "integration_suggestions": [...],
  "confidence_score": 0.0
}

Rules:
- Include all top-level keys
- Use empty arrays only if truly no boundaries exist
- confidence_score: 0.0-10.0"#
                    .to_string(),

            opening_instruction: "Analyze the system's boundary interfaces based on the following code:".to_string(),

            closing_instruction: r#"
## Analysis Instructions:
1. **Entry files**: Look for CLI arguments, environment variables, config loading - these ARE boundaries!
2. **Config files**: Document configuration parameters as CLI boundaries or integration suggestions
3. **No API/Router code?** That's fine - focus on CLI/configuration interfaces
4. **Minimum output**: If you have Entry/Config code, document at least the startup interface

DO NOT return all empty arrays if you have Entry or Config code to analyze!"#
                .to_string(),

            llm_call_mode: LLMCallMode::Extract,

            formatter_config: FormatterConfig::default(),
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
    ) -> Result<Vec<FileInsight>> {
        let all_insights = context
            .get_from_memory::<CodeAndDirectoryInsights>(MemoryScope::PREPROCESS, ScopedKeys::CODE_INSIGHTS)
            .await
            .ok_or_else(|| anyhow!("CODE_INSIGHTS not found in PREPROCESS memory"))?;

        // Flatten all file_insights from directory_dossiers and filter by boundary-related purpose
        let boundary_insights: Vec<FileInsight> = all_insights
            .directory_insights
            .iter()
            .flat_map(|d| d.file_insights.iter())
            .filter(|fi| {
                matches!(
                    fi.code_purpose,
                    CodePurpose::Entry
                        | CodePurpose::Api
                        | CodePurpose::Config
                        | CodePurpose::Router
                        | CodePurpose::Controller
                )
            })
            .cloned()
            .collect();

        // Group by type and count
        let mut entry_count = 0;
        let mut api_count = 0;
        let mut config_count = 0;
        let mut router_count = 0;

        for fi in &boundary_insights {
            match fi.code_purpose {
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

        Ok(boundary_insights)
    }

    /// Format boundary code insights - specialized formatting logic
    fn format_boundary_insights(&self, insights: &[FileInsight]) -> String {
        let mut content = String::from("### Boundary-Related Code Insights\n");

        // Group by CodePurpose for display
        let mut entry_codes = Vec::new();
        let mut api_codes = Vec::new();
        let mut config_codes = Vec::new();
        let mut router_codes = Vec::new();

        for fi in insights {
            match fi.code_purpose {
                CodePurpose::Entry => entry_codes.push(fi),
                CodePurpose::Api => api_codes.push(fi),
                CodePurpose::Controller => api_codes.push(fi),
                CodePurpose::Config => config_codes.push(fi),
                CodePurpose::Router => router_codes.push(fi),
                _ => {}
            }
        }

        if !entry_codes.is_empty() {
            content.push_str("#### Entry Point Code (Entry)\n");
            content.push_str("These code usually contain CLI command definitions, main function entry points, etc.:\n\n");
            for fi in entry_codes {
                self.add_boundary_insight_item(&mut content, fi);
            }
        }

        if !api_codes.is_empty() {
            content.push_str("#### API/Controller Code (API/Controller)\n");
            content.push_str("These code usually contain HTTP endpoints, API routes, controller logic, etc.:\n\n");
            for fi in api_codes {
                self.add_boundary_insight_item(&mut content, fi);
            }
        }

        if !config_codes.is_empty() {
            content.push_str("#### Configuration-Related Code (Config)\n");
            content.push_str("These code usually contain configuration structures, parameter definitions, environment variables, etc.:\n\n");
            for fi in config_codes {
                self.add_boundary_insight_item(&mut content, fi);
            }
        }

        if !router_codes.is_empty() {
            content.push_str("#### Router-Related Code (Router)\n");
            content.push_str("These code usually contain route definitions, middleware, request handling, etc.:\n\n");
            for fi in router_codes {
                self.add_boundary_insight_item(&mut content, fi);
            }
        }

        content.push_str("\n");
        content
    }

    /// Add single boundary code insight item with full context
    fn add_boundary_insight_item(&self, content: &mut String, fi: &FileInsight) {
        content.push_str(&format!(
            "**File**: `{}` (Importance: {:.2}, Purpose: {:?})\n",
            fi.file_path.to_string_lossy(),
            fi.importance_score,
            fi.code_purpose
        ));

        if !fi.detailed_description.is_empty() {
            content.push_str(&format!("- **Description**: {}\n", fi.detailed_description));
        } else if !fi.summary.is_empty() {
            content.push_str(&format!("- **Description**: {}\n", fi.summary));
        }

        if !fi.responsibilities.is_empty() {
            content.push_str(&format!("- **Responsibilities**: {}\n", fi.responsibilities.join(", ")));
        }

        // Include detailed interface information for CLI/API extraction
        if !fi.interfaces.is_empty() {
            content.push_str("- **Interfaces/Functions**:\n");
            for interface in &fi.interfaces {
                content.push_str(&format!("  - `{}` ({})", interface.name, interface.interface_type));
                if !interface.parameters.is_empty() {
                    let params: Vec<String> = interface.parameters.iter()
                        .map(|p| format!("{}: {}", p.name, p.param_type))
                        .collect();
                    content.push_str(&format!("({})", params.join(", ")));
                }
                if let Some(ref ret) = interface.return_type {
                    content.push_str(&format!(" -> {}", ret));
                }
                content.push_str("\n");
            }
        }

        // Include dependencies for understanding external integrations
        if !fi.dependencies.is_empty() {
            content.push_str("- **Key Dependencies**: ");
            let dep_names: Vec<&str> = fi.dependencies.iter()
                .filter(|d| d.is_external)
                .map(|d| d.name.as_str())
                .take(10)
                .collect();
            content.push_str(&format!("{}\n", dep_names.join(", ")));
        }

        // Always include source summary for boundary analysis
        if !fi.source_summary.is_empty() {
            content.push_str(&format!(
                "- **Source Code**:\n```\n{}\n```\n",
                fi.source_summary
            ));
        }

        content.push_str("\n");
    }
}
