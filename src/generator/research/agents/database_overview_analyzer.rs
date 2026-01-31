use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::research::types::{AgentType, DatabaseOverviewReport};
use crate::generator::{
    context::GeneratorContext,
    step_forward_agent::{
        AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
    },
};
use crate::types::code::{CodeInsight, CodePurpose};
use anyhow::{Result, anyhow};
use async_trait::async_trait;

/// Database Overview Analyzer - Analyzes SQL database projects, tables, views, stored procedures, and data relationships
#[derive(Default, Clone)]
pub struct DatabaseOverviewAnalyzer;

#[async_trait]
impl StepForwardAgent for DatabaseOverviewAnalyzer {
    type Output = DatabaseOverviewReport;

    fn agent_type(&self) -> String {
        AgentType::DatabaseOverviewAnalyzer.to_string()
    }

    fn agent_type_enum(&self) -> Option<AgentType> {
        Some(AgentType::DatabaseOverviewAnalyzer)
    }

    fn memory_scope_key(&self) -> String {
        crate::generator::research::memory::MemoryScope::STUDIES_RESEARCH.to_string()
    }

    fn data_config(&self) -> AgentDataConfig {
        AgentDataConfig {
            required_sources: vec![
                DataSource::PROJECT_STRUCTURE,
                DataSource::CODE_INSIGHTS,
            ],
            // Use database documentation for additional context
            optional_sources: vec![DataSource::knowledge_categories(vec!["database", "architecture"])],
        }
    }

    fn prompt_template(&self) -> PromptTemplate {
        PromptTemplate {
            system_prompt:
                r#"You are a professional database architect and SQL analyst, focused on analyzing SQL Server database projects and their structures.

Your task is to analyze the provided SQL code insights and produce a comprehensive database overview including:

1. **Database Projects** - Identify .sqlproj files and their structure
2. **Tables** - Extract table definitions, columns, data types, constraints
3. **Views** - Identify views and their source tables
4. **Stored Procedures** - Analyze stored procedures, their parameters, and the tables they interact with
5. **Functions** - Identify scalar and table-valued functions
6. **Relationships** - Detect foreign key relationships and implicit references between tables
7. **Data Flows** - Identify data movement patterns through procedures and ETL-like operations

You may have access to existing database documentation from external sources.
If available:
- Cross-reference discovered objects with documented schemas
- Validate naming conventions and data types
- Use documented business context for descriptions
- Identify any undocumented database objects

Focus on:
- Extract schema and object names accurately
- Identify column data types and constraints
- Detect relationships between tables (explicit FKs and implicit references via JOINs)
- Understand the purpose of stored procedures and functions
- Map data flow patterns through the database

Please return the analysis results in structured JSON format."#
                    .to_string(),

            opening_instruction: "Analyze the database structure based on the following SQL code insights and project information:".to_string(),

            closing_instruction: r#"
## Analysis Requirements:
- Focus on Database-purpose code (.sql, .sqlproj files)
- Extract table schemas, columns, and data types from CREATE TABLE statements
- Identify stored procedure parameters and referenced tables
- Detect foreign key relationships from constraint definitions
- Identify implicit relationships from JOIN conditions in views and procedures
- Map data flows through INSERT/UPDATE/DELETE operations in procedures
- If certain database objects don't exist, the corresponding arrays can be empty
- Provide meaningful descriptions based on naming conventions and context"#
                .to_string(),

            llm_call_mode: LLMCallMode::Extract,
            formatter_config: FormatterConfig {
                include_source_code: true, // Database analysis requires viewing SQL source code
                code_insights_limit: 200,  // Increase limit to capture all database objects
                only_directories_when_files_more_than: Some(500),
                ..FormatterConfig::default()
            },
        }
    }

    /// Provide custom database code analysis content
    async fn provide_custom_prompt_content(
        &self,
        context: &GeneratorContext,
    ) -> Result<Option<String>> {
        // Filter database-related code insights
        let database_insights = self.filter_database_code_insights(context).await?;

        if database_insights.is_empty() {
            return Ok(Some(
                "### Database-Related Code Insights\nNo SQL database-related code found in this project.\n\n".to_string(),
            ));
        }

        // Format database code insights
        let formatted_content = self.format_database_insights(&database_insights);

        Ok(Some(formatted_content))
    }

    /// Post-processing - output analysis summary
    fn post_process(
        &self,
        result: &DatabaseOverviewReport,
        _context: &GeneratorContext,
    ) -> Result<()> {
        println!("✅ Database overview analysis completed:");
        println!("   - Database projects: {} items", result.database_projects.len());
        println!("   - Tables: {} items", result.tables.len());
        println!("   - Views: {} items", result.views.len());
        println!("   - Stored procedures: {} items", result.stored_procedures.len());
        println!("   - Functions: {} items", result.database_functions.len());
        println!("   - Table relationships: {} items", result.table_relationships.len());
        println!("   - Data flows: {} items", result.data_flows.len());
        println!("   - Confidence: {:.1}/10", result.confidence_score);

        Ok(())
    }
}

