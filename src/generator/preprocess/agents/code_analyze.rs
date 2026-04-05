use crate::generator::agent_executor::{AgentExecuteParams, extract};
use crate::{
    generator::{
        context::GeneratorContext,
        preprocess::extractors::language_processors::LanguageProcessorManager,
    },
    types::{
        code::{CodeDossier, CodeInsight, CodeInsightLLMOutput},
        project_structure::ProjectStructure,
    },
    utils::{sources::read_dependency_code_source, threads::do_parallel_with_limit},
};
use anyhow::Result;

pub struct CodeAnalyze {
    language_processor: LanguageProcessorManager,
}

impl CodeAnalyze {
    pub fn new() -> Self {
        Self {
            language_processor: LanguageProcessorManager::new(),
        }
    }

    pub async fn execute(
        &self,
        context: &GeneratorContext,
        codes: &Vec<CodeDossier>,
        project_structure: &ProjectStructure,
    ) -> Result<Vec<CodeInsight>> {
        let max_parallels = context.config.llm.max_parallels;

        // Create concurrent tasks
        let analysis_futures: Vec<_> = codes
            .iter()
            .map(|code| {
                let code_clone = code.clone();
                let context_clone = context.clone();
                let project_structure_clone = project_structure.clone();
                let language_processor = self.language_processor.clone();

                Box::pin(async move {
                    let code_analyze = CodeAnalyze { language_processor };
                    let (agent_params, mut static_insight) = code_analyze
                        .prepare_single_code_agent_params(&project_structure_clone, &code_clone)
                        .await?;
                    static_insight.code_dossier.source_summary = code_clone.source_summary.to_owned();

                    // Use simplified LLM output structure for better reliability
                    let llm_output = match extract::<CodeInsightLLMOutput>(&context_clone, agent_params).await {
                        Ok(output) => output,
                        Err(e) => {
                            eprintln!(
                                "⚠️ AI code insight failed for {}: {}. Falling back to static analysis.",
                                code_clone.name, e
                            );
                            return Result::<CodeInsight>::Ok(static_insight);
                        }
                    };

                    // Merge LLM output into static insight
                    static_insight.merge_llm_output(llm_output);
                    static_insight.code_dossier.source_summary = code_clone.source_summary.to_owned();
                    Result::<CodeInsight>::Ok(static_insight)
                })
            })
            .collect();

        // Use do_parallel_with_limit for concurrency control
        let analysis_results = do_parallel_with_limit(analysis_futures, max_parallels).await;

        // Process analysis results
        let mut code_insights = Vec::new();
        for result in analysis_results {
            match result {
                Ok(code_insight) => {
                    code_insights.push(code_insight);
                }
                Err(e) => {
                    eprintln!("❌ Code analysis failed: {}", e);
                    return Err(e);
                }
            }
        }

        println!(
            "✓ Concurrent code analysis completed, successfully analyzed {} files",
            code_insights.len()
        );
        Ok(code_insights)
    }
}

impl CodeAnalyze {
    async fn prepare_single_code_agent_params(
        &self,
        project_structure: &ProjectStructure,
        codes: &CodeDossier,
    ) -> Result<(AgentExecuteParams, CodeInsight)> {
        // First perform static analysis
        let code_analyse = self.analyze_code_by_rules(codes, project_structure).await?;

        // Then use AI for enhanced analysis
        let prompt_user = self.build_code_analysis_prompt(project_structure, &code_analyse);
        let prompt_sys = include_str!("prompts/code_analyze_sys.tpl").to_string();

        Ok((
            AgentExecuteParams {
                prompt_sys,
                prompt_user,
                cache_scope: "ai_code_insight".to_string(),
                log_tag: codes.name.to_string(),
            },
            code_analyse,
        ))
    }
}

impl CodeAnalyze {
    fn build_code_analysis_prompt(
        &self,
        project_structure: &ProjectStructure,
        analysis: &CodeInsight,
    ) -> String {
        let project_path = &project_structure.root_path;

        // Read source code snippets of dependency components
        let dependency_code =
            read_dependency_code_source(&self.language_processor, analysis, project_path);

        format!(
            include_str!("prompts/code_analyze_user.tpl"),
            analysis.code_dossier.name,
            analysis.code_dossier.file_path.display(),
            analysis.code_dossier.code_purpose.display_name(),
            analysis.code_dossier.importance_score,
            analysis.responsibilities.join(", "),
            analysis.interfaces.len(),
            analysis.dependencies.len(),
            analysis.complexity_metrics.lines_of_code,
            analysis.complexity_metrics.cyclomatic_complexity,
            analysis.code_dossier.source_summary,
            dependency_code
        )
    }

    async fn analyze_code_by_rules(
        &self,
        code: &CodeDossier,
        project_structure: &ProjectStructure,
    ) -> Result<CodeInsight> {
        let full_path = project_structure.root_path.join(&code.file_path);

        // Read file content
        let content = if full_path.exists() {
            tokio::fs::read_to_string(&full_path).await?
        } else {
            String::new()
        };

        // Analyze interfaces
        let interfaces = self
            .language_processor
            .extract_interfaces(&code.file_path, &content);

        // Analyze dependencies
        let dependencies = self
            .language_processor
            .extract_dependencies(&code.file_path, &content);

        // Calculate complexity metrics
        let complexity_metrics = self
            .language_processor
            .calculate_complexity_metrics(&content);

        Ok(CodeInsight {
            code_dossier: code.clone(),
            detailed_description: format!("Detailed analysis of {}", code.name),
            interfaces,
            dependencies,
            complexity_metrics,
            responsibilities: vec![],
        })
    }
}
