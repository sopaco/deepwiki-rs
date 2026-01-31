use crate::generator::compose::memory::MemoryScope;
use crate::generator::compose::types::AgentType;
use crate::generator::research::types::AgentType as ResearchAgentType;
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};

#[derive(Default)]
pub struct WorkflowEditor;

impl StepForwardAgent for WorkflowEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        AgentType::Workflow.to_string()
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
                DataSource::ResearchResult(ResearchAgentType::WorkflowResearcher.to_string()),
                DataSource::CODE_INSIGHTS,
            ],
            // Use workflow docs for workflow documentation
            optional_sources: vec![DataSource::knowledge_categories(vec!["workflow", "architecture"])],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"You are a professional software architecture documentation expert, focused on analyzing and writing system core workflow documentation.

Your task is to write a complete, in-depth, and detailed workflow document titled `Core Workflows` based on the provided multi-dimensional research analysis results.

## Your Professional Capabilities:
1. **Workflow Analysis Skills**: Deep understanding of system core workflows, business processes, and technical processes
2. **Process Visualization Skills**: Proficient in flowchart design, sequence diagrams, and workflow diagram design
3. **System Insight Skills**: Identify key execution paths, process nodes, and system coordination mechanisms
4. **Technical Documentation Skills**: Express complex workflows in a clear and understandable manner

## External Knowledge Integration:
You may have access to existing product description, requirements and architecture documentation from external sources.
If available:
- Incorporate documented business process flows and terminology
- Cross-reference code workflows with documented business requirements
- Highlight any gaps between documented processes and implementation
- Use established process naming conventions and descriptions
- Reference documented process owners and stakeholders
- Validate implementation completeness against documented requirements

## Workflow Documentation Standards:
You need to generate complete workflow documentation that meets both business and technical requirements, including:
- **Main Process Overview**: System core workflows and key execution paths
- **Key Process Details**: Detailed descriptions of important business and technical processes
- **Process Coordination Mechanisms**: Inter-module coordination, data flow, and state management
- **Exception Handling Processes**: Error handling, recovery mechanisms, and fault tolerance strategies
- **Performance Optimization Processes**: Concurrent processing, resource management, and optimization strategies

## Documentation Quality Requirements:
1. **Completeness**: Cover all core workflows of the system without missing key steps
2. **Accuracy**: Based on research data, ensure accuracy and executability of process descriptions
3. **Professionalism**: Use standard process analysis terminology and expressions
4. **Readability**: Clear structure, rich narrative language, easy to understand and execute
5. **Practicality**: Provide valuable process guidance and operational details
6. **Alignment**: Maintain consistency with external business process documentation when available"#.to_string(),

            opening_instruction: r#"Based on the following comprehensive research materials, write a complete, in-depth, and detailed system core workflow document. Please carefully analyze all provided research reports and extract key workflow information:

## Analysis Guidance:
1. **System Context Analysis**: Understand the overall positioning, core value, and business boundaries of the system
2. **Domain Module Analysis**: Identify responsibility divisions of functional domains and inter-module collaboration relationships
3. **Workflow Analysis**: Deeply understand the system's main workflows and key execution paths
4. **Code Insight Analysis**: Combine code implementation details to understand technical processes and execution mechanisms
5. **Process Optimization Analysis**: Identify performance bottlenecks, concurrent processing, and resource management strategies

## Research Materials Description:
The system will automatically provide you with the following research materials:
- **System Context Research Report**: Project overview, user roles, system boundaries, and external interactions
- **Domain Module Research Report**: Functional domain divisions, module relationships, business processes, and architectural design
- **Workflow Research Report**: Core workflows, execution paths, flowcharts, and key nodes
- **Code Insight Data**: Core component implementations, technical details, dependencies, and performance characteristics

Please synthesize these research materials and focus on the following aspects of workflows:
- Execution order and dependencies of main workflows
- Inputs, outputs, and state transitions of key process nodes
- Exception handling mechanisms and recovery strategies
- Implementation of concurrent processing and performance optimization"#.to_string(),

            closing_instruction: r#"
## Output Requirements:
Please generate a high-quality core workflow document ensuring:

### 1. Complete Document Structure
```
# Core Workflows

## 1. Workflow Overview
- System main workflows
- Core execution paths
- Key process nodes
- Process coordination mechanisms

## 2. Main Workflows
- Core business process details
- Key technical process descriptions
- Process execution order and dependencies
- Input/output data flows

## 3. Flow Coordination and Control
- Multi-module coordination mechanisms
- State management and synchronization
- Data passing and sharing
- Execution control and scheduling

## 4. Exception Handling and Recovery
- Error detection and handling
- Exception recovery mechanisms
- Fault tolerance strategy design
- Failure retry and degradation

## 5. Key Process Implementation
- Core algorithm processes
- Data processing pipelines
- Business rule execution
- Technical implementation details
```

### 2. Content Quality Standards
- **Process Depth**: In-depth analysis of execution details and implementation mechanisms of each key process
- **Business Understanding**: Accurate understanding of business requirements and functional process value
- **Technical Insight**: Provide valuable technical process analysis and optimization suggestions
- **Operability**: Ensure process descriptions are executable and provide guidance

### 3. Diagram Requirements
- Use Mermaid format to draw core workflow diagrams
- Include main process diagrams, key subprocess diagrams, state transition diagrams
- Draw data flow diagrams and module interaction sequence diagrams
- Ensure diagrams are clear, accurate, and easy to understand

### 4. Professional Expression
- Use standard process analysis and business process terminology
- Maintain accuracy and professionalism in technical expression
- Provide clear logical structure and execution order
- Ensure content completeness and coherence

### 5. Practical Value Requirements
- **Development Guidance**: Provide clear process implementation guidance for development teams
- **Operations Support**: Provide process monitoring and troubleshooting guidance for operations teams
- **Business Value**: Clarify the business value and importance of each process step
- **Knowledge Transfer**: Facilitate quick understanding of system workflows for new team members

Please generate a high-quality and detailed core workflow documentation based on the research materials that meets the above requirements."#.to_string(),

            llm_call_mode: LLMCallMode::PromptWithTools,
            formatter_config: FormatterConfig::default(),
        }
    }
}
