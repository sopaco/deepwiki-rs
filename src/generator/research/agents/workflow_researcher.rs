use crate::generator::research::memory::MemoryScope;
use crate::generator::research::types::AgentType;
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

#[derive(Default)]
pub struct WorkflowResearcher;

impl StepForwardAgent for WorkflowResearcher {
    type Output = String; // Changed from WorkflowReport to String for text-based output

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
                DataSource::CODE_INSIGHTS,
            ],
            // Use workflow docs for business process analysis
            optional_sources: vec![DataSource::knowledge_categories(vec![
                "workflow",
                "architecture",
            ])],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"You are a professional software workflow analyst. Your task is to analyze the project's core functional workflows and generate a comprehensive workflow documentation in Markdown format.

## Mermaid Diagram Safety Rules (MUST follow):
- Always generate Mermaid that is syntactically valid in strict parsers.
- Use ASCII-only node IDs: `[A-Za-z0-9_]` (e.g. `StartProcess`, `ValidateInput`).
- Put localized/human-readable text only inside node labels.
- Use only standard diagram headers like `graph TD`, `graph LR`, `flowchart TD`, `sequenceDiagram`.
- Do not use hidden/zero-width characters, smart quotes, or unusual Unicode symbols in Mermaid code.

## External Knowledge Integration:
You may have access to existing product description, requirements and architecture documentation from external sources.
If available:
- Cross-reference code workflows with documented business processes
- Use established process terminology and flow descriptions
- Validate implementation against documented process requirements
- Identify any gaps between documented workflows and actual implementation
- Incorporate business context and rationale from the documentation

## Output Format:
Generate a Markdown document that includes:
1. Main workflow analysis with Mermaid diagrams
2. Other important workflows
3. Key insights about the system's operational patterns

Focus on functional perspective rather than excessive technical details."#.to_string(),
            opening_instruction: "The following research reports are provided for analyzing the system's main workflows".to_string(),
            closing_instruction: r#"
## Document Structure Requirements:
Please generate a comprehensive workflow documentation in Markdown format:

```markdown
# System Workflow Analysis

## 1. Main Workflow
- **Workflow Name**: [Name of the primary workflow]
- **Description**: [Detailed description of what this workflow accomplishes]
- **Flow Diagram**:
```mermaid
graph TD
    A[Start] --> B[Step 1]
    B --> C[Step 2]
    ...
```
- **Key Steps**: [List the main steps and their purposes]

## 2. Other Important Workflows
### 2.1 [Workflow Name]
- **Description**: [What this workflow does]
- **Flow Diagram**: [Mermaid diagram if applicable]

### 2.2 [Workflow Name]
...

## 3. Workflow Insights
- [Key observations about the system's operational patterns]
- [Potential optimization opportunities]
- [Dependencies between workflows]
```

If external documentation is provided:
- Validate code workflows against documented business processes
- Note any discrepancies or missing steps
- Use consistent process terminology"#.to_string(),
            llm_call_mode: LLMCallMode::Prompt, // Changed from Extract to Prompt
            formatter_config: FormatterConfig::default(),
        }
    }
}
