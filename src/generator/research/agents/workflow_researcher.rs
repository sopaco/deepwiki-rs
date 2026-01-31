use crate::generator::{
    {
        step_forward_agent::{StepForwardAgent, AgentDataConfig, DataSource, PromptTemplate, LLMCallMode, FormatterConfig},
    },
};
use crate::generator::research::memory::MemoryScope;
use crate::generator::research::types::{AgentType, WorkflowReport};

#[derive(Default)]
pub struct WorkflowResearcher;

impl StepForwardAgent for WorkflowResearcher {
    type Output = WorkflowReport;
    
    fn agent_type(&self) -> String {
        AgentType::WorkflowResearcher.to_string()
    }

    fn agent_type_enum(&self) -> Option<AgentType> {
        Some(AgentType::WorkflowResearcher)
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::ResearchResult(AgentType::SystemContextResearcher.to_string()),
                DataSource::ResearchResult(AgentType::DomainModulesDetector.to_string()),
                DataSource::CODE_INSIGHTS
            ],
            // Use workflow docs for business process analysis
            optional_sources: vec![DataSource::knowledge_categories(vec!["workflow", "architecture"])],
        }
    }
    
    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"Analyze the project's core functional workflows, focusing from a functional perspective without being limited to excessive technical details.

You may have access to existing product description, requirements and architecture documentation from external sources.
If available:
- Cross-reference code workflows with documented business processes
- Use established process terminology and flow descriptions
- Validate implementation against documented process requirements
- Identify any gaps between documented workflows and actual implementation
- Incorporate business context and rationale from the documentation"#.to_string(),
            opening_instruction: "The following research reports are provided for analyzing the system's main workflows".to_string(),
            closing_instruction: r#"Please analyze the system's core workflows based on the research materials.

If external documentation is provided:
- Validate code workflows against documented business processes
- Note any discrepancies or missing steps
- Use consistent process terminology"#.to_string(),
            llm_call_mode: LLMCallMode::Extract,
            formatter_config: FormatterConfig::default(),
        }
    }
}