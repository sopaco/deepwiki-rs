use crate::generator::compose::memory::MemoryScope;
use crate::generator::compose::types::AgentType;
use crate::generator::context::GeneratorContext;
use crate::generator::research::memory::MemoryRetriever;
use crate::generator::research::types::{
    AgentType as ResearchAgentType, DatabaseOverviewReport, DatabaseProject, DatabaseTable,
    DatabaseView, StoredProcedure, DatabaseFunction, TableRelationship, DataFlow,
};
use crate::generator::step_forward_agent::{
    AgentDataConfig, DataSource, PromptTemplate, StepForwardAgent,
};
use anyhow::Result;
use async_trait::async_trait;

/// Database Documentation Editor - Orchestrates database analysis results into standardized documentation
#[derive(Default)]
pub struct DatabaseEditor;

#[async_trait]
impl StepForwardAgent for DatabaseEditor {
    type Output = String;

    fn agent_type(&self) -> String {
        AgentType::Database.to_string()
    }

    fn memory_scope_key(&self) -> String {
        MemoryScope::DOCUMENTATION.to_string()
    }

    fn should_include_timestamp(&self) -> bool {
        true
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![],
            optional_sources: vec![
                DataSource::ResearchResult(ResearchAgentType::DatabaseOverviewAnalyzer.to_string()),
                DataSource::PROJECT_STRUCTURE,
                DataSource::CODE_INSIGHTS,
                // Use database docs for additional context
                DataSource::knowledge_categories(vec!["database"]),
            ],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt: r#"You are a professional database documentation expert, focused on generating clear, detailed database schema and structure documentation."#.to_string(),
            opening_instruction: "Based on the following database analysis results, generate database overview documentation:".to_string(),
            closing_instruction: "".to_string(),
            llm_call_mode: crate::generator::step_forward_agent::LLMCallMode::Prompt,
            formatter_config: crate::generator::step_forward_agent::FormatterConfig::default(),
        }
    }

    /// Custom execute implementation that generates documentation directly without using LLM
    async fn execute(&self, context: &GeneratorContext) -> Result<Self::Output> {
        // Get database analysis results from memory
        let database_analysis = context
            .get_research(&ResearchAgentType::DatabaseOverviewAnalyzer.to_string())
            .await;

        // If no database analysis exists, return minimal content
        let content = if let Some(analysis) = database_analysis {
            // Parse as DatabaseOverviewReport
            let report: DatabaseOverviewReport = serde_json::from_value(analysis)?;
            self.generate_database_documentation(&report)
        } else {
            "## Database Overview\n\nNo database components were detected in this project.\n".to_string()
        };

        // Store to memory
        let value = serde_json::to_value(&content)?;
        context
            .store_to_memory(&self.memory_scope_key(), &self.agent_type(), value)
            .await?;

        Ok(content)
    }
}

impl DatabaseEditor {
    /// Generate database overview documentation
    fn generate_database_documentation(&self, report: &DatabaseOverviewReport) -> String {
        let mut content = String::new();

        // Title
        content.push_str("## Database Overview\n\n");

        // Summary statistics
        content.push_str("### Summary\n\n");
        content.push_str(&format!("| Metric | Count |\n"));
        content.push_str(&format!("|--------|-------|\n"));
        content.push_str(&format!("| Database Projects | {} |\n", report.database_projects.len()));
        content.push_str(&format!("| Tables | {} |\n", report.tables.len()));
        content.push_str(&format!("| Views | {} |\n", report.views.len()));
        content.push_str(&format!("| Stored Procedures | {} |\n", report.stored_procedures.len()));
        content.push_str(&format!("| Functions | {} |\n", report.database_functions.len()));
        content.push_str(&format!("| Relationships | {} |\n", report.table_relationships.len()));
        content.push_str("\n");

        // Database Projects
        if !report.database_projects.is_empty() {
            content.push_str("### Database Projects\n\n");
            for project in &report.database_projects {
                self.format_database_project(&mut content, project);
            }
        }

        // Tables
        if !report.tables.is_empty() {
            content.push_str("### Tables\n\n");
            for table in &report.tables {
                self.format_table(&mut content, table);
            }
        }

        // Views
        if !report.views.is_empty() {
            content.push_str("### Views\n\n");
            for view in &report.views {
                self.format_view(&mut content, view);
            }
        }

        // Stored Procedures
        if !report.stored_procedures.is_empty() {
            content.push_str("### Stored Procedures\n\n");
            for proc in &report.stored_procedures {
                self.format_stored_procedure(&mut content, proc);
            }
        }

        // Functions
        if !report.database_functions.is_empty() {
            content.push_str("### Functions\n\n");
            for func in &report.database_functions {
                self.format_function(&mut content, func);
            }
        }

        // Table Relationships
        if !report.table_relationships.is_empty() {
            content.push_str("### Table Relationships\n\n");
            content.push_str("```mermaid\nerDiagram\n");
            for rel in &report.table_relationships {
                self.format_relationship_mermaid(&mut content, rel);
            }
            content.push_str("```\n\n");
            
            // Also add table format
            content.push_str("| From Table | From Columns | To Table | To Columns | Type |\n");
            content.push_str("|------------|--------------|----------|------------|------|\n");
            for rel in &report.table_relationships {
                self.format_relationship_table(&mut content, rel);
            }
            content.push_str("\n");
        }

        // Data Flows
        if !report.data_flows.is_empty() {
            content.push_str("### Data Flows\n\n");
            for flow in &report.data_flows {
                self.format_data_flow(&mut content, flow);
            }
        }

        content
    }

