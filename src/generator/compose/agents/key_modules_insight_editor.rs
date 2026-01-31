use crate::generator::compose::memory::MemoryScope;
use crate::generator::context::GeneratorContext;
use crate::generator::outlet::DocTree;
use crate::generator::research::memory::MemoryRetriever;
use crate::generator::research::types::{AgentType as ResearchAgentType, KeyModuleReport};
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
};
use crate::utils::threads::do_parallel_with_limit;
use anyhow::Result;

#[derive(Default)]
pub struct KeyModulesInsightEditor {}

impl KeyModulesInsightEditor {
    pub async fn execute(&self, context: &GeneratorContext, doc_tree: &mut DocTree) -> Result<()> {
        if let Some(value) = context
            .get_research(&ResearchAgentType::KeyModulesInsight.to_string())
            .await
        {
            let insight_reports: Vec<KeyModuleReport> = serde_json::from_value(value)?;
            let max_parallels = context.config.llm.max_parallels;

            println!(
                "🚀 Starting concurrent analysis of insight reports, max concurrency: {}",
                max_parallels
            );

            // Create concurrent tasks
            let analysis_futures: Vec<_> = insight_reports
                .into_iter()
                .map(|insight_report| {
                    let insight_key = format!(
                        "{}_{}",
                        ResearchAgentType::KeyModulesInsight,
                        &insight_report.domain_name
                    );
                    let domain_name = insight_report.domain_name.clone();
                    let kmie = KeyModuleInsightEditor::new(insight_key.clone(), insight_report);
                    let context_clone = context.clone();

                    Box::pin(async move {
                        let result = kmie.execute(&context_clone).await;
                        (insight_key, domain_name, result)
                    })
                })
                .collect();

            // Use do_parallel_with_limit for concurrency control
            let analysis_results = do_parallel_with_limit(analysis_futures, max_parallels).await;

            // Process results and update doc_tree
            for (insight_key, domain_name, result) in analysis_results {
                result?; // Check for errors

                doc_tree.insert(
                    &insight_key,
                    format!("{}/{}.md", context.config.target_language.get_directory_name("deep_exploration"), &domain_name).as_str(),
                );
            }
        }

        Ok(())
    }
}

struct KeyModuleInsightEditor {
    insight_key: String,
    report: KeyModuleReport,
}

impl KeyModuleInsightEditor {
    fn new(insight_key: String, report: KeyModuleReport) -> Self {
        KeyModuleInsightEditor {
            insight_key,
            report,
        }
    }
}

impl StepForwardAgent for KeyModuleInsightEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        self.insight_key.to_string()
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
                DataSource::ResearchResult(ResearchAgentType::ArchitectureResearcher.to_string()),
                DataSource::ResearchResult(ResearchAgentType::WorkflowResearcher.to_string()),
                DataSource::ResearchResult(self.insight_key.to_string()),
            ],
            // Use architecture and database docs for key module documentation
            optional_sources: vec![DataSource::knowledge_categories(vec!["architecture", "database"])],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        let report = &self.report;
        let opening_instruction = format!(
            r#"The topic you need to analyze is: {}
            ## Documentation Quality Requirements:
            1. **Completeness**: Based on research materials, cover all important aspects of the topic `{}`, without omitting key information
            2. **Accuracy**: Based on research data, ensure accuracy of technical details
            3. **Professionalism**: Use standard architecture terminology and expressions
            4. **Readability**: Clear structure, rich language narrative, and easy to understand
            5. **Practicality**: Provide valuable module knowledge and technical implementation details.
            "#,
            &report.domain_name, &report.domain_name
        );

        PromptTemplate {
            system_prompt: r#"You are a software expert skilled at writing technical documentation. Based on the research materials and requirements provided by users, write technical documentation for the technical implementation of corresponding modules in existing projects"#.to_string(),

            opening_instruction,

            closing_instruction: String::new(),

            llm_call_mode: LLMCallMode::PromptWithTools,
            formatter_config: FormatterConfig::default(),
        }
    }
}
