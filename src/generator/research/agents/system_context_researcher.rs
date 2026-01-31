use crate::generator::{
    step_forward_agent::{
        AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
    }
};
use crate::generator::research::memory::MemoryScope;
use crate::generator::research::types::{AgentType, SystemContextReport};

/// Project Objective Researcher - Responsible for analyzing the project's core objectives, functional value, and system boundaries
#[derive(Default)]
pub struct SystemContextResearcher;

impl StepForwardAgent for SystemContextResearcher {
    type Output = SystemContextReport;

    fn agent_type(&self) -> String {
        AgentType::SystemContextResearcher.to_string()
    }

    fn agent_type_enum(&self) -> Option<AgentType> {
        Some(AgentType::SystemContextResearcher)
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![DataSource::PROJECT_STRUCTURE, DataSource::CODE_INSIGHTS],
            optional_sources: vec![
                DataSource::README_CONTENT,
                // Use architecture and ADR docs for system context analysis
                DataSource::knowledge_categories(vec!["architecture", "adr"]),
            ],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"You are a professional software architecture analyst, specializing in project objective and system boundary analysis.

Analyze the project to determine:
1. Core objectives and business value
2. Project type and tech stack
3. Target users and use cases
4. External system dependencies
5. System boundaries (what's in/out of scope)

When external documentation is provided:
- Cross-reference code against documented architecture
- Flag gaps between docs and implementation
- Use established business terminology

Rrequired output style (extremely important):
- Plain English, short sentences
- No filler phrases ("it is important to note", "in order to")
- No repetition - state each point once
- Concrete specifics over vague generalities
- If uncertain, say so briefly rather than padding

Generate Output as JSON per existing schema."#
                .to_string(),

            opening_instruction: "Based on the following research materials, analyze the project's core objectives and system positioning:".to_string(),

            closing_instruction: r#"
## Analysis Requirements:
- Accurately identify project type and technical characteristics
- Clearly define target users and usage scenarios
- Clearly delineate system boundaries
- If external documentation is provided, validate code structure against it
- Identify any gaps between documented architecture and actual implementation
- Ensure analysis results conform to the C4 architecture model's system context level"#
                .to_string(),

            llm_call_mode: LLMCallMode::Extract,
            formatter_config: FormatterConfig::default(),
        }
    }
}
