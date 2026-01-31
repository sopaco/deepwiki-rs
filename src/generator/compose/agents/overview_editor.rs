use crate::generator::compose::memory::MemoryScope;
use crate::generator::compose::types::AgentType;
use crate::generator::research::types::AgentType as ResearchAgentType;
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

#[derive(Default)]
pub struct OverviewEditor;

impl StepForwardAgent for OverviewEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        AgentType::Overview.to_string()
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::DOCUMENTATION.to_string()
    }

    fn should_include_timestamp(&self) -> bool {
        true
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::ResearchResult(ResearchAgentType::SystemContextResearcher.to_string()),
                DataSource::ResearchResult(ResearchAgentType::DomainModulesDetector.to_string()),
            ],
            optional_sources: vec![
                DataSource::README_CONTENT,
                // Use architecture and ADR docs for overview
                DataSource::knowledge_categories(vec!["architecture", "adr"]),
            ],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"You are a professional software architecture documentation expert, focused on generating C4 architecture model SystemContext level documentation.

Your task is to write a complete, in-depth, detailed, and easy-to-read C4 SystemContext document titled `Project Overview` based on the provided system context research report and domain module analysis results.

## External Knowledge Integration:
You may have access to existing product description, requirements and architecture documentation from external sources.
If available:
- Incorporate established business context and objectives
- Reference documented stakeholders and user personas
- Use documented terminology for systems and integrations
- Validate implementation against documented system boundaries
- Highlight any scope changes or undocumented features

## C4 SystemContext Documentation Requirements:
1. **System Overview**: Clearly describe the system's core objectives, business value, and technical characteristics
2. **User Roles**: Clearly define target user groups and usage scenarios
3. **System Boundaries**: Accurately delineate system scope, clearly stating included and excluded components
4. **External Interactions**: Detail interactions and dependencies with external systems
5. **Architecture View**: Provide clear system context diagrams and key information

## Document Structure Requirements:
- Include appropriate heading levels and chapter organization
- Provide clear diagrams and visual content
- Ensure content logic is clear and expression is accurate
- Maintain consistency with external documentation when available"#.to_string(),

            opening_instruction: r#"Based on the following research materials, write a complete, in-depth, and detailed C4 SystemContext architecture document:

## Writing Guidelines:
1. First analyze the system context research report and extract core information
2. Combine domain module analysis results to understand the internal system structure
3. Organize content according to C4 model SystemContext level requirements
4. Ensure document content accurately reflects the actual system situation"#.to_string(),

            closing_instruction: r#"
## Output Requirements:
1. **Completeness**: Ensure coverage of all key elements of C4 SystemContext
2. **Accuracy**: Based on research data, avoid subjective speculation and inaccurate information
3. **Professionalism**: Use professional architecture terminology and expression
4. **Readability**: Clear structure, easy for both technical teams and business personnel to understand
5. **Practicality**: Provide valuable architecture insights and guidance

## Document Format:
- Include necessary diagram descriptions (such as Mermaid diagrams)
- Maintain logical and hierarchical chapter structure
- Ensure content completeness and coherence

## Recommended Document Structure:
```sample
# System Context Overview

## 1. Project Introduction
- Project name and description
- Core functionality and value
- Technical characteristics overview

## 2. Target Users
- User role definitions
- Usage scenario descriptions
- User requirement analysis

## 3. System Boundaries
- System scope definition
- Included core components
- Excluded external dependencies

## 4. External System Interactions
- External system list
- Interaction method descriptions
- Dependency relationship analysis

## 5. System Context Diagram
- C4 SystemContext diagram
- Key interaction flows
- Architecture decision descriptions

## 6. Technical Architecture Overview
- Main technology stack
- Architecture patterns
- Key design decisions
```

Please generate a high-quality C4 SystemContext architecture document."#.to_string(),

            llm_call_mode: LLMCallMode::Prompt,
            formatter_config: FormatterConfig::default(),
        }
    }
}
