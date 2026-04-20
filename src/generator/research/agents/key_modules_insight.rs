use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::research::memory::MemoryRetriever;
use crate::generator::research::types::{
    AgentType, DomainModule, DomainModulesReport, KeyModuleReport, SubModule,
};
use crate::generator::{
    agent_executor::{AgentExecuteParams, extract},
    context::GeneratorContext,
    step_forward_agent::{
        AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
    },
};
use crate::types::{CodeAndDirectoryInsights, FileInsight};
use crate::utils::threads::do_parallel_with_limit;
use anyhow::{Result, anyhow};
use async_trait::async_trait;
use std::collections::HashSet;

// Research materials for domain modules
#[derive(Default, Clone)]
pub struct KeyModulesInsight;

#[async_trait]
impl StepForwardAgent for KeyModulesInsight {
    type Output = Vec<KeyModuleReport>;

    fn agent_type(&self) -> String {
        AgentType::KeyModulesInsight.to_string()
    }

    fn agent_type_enum(&self) -> Option<AgentType> {
        Some(AgentType::KeyModulesInsight)
    }

    fn memory_scope_key(&self) -> String {
        crate::generator::research::memory::MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::ResearchResult(AgentType::SystemContextResearcher.to_string()),
                DataSource::ResearchResult(AgentType::DomainModulesDetector.to_string()),
            ],
            // Use architecture and database docs for module insights
            optional_sources: vec![DataSource::knowledge_categories(vec!["architecture", "database"])],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"You are a software development expert. Based on the information provided by the user, investigate the technical details of core modules.

You may have access to existing product description, requirements and architecture documentation from external sources.
If available:
- Reference documented component responsibilities and interfaces
- Validate implementation against documented design patterns
- Use established terminology for components and modules
- Identify any gaps between documented and actual component behavior
- Incorporate design rationale and constraints from the documentation"#
                .to_string(),
            opening_instruction: "Based on the following project information and research materials, analyze the core modules:".to_string(),
            closing_instruction: "".to_string(),
            llm_call_mode: LLMCallMode::Extract,
            formatter_config: FormatterConfig::default(),
        }
    }

    // Override execute method to implement multi-domain analysis
    async fn execute(&self, context: &GeneratorContext) -> Result<Self::Output> {
        let reports = self.execute_multi_domain_analysis(context).await?;
        let value = serde_json::to_value(&reports)?;

        context
            .store_to_memory(&self.memory_scope_key(), &self.agent_type(), value.clone())
            .await?;

        Ok(reports)
    }
}

impl KeyModulesInsight {
    // Multi-domain analysis main logic
    async fn execute_multi_domain_analysis(
        &self,
        context: &GeneratorContext,
    ) -> Result<Vec<KeyModuleReport>> {
        println!("🔍 Starting multi-domain module analysis...");
        let mut reports = vec![];
        let max_parallels = context.config.llm.max_parallels;

        // 1. Get domain module data
        let domain_modules = self.get_domain_modules(context).await?;

        if domain_modules.is_empty() {
            return Err(anyhow!("No domain module data found"));
        }

        let domain_names: Vec<String> = domain_modules.iter().map(|d| d.name.clone()).collect();
        println!(
            "📋 Discovered {} domain modules: {}",
            domain_modules.len(),
            domain_names.join(", ")
        );

        // 2. Perform concurrent analysis for each domain module
        println!("🚀 Starting concurrent analysis, max parallelism: {}", max_parallels);

        // Create concurrent tasks
        let analysis_futures: Vec<_> = domain_modules
            .iter()
            .map(|domain| {
                let domain_clone = domain.clone();
                let context_clone = context.clone();
                Box::pin(async move {
                    let key_modules_insight = KeyModulesInsight::default();
                    let result = key_modules_insight
                        .analyze_single_domain(&domain_clone, &context_clone)
                        .await;
                    (domain_clone.name.clone(), result)
                })
            })
            .collect();

        // Use do_parallel_with_limit for concurrency control
        let analysis_results = do_parallel_with_limit(analysis_futures, max_parallels).await;

        // Process analysis results
        let mut successful_analyses = 0;
        for (domain_name, result) in analysis_results {
            match result {
                Ok(report) => {
                    // Store results for each domain
                    let storage_key = format!("{}_{}", self.agent_type(), domain_name);
                    context
                        .store_research(&storage_key, serde_json::to_value(&report)?)
                        .await?;
                    successful_analyses += 1;
                    reports.push(report);
                    println!("✅ Domain module analysis: {} completed and stored", domain_name);
                }
                Err(e) => {
                    let msg = context.config.target_language.msg_domain_analysis_failed();
                    println!("{}", msg.replace("{}", &domain_name).replace("{}", &e.to_string()));
                    // Continue processing other domains without interrupting the entire flow
                }
            }
        }

        if successful_analyses == 0 {
            return Err(anyhow!("All domain analyses failed"));
        }

        Ok(reports)
    }
}

impl KeyModulesInsight {
    // Get domain module data
    async fn get_domain_modules(&self, context: &GeneratorContext) -> Result<Vec<DomainModule>> {
        let domain_report = context
            .get_research(&AgentType::DomainModulesDetector.to_string())
            .await
            .ok_or_else(|| anyhow!("DomainModulesDetector result is not available"))?;

        let domain_modules_report: DomainModulesReport = serde_json::from_value(domain_report)?;
        Ok(domain_modules_report.domain_modules)
    }

