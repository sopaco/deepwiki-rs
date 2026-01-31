use crate::generator::research::memory::MemoryScope;
use crate::generator::research::types::AgentType;
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

/// Architecture Researcher - Responsible for analyzing the overall architecture of the project
#[derive(Default)]
pub struct ArchitectureResearcher;

impl StepForwardAgent for ArchitectureResearcher {
    type Output = String; // Returns text result

    fn agent_type(&self) -> String {
        AgentType::ArchitectureResearcher.to_string()
    }

    fn agent_type_enum(&self) -> Option<AgentType> {
        Some(AgentType::ArchitectureResearcher)
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::ResearchResult(AgentType::SystemContextResearcher.to_string()),
                DataSource::ResearchResult(AgentType::DomainModulesDetector.to_string()),
            ],
            optional_sources: vec![
                DataSource::PROJECT_STRUCTURE,
                DataSource::DEPENDENCY_ANALYSIS,
                // Use architecture, deployment, database and ADR docs for architecture analysis
                DataSource::knowledge_categories(vec!["architecture", "deployment", "database", "adr"]),
            ],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt:
                r#"You are a professional software architecture analyst, analyze system architecture based on research reports, output project architecture research documentation.

You may have access to existing product description, requirements and architecture documentation from external sources.
If available:
- Validate code structure against documented architecture patterns
- Cross-reference implementation with architectural decision records (ADRs)
- Identify gaps between documented design and actual implementation
- Incorporate established architectural principles and patterns from the documentation
- Note any inconsistencies that should be addressed"#
                    .to_string(),

            opening_instruction: "The following research reports are provided for analyzing the system architecture:".to_string(),

            closing_instruction: r#"
## Analysis Requirements:
- Draw system architecture diagram based on the provided project information and research materials
- Use mermaid format to represent architecture relationships
- Highlight core components and interaction patterns
- If external documentation is provided, validate implementation against documented architecture
- Identify any architectural drift or gaps between documentation and code"#
                .to_string(),

            llm_call_mode: LLMCallMode::PromptWithTools, // Use prompt mode
            formatter_config: FormatterConfig::default(),
        }
    }
}