    fn format_database_project(&self, content: &mut String, project: &DatabaseProject) {
        content.push_str(&format!("#### {}\n\n", project.name));
        content.push_str(&format!("- **Project Path:** `{}`\n", project.project_path));
        if let Some(platform) = &project.target_platform {
            content.push_str(&format!("- **Target Platform:** {}\n", platform));
        }
        content.push_str(&format!("- **Objects:** {} tables, {} views, {} procedures, {} functions\n",
            project.table_count, project.view_count, project.procedure_count, project.function_count));
        if !project.references.is_empty() {
            content.push_str(&format!("- **References:** {}\n", project.references.join(", ")));
        }
        content.push_str("\n");
    }

    fn format_table(&self, content: &mut String, table: &DatabaseTable) {
        content.push_str(&format!("#### {}.{}\n\n", table.schema, table.name));
        if !table.description.is_empty() {
            content.push_str(&format!("{}\n\n", table.description));
        }
        content.push_str(&format!("**Source:** `{}`\n\n", table.source_path));
        
        if !table.columns.is_empty() {
            content.push_str("| Column | Type | Nullable | Identity |\n");
            content.push_str("|--------|------|----------|----------|\n");
            for col in &table.columns {
                let nullable = if col.nullable { "Yes" } else { "No" };
                let identity = if col.is_identity { "Yes" } else { "No" };
                content.push_str(&format!("| {} | {} | {} | {} |\n",
                    col.name, col.data_type, nullable, identity));
            }
            content.push_str("\n");
        }
        
        if !table.primary_key.is_empty() {
            content.push_str(&format!("**Primary Key:** {}\n\n", table.primary_key.join(", ")));
        }
    }

    fn format_view(&self, content: &mut String, view: &DatabaseView) {
        content.push_str(&format!("#### {}.{}\n\n", view.schema, view.name));
        if !view.description.is_empty() {
            content.push_str(&format!("{}\n\n", view.description));
        }
        content.push_str(&format!("**Source:** `{}`\n\n", view.source_path));
        if !view.referenced_tables.is_empty() {
            content.push_str(&format!("**References Tables:** {}\n\n", view.referenced_tables.join(", ")));
        }
    }

    fn format_stored_procedure(&self, content: &mut String, proc: &StoredProcedure) {
        content.push_str(&format!("#### {}.{}\n\n", proc.schema, proc.name));
        if !proc.description.is_empty() {
            content.push_str(&format!("{}\n\n", proc.description));
        }
        content.push_str(&format!("**Source:** `{}`\n\n", proc.source_path));
        
        if !proc.parameters.is_empty() {
            content.push_str("**Parameters:**\n\n");
            content.push_str("| Name | Type | Direction | Optional |\n");
            content.push_str("|------|------|-----------|----------|\n");
            for param in &proc.parameters {
                let optional = if param.is_optional { "Yes" } else { "No" };
                content.push_str(&format!("| {} | {} | {} | {} |\n",
                    param.name, param.data_type, param.direction, optional));
            }
            content.push_str("\n");
        }
        
        if !proc.referenced_tables.is_empty() {
            content.push_str(&format!("**Accesses Tables:** {}\n\n", proc.referenced_tables.join(", ")));
        }
    }

    fn format_function(&self, content: &mut String, func: &DatabaseFunction) {
        content.push_str(&format!("#### {}.{}\n\n", func.schema, func.name));
        if !func.description.is_empty() {
            content.push_str(&format!("{}\n\n", func.description));
        }
        content.push_str(&format!("**Type:** {}\n", func.function_type));
        content.push_str(&format!("**Returns:** {}\n", func.return_type));
        content.push_str(&format!("**Source:** `{}`\n\n", func.source_path));
        
        if !func.parameters.is_empty() {
            content.push_str("**Parameters:**\n\n");
            content.push_str("| Name | Type | Optional |\n");
            content.push_str("|------|------|----------|\n");
            for param in &func.parameters {
                let optional = if param.is_optional { "Yes" } else { "No" };
                content.push_str(&format!("| {} | {} | {} |\n",
                    param.name, param.data_type, optional));
            }
            content.push_str("\n");
        }
    }

    fn format_relationship_mermaid(&self, content: &mut String, rel: &TableRelationship) {
        // Extract table names without schema for cleaner diagram
        let from_table = rel.from_table.split('.').last().unwrap_or(&rel.from_table);
        let to_table = rel.to_table.split('.').last().unwrap_or(&rel.to_table);
        
        let rel_symbol = match rel.relationship_type.as_str() {
            "ForeignKey" => "}o--||",
            "Reference" => "..>",
            _ => "--",
        };
        
        content.push_str(&format!("    {} {} {} : \"{}\"\n", 
            from_table, rel_symbol, to_table, 
            rel.constraint_name.as_deref().unwrap_or("references")));
    }

    fn format_relationship_table(&self, content: &mut String, rel: &TableRelationship) {
        content.push_str(&format!("| {} | {} | {} | {} | {} |\n",
            rel.from_table,
            rel.from_columns.join(", "),
            rel.to_table,
            rel.to_columns.join(", "),
            rel.relationship_type));
    }

    fn format_data_flow(&self, content: &mut String, flow: &DataFlow) {
        content.push_str(&format!("#### {}\n\n", flow.name));
        content.push_str(&format!("- **Source:** {}\n", flow.source));
        content.push_str(&format!("- **Destination:** {}\n", flow.destination));
        content.push_str(&format!("- **Operations:** {}\n", flow.operations.join(", ")));
        if !flow.procedures_involved.is_empty() {
            content.push_str(&format!("- **Procedures:** {}\n", flow.procedures_involved.join(", ")));
        }
        content.push_str("\n");
    }
}