    // Filter code insights related to the domain
    async fn filter_code_insights_for_domain(
        &self,
        domain: &DomainModule,
        context: &GeneratorContext,
    ) -> Result<Vec<FileInsight>> {
        let all_insights = context
            .get_from_memory::<CodeAndDirectoryInsights>(MemoryScope::PREPROCESS, ScopedKeys::CODE_INSIGHTS)
            .await
            .expect("memory of CODE_INSIGHTS not found in PREPROCESS");

        // Collect all code paths associated with this domain
        let mut domain_paths: HashSet<String> = HashSet::new();

        // 1. Add the domain's own code paths
        for path in &domain.code_paths {
            domain_paths.insert(path.clone());
        }

        // 2. Add submodule code paths
        for sub in &domain.sub_modules {
            for path in &sub.code_paths {
                domain_paths.insert(path.clone());
            }
        }

        if domain_paths.is_empty() {
            let msg = context.config.target_language.msg_no_code_path_for_domain();
            println!("{}", msg.replace("{}", &domain.name));
            return Ok(Vec::new());
        }

        // Flatten all file_insights from directory_dossiers
        let all_files: Vec<FileInsight> = all_insights
            .directory_insights
            .iter()
            .flat_map(|d| d.file_insights.iter())
            .filter(|fi| {
                let file_path = fi.file_path.to_string_lossy();
                let file_path = file_path.replace('\\', "/");
                domain_paths.iter().any(|path| {
                    let path = path.replace('\\', "/");
                    file_path.contains(&path) || path.contains(&file_path)
                })
            })
            .take(50)
            .cloned()
            .collect();

        println!(
            "📁 Filtered {} related code files for domain '{}'",
            all_files.len(),
            domain.name
        );
        Ok(all_files)
    }

    // Execute analysis for a single domain module
    async fn analyze_single_domain(
        &self,
        domain: &DomainModule,
        context: &GeneratorContext,
    ) -> Result<KeyModuleReport> {
        // 1. Filter code insights related to this domain
        let filtered_insights = self
            .filter_code_insights_for_domain(domain, context)
            .await?;

        // 2. Build domain-specific prompt
        let (system_prompt, user_prompt) = self.build_domain_prompt(domain, &filtered_insights);

        // 3. Use agent_executor::extract for analysis
        let params = AgentExecuteParams {
            prompt_sys: system_prompt,
            prompt_user: user_prompt,
            cache_scope: format!(
                "{}/{}/{}",
                crate::generator::research::memory::MemoryScope::STUDIES_RESEARCH.to_string(),
                self.agent_type(),
                domain.name
            ),
            log_tag: format!("{} domain analysis", domain.name),
            progress: None,
        };

        println!("🤖 Analyzing '{}' domain...", domain.name);
        let mut report: KeyModuleReport = extract(context, params).await?;

        // 4. Set domain context information
        report.domain_name = domain.name.clone();
        if report.module_name.is_empty() {
            report.module_name = format!("{} Core Module", domain.name);
        }

        println!("✅ '{}' domain analysis completed", domain.name);
        Ok(report)
    }

    // Build domain-specific prompt
    fn build_domain_prompt(
        &self,
        domain: &DomainModule,
        insights: &[FileInsight],
    ) -> (String, String) {
        let system_prompt =
            "Based on the provided domain and code insights, conduct in-depth analysis and return strict JSON only.

Output requirements (no markdown, no code fences, no prose outside JSON):
{
  \"domain_name\": \"string\",
  \"module_name\": \"string\",
  \"module_description\": \"string\",
  \"interaction\": \"string\",
  \"implementation\": \"string\",
  \"associated_files\": [\"string\"],
  \"flowchart_mermaid\": \"string\",
  \"sequence_diagram_mermaid\": \"string\"
}

Rules:
- Include all fields every time.
- Use plain strings for all textual fields.
- associated_files must be an array of strings.
- If uncertain, use empty strings/empty array.
- Mermaid fields must be valid mermaid text or empty string.
"
            .to_string();

        let user_prompt = format!(
            "## Domain Analysis Task\nAnalyze the core module technical details of the '{}' domain\n\n### Domain Information\n- Domain Name: {}\n- Domain Type: {}\n- Importance: {:.1}/10\n- Complexity: {:.1}/10\n- Description: {}\n\n### Submodule Overview\n{}\n\n### Related Code Insights\n{}\n",
            domain.name,
            domain.name,
            domain.domain_type,
            domain.importance,
            domain.complexity,
            domain.description,
            self.format_sub_modules(&domain.sub_modules),
            self.format_filtered_insights(insights)
        );

        (system_prompt, user_prompt)
    }

    // Format submodule information
    fn format_sub_modules(&self, sub_modules: &[SubModule]) -> String {
        if sub_modules.is_empty() {
            return "No submodule information available".to_string();
        }

        sub_modules.iter()
            .enumerate()
            .map(|(i, sub)| format!(
                "{}. **{}**\n   - Description: {}\n   - Importance: {:.1}/10\n   - Core Functions: {}\n   - Code Files: {}",
                i + 1,
                sub.name,
                sub.description,
                sub.importance,
                sub.key_functions.join(", "),
                sub.code_paths.join(", ")
            ))
            .collect::<Vec<_>>()
            .join("\n\n")
    }

    // Format filtered code insights
    fn format_filtered_insights(&self, insights: &[FileInsight]) -> String {
        if insights.is_empty() {
            return "No related code insights available".to_string();
        }

        insights
            .iter()
            .enumerate()
            .map(|(i, fi)| {
                format!(
                    "{}. File `{}`, Purpose: {}\n   Description: {}\n   Source Code\n```code\n{}```\n---\n",
                    i + 1,
                    fi.file_path.to_string_lossy(),
                    fi.code_purpose,
                    fi.summary,
                    fi.source_summary
                )
            })
            .collect::<Vec<_>>()
            .join("\n")
    }
}
