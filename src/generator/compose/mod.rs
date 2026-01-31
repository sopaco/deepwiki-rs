use crate::generator::compose::agents::architecture_editor::ArchitectureEditor;
use crate::generator::compose::agents::boundary_editor::BoundaryEditor;
use crate::generator::compose::agents::database_editor::DatabaseEditor;
use crate::generator::compose::agents::key_modules_insight_editor::KeyModulesInsightEditor;
use crate::generator::compose::agents::overview_editor::OverviewEditor;
use crate::generator::compose::agents::workflow_editor::WorkflowEditor;
use crate::generator::context::GeneratorContext;
use crate::generator::outlet::DocTree;
use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::step_forward_agent::StepForwardAgent;
use crate::types::code::{CodeInsight, CodePurpose};
use anyhow::Result;

mod agents;
pub mod memory;
pub mod types;

/// Documentation composer
#[derive(Default)]
pub struct DocumentationComposer;

impl DocumentationComposer {
    pub async fn execute(&self, context: &GeneratorContext, doc_tree: &mut DocTree) -> Result<()> {
        println!("\n🤖 Executing documentation generation process...");
        println!("📝 Target language: {}", context.config.target_language.display_name());

        let overview_editor = OverviewEditor::default();
        overview_editor.execute(context).await?;

        let architecture_editor = ArchitectureEditor::default();
        architecture_editor.execute(context).await?;

        let workflow_editor = WorkflowEditor::default();
        workflow_editor.execute(context).await?;

        let key_modules_insight_editor = KeyModulesInsightEditor::default();
        key_modules_insight_editor
            .execute(context, doc_tree)
            .await?;

        let boundary_editor = BoundaryEditor::default();
        boundary_editor.execute(context).await?;

        // Database documentation (only if database files exist)
        if self.has_database_files(context).await {
            let database_editor = DatabaseEditor::default();
            database_editor.execute(context).await?;
        }

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
}