impl DatabaseOverviewAnalyzer {
    /// Filter database-related code insights
    async fn filter_database_code_insights(
        &self,
        context: &GeneratorContext,
    ) -> Result<Vec<CodeInsight>> {
        let all_insights = context
            .get_from_memory::<Vec<CodeInsight>>(MemoryScope::PREPROCESS, ScopedKeys::CODE_INSIGHTS)
            .await
            .ok_or_else(|| anyhow!("CODE_INSIGHTS not found in PREPROCESS memory"))?;

        // Filter database-related code
        let database_insights: Vec<CodeInsight> = all_insights
            .into_iter()
            .filter(|insight| {
                // Include files with Database purpose
                matches!(insight.code_dossier.code_purpose, CodePurpose::Database)
                    // Also include DAO files as they often reflect database structure
                    || matches!(insight.code_dossier.code_purpose, CodePurpose::Dao)
                    // Include files with SQL-related component types
                    || insight.code_dossier.file_path.to_string_lossy().ends_with(".sql")
                    || insight.code_dossier.file_path.to_string_lossy().ends_with(".sqlproj")
            })
            .collect();

        // Sort by importance
        let mut sorted_insights = database_insights;
        sorted_insights.sort_by(|a, b| {
            b.code_dossier
                .importance_score
                .partial_cmp(&a.code_dossier.importance_score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        // Take up to 200 most important
        sorted_insights.truncate(200);

        // Group by type and count
        let mut sqlproj_count = 0;
        let mut sql_count = 0;
        let mut dao_count = 0;

        for insight in &sorted_insights {
            let path = insight.code_dossier.file_path.to_string_lossy();
            if path.ends_with(".sqlproj") {
                sqlproj_count += 1;
            } else if path.ends_with(".sql") {
                sql_count += 1;
            } else if matches!(insight.code_dossier.code_purpose, CodePurpose::Dao) {
                dao_count += 1;
            }
        }

        println!(
            "📊 Database code distribution: Projects({}) SQL Files({}) DAO({})",
            sqlproj_count, sql_count, dao_count
        );

        Ok(sorted_insights)
    }

    /// Format database code insights
    fn format_database_insights(&self, insights: &[CodeInsight]) -> String {
        let mut content = String::from("### Database-Related Code Insights\n\n");

        // Group by type
        let mut projects = Vec::new();
        let mut tables = Vec::new();
        let mut views = Vec::new();
        let mut procedures = Vec::new();
        let mut functions = Vec::new();
        let mut other_sql = Vec::new();
        let mut dao_files = Vec::new();

        for insight in insights {
            let path = insight.code_dossier.file_path.to_string_lossy().to_lowercase();
            
            if path.ends_with(".sqlproj") {
                projects.push(insight);
            } else if path.ends_with(".sql") {
                // Categorize SQL files by content/path
                if path.contains("table") || insight.code_dossier.name.to_lowercase().contains("table") {
                    tables.push(insight);
                } else if path.contains("view") || insight.code_dossier.name.to_lowercase().contains("view") {
                    views.push(insight);
                } else if path.contains("procedure") || path.contains("storedproc") || path.contains("sproc") {
                    procedures.push(insight);
                } else if path.contains("function") {
                    functions.push(insight);
                } else {
                    other_sql.push(insight);
                }
            } else if matches!(insight.code_dossier.code_purpose, CodePurpose::Dao) {
                dao_files.push(insight);
            }
        }

        // Format each category
        if !projects.is_empty() {
            content.push_str("#### Database Projects (.sqlproj)\n");
            content.push_str("These are SQL Server Database Projects:\n\n");
            for insight in projects {
                self.add_insight_item(&mut content, insight);
            }
        }

        if !tables.is_empty() {
            content.push_str("#### Table Definitions\n");
            content.push_str("SQL files containing table definitions:\n\n");
            for insight in tables {
                self.add_insight_item(&mut content, insight);
            }
        }

        if !views.is_empty() {
            content.push_str("#### View Definitions\n");
            content.push_str("SQL files containing view definitions:\n\n");
            for insight in views {
                self.add_insight_item(&mut content, insight);
            }
        }

        if !procedures.is_empty() {
            content.push_str("#### Stored Procedures\n");
            content.push_str("SQL files containing stored procedure definitions:\n\n");
            for insight in procedures {
                self.add_insight_item(&mut content, insight);
            }
        }

        if !functions.is_empty() {
            content.push_str("#### Functions\n");
            content.push_str("SQL files containing function definitions:\n\n");
            for insight in functions {
                self.add_insight_item(&mut content, insight);
            }
        }

        if !other_sql.is_empty() {
            content.push_str("#### Other SQL Files\n");
            content.push_str("Other SQL scripts and files:\n\n");
            for insight in other_sql {
                self.add_insight_item(&mut content, insight);
            }
        }

        if !dao_files.is_empty() {
            content.push_str("#### Data Access Objects (DAO)\n");
            content.push_str("Code files that interact with the database:\n\n");
            for insight in dao_files {
                self.add_insight_item(&mut content, insight);
            }
        }

        content.push_str("\n");
        content
    }

    /// Add single insight item to content
    fn add_insight_item(&self, content: &mut String, insight: &CodeInsight) {
        content.push_str(&format!(
            "- **{}** (`{}`)\n",
            insight.code_dossier.name,
            insight.code_dossier.file_path.display()
        ));
        
        if let Some(desc) = &insight.code_dossier.description {
            content.push_str(&format!("  - Description: {}\n", desc));
        }
        
        // Add interface information for SQL objects
        if !insight.code_dossier.interfaces.is_empty() {
            content.push_str("  - SQL Objects: ");
            content.push_str(&insight.code_dossier.interfaces.join(", "));
            content.push_str("\n");
        }
        
        // Add source summary if available
        if !insight.code_dossier.source_summary.is_empty() {
            content.push_str("  - Source Preview:\n");
            content.push_str("    ```sql\n");
            // Limit to first 500 chars
            let preview: String = insight.code_dossier.source_summary.chars().take(500).collect();
            for line in preview.lines().take(15) {
                content.push_str(&format!("    {}\n", line));
            }
            if insight.code_dossier.source_summary.len() > 500 {
                content.push_str("    ...\n");
            }
            content.push_str("    ```\n");
        }
        
        content.push_str("\n");
    }
}
