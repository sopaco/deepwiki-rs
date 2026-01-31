use anyhow::Result;

use crate::generator::context::GeneratorContext;
use crate::generator::research::agents::architecture_researcher::ArchitectureResearcher;
use crate::generator::research::agents::boundary_analyzer::BoundaryAnalyzer;
use crate::generator::research::agents::database_overview_analyzer::DatabaseOverviewAnalyzer;
use crate::generator::research::agents::domain_modules_detector::DomainModulesDetector;
use crate::generator::research::agents::key_modules_insight::KeyModulesInsight;
use crate::generator::research::agents::system_context_researcher::SystemContextResearcher;
use crate::generator::research::agents::workflow_researcher::WorkflowResearcher;
use crate::generator::step_forward_agent::StepForwardAgent;
use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::types::code::{CodeInsight, CodePurpose};

/// Multi-agent research orchestrator
#[derive(Default)]
pub struct ResearchOrchestrator;

impl ResearchOrchestrator {
    /// Execute all agent analysis pipelines
    pub async fn execute_research_pipeline(&self, context: &GeneratorContext) -> Result<()> {
        println!("🚀 Starting Litho Studies Research investigation pipeline...");

        // First layer: Macro analysis (C1)
        self.execute_agent(&SystemContextResearcher, context)
            .await?;

        // Second layer: Meso analysis (C2)
        self.execute_agent(&DomainModulesDetector, context)
            .await?;
        self.execute_agent(&ArchitectureResearcher, context)
            .await?;
        self.execute_agent(&WorkflowResearcher, context)
            .await?;

        // Third layer: Micro analysis (C3-C4)
        self.execute_agent(&KeyModulesInsight, context)
            .await?;

        // Boundary interface analysis
        self.execute_agent(&BoundaryAnalyzer::default(), context)
            .await?;

        // Database overview analysis (only if database files exist)
        if self.has_database_files(context).await {
            self.execute_agent(&DatabaseOverviewAnalyzer::default(), context)
                .await?;
        }

        println!("✓ Litho Studies Research pipeline execution completed");

        Ok(())
    }

    /// Check if the project has database-related files
    async fn has_database_files(&self, context: &GeneratorContext) -> bool {
        if let Some(insights) = context
            .get_from_memory::<Vec<CodeInsight>>(MemoryScope::PREPROCESS, ScopedKeys::CODE_INSIGHTS)
            .await
        {
            insights.iter().any(|insight| {
                matches!(insight.code_dossier.code_purpose, CodePurpose::Database)
                    || insight.code_dossier.file_path.to_string_lossy().ends_with(".sql")
                    || insight.code_dossier.file_path.to_string_lossy().ends_with(".sqlproj")
            })
        } else {
            false
        }
    }

    /// Execute a single agent
    async fn execute_agent<T>(
        &self,
        agent: &T,
        context: &GeneratorContext,
    ) -> Result<()>
    where
        T: StepForwardAgent + Send + Sync,
    {
        // Use localized agent name if available
        let agent_name = if let Some(agent_enum) = agent.agent_type_enum() {
            agent_enum.display_name(&context.config.target_language)
        } else {
            agent.agent_type()
        };
        
        println!("🤖 Executing {} agent analysis...", agent_name);

        agent.execute(context).await?;
        println!("✓ {} analysis completed", agent_name);
        Ok(())
    }
}
