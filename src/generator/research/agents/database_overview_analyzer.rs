use crate::generator::preprocess::memory::{MemoryScope, ScopedKeys};
use crate::generator::research::types::{AgentType, DatabaseOverviewReport};
use crate::generator::{
    context::GeneratorContext,
    step_forward_agent::{
        AgentDataConfig, DataSource, FormatterConfig, LLMCallMode, PromptTemplate, StepForwardAgent,
    },
};
use crate::types::code::CodePurpose;
use crate::types::{CodeAndDirectoryInsights, FileInsight};
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

You MUST output strict JSON only (no markdown, no code fences, no prose outside JSON).
Return exactly this shape with all keys present:
{
  "database_projects": [
    {
      "name": "string",
      "project_path": "string",
      "target_platform": "string or null",
      "table_count": 0,
      "view_count": 0,
      "procedure_count": 0,
      "function_count": 0,
      "references": ["string"]
    }
  ],
  "tables": [
    {
      "schema": "string",
      "name": "string",
      "columns": [
        {
          "name": "string",
          "data_type": "string",
          "nullable": false,
          "is_identity": false,
          "default_value": "string or null"
        }
      ],
      "primary_key": ["string"],
      "description": "string",
      "source_path": "string"
    }
  ],
  "views": [
    {
      "schema": "string",
      "name": "string",
      "description": "string",
      "referenced_tables": ["string"],
      "source_path": "string"
    }
  ],
  "stored_procedures": [
    {
      "schema": "string",
      "name": "string",
      "parameters": [
        {
          "name": "string",
          "data_type": "string",
          "is_optional": false,
          "direction": "string"
        }
      ],
      "description": "string",
      "referenced_tables": ["string"],
      "source_path": "string"
    }
  ],
  "database_functions": [
    {
      "schema": "string",
      "name": "string",
      "function_type": "string",
      "parameters": [
        {
          "name": "string",
          "data_type": "string",
          "is_optional": false,
          "direction": "string"
        }
      ],
      "return_type": "string",
      "description": "string",
      "source_path": "string"
    }
  ],
  "table_relationships": [
    {
      "from_table": "string",
      "from_columns": ["string"],
      "to_table": "string",
      "to_columns": ["string"],
      "relationship_type": "string",
      "constraint_name": "string or null"
    }
  ],
  "data_flows": [
    {
      "name": "string",
      "source": "string",
      "destination": "string",
      "operations": ["string"],
      "procedures_involved": ["string"]
    }
  ],
  "confidence_score": 0.0
}

Rules:
- Always include all top-level keys.
- Items in arrays must be objects, never plain strings.
- Use empty arrays if no database objects exist.
- Use empty strings or null for unknown fields.
- confidence_score must be numeric (0.0-10.0)."#
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
                code_insights_limit: 50,   // Reduced limit to prevent token overflow
                only_directories_when_files_more_than: Some(300),
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
    ) -> Result<Vec<FileInsight>> {
        let all_insights = context
            .get_from_memory::<CodeAndDirectoryInsights>(MemoryScope::PREPROCESS, ScopedKeys::CODE_INSIGHTS)
            .await
            .ok_or_else(|| anyhow!("CODE_INSIGHTS not found in PREPROCESS memory"))?;

        // Flatten all file_insights from directory_dossiers and filter by database-related purpose
        let database_insights: Vec<FileInsight> = all_insights
            .directory_insights
            .iter()
            .flat_map(|d| d.file_insights.iter())
            .filter(|fi| {
                // First check file size to exclude extremely large files
                let source_len = fi.source_summary.len();
                if source_len > 50000 {
                    return false; // Skip files with source summaries larger than 50KB
                }

                // Include files with Database or DAO purpose
                matches!(fi.code_purpose, CodePurpose::Database | CodePurpose::Dao)
                    // Include files with SQL-related file extensions
                    || fi.file_path.to_string_lossy().ends_with(".sql")
                    || fi.file_path.to_string_lossy().ends_with(".sqlproj")
            })
            .cloned()
            .collect();

        // Truncate source summaries for remaining files to prevent token overflow
        let mut truncated: Vec<FileInsight> = database_insights
            .into_iter()
            .map(|mut fi| {
                if fi.source_summary.len() > 10000 {
                    fi.source_summary = fi.source_summary.chars().take(10000).collect::<String>();
                    fi.source_summary.push_str("\n\n[Content truncated for analysis]");
                }
                fi
            })
            .collect();

        // Take up to 50 most important to prevent token overflow
        truncated.truncate(50);

        // Group by type and count
        let mut sqlproj_count = 0;
        let mut sql_count = 0;
        let mut dao_count = 0;

        for fi in &truncated {
            let path = fi.file_path.to_string_lossy();
            if path.ends_with(".sqlproj") {
                sqlproj_count += 1;
            } else if path.ends_with(".sql") {
                sql_count += 1;
            } else if matches!(fi.code_purpose, CodePurpose::Dao) {
                dao_count += 1;
            }
        }

        println!(
            "📊 Database code distribution: Projects({}) SQL Files({}) DAO({})",
            sqlproj_count, sql_count, dao_count
        );

        Ok(truncated)
    }

    /// Format database code insights
    fn format_database_insights(&self, insights: &[FileInsight]) -> String {
        let mut content = String::from("### Database-Related Code Insights\n\n");

        // Group by type
        let mut projects = Vec::new();
        let mut tables = Vec::new();
        let mut views = Vec::new();
        let mut procedures = Vec::new();
        let mut functions = Vec::new();
        let mut other_sql = Vec::new();
        let mut dao_files = Vec::new();

        for fi in insights {
            let path = fi.file_path.to_string_lossy().to_lowercase();

            if path.ends_with(".sqlproj") {
                projects.push(fi);
            } else if path.ends_with(".sql") {
                // Categorize SQL files by content/path
                if path.contains("table") || fi.name.to_lowercase().contains("table") {
                    tables.push(fi);
                } else if path.contains("view") || fi.name.to_lowercase().contains("view") {
                    views.push(fi);
                } else if path.contains("procedure") || path.contains("storedproc") || path.contains("sproc") {
                    procedures.push(fi);
                } else if path.contains("function") {
                    functions.push(fi);
                } else {
                    other_sql.push(fi);
                }
            } else if matches!(fi.code_purpose, CodePurpose::Dao) {
                dao_files.push(fi);
            }
        }

        // Format each category
        if !projects.is_empty() {
            content.push_str("#### Database Projects (.sqlproj)\n");
            content.push_str("These are SQL Server Database Projects:\n\n");
            for fi in projects {
                self.add_insight_item(&mut content, fi);
            }
        }

        if !tables.is_empty() {
            content.push_str("#### Table Definitions\n");
            content.push_str("SQL files containing table definitions:\n\n");
            for fi in tables {
                self.add_insight_item(&mut content, fi);
            }
        }

        if !views.is_empty() {
            content.push_str("#### View Definitions\n");
            content.push_str("SQL files containing view definitions:\n\n");
            for fi in views {
                self.add_insight_item(&mut content, fi);
            }
        }

        if !procedures.is_empty() {
            content.push_str("#### Stored Procedures\n");
            content.push_str("SQL files containing stored procedure definitions:\n\n");
            for fi in procedures {
                self.add_insight_item(&mut content, fi);
            }
        }

        if !functions.is_empty() {
            content.push_str("#### Functions\n");
            content.push_str("SQL files containing function definitions:\n\n");
            for fi in functions {
                self.add_insight_item(&mut content, fi);
            }
        }

        if !other_sql.is_empty() {
            content.push_str("#### Other SQL Files\n");
            content.push_str("Other SQL scripts and files:\n\n");
            for fi in other_sql {
                self.add_insight_item(&mut content, fi);
            }
        }

        if !dao_files.is_empty() {
            content.push_str("#### Data Access Objects (DAO)\n");
            content.push_str("Code files that interact with the database:\n\n");
            for fi in dao_files {
                self.add_insight_item(&mut content, fi);
            }
        }

        content.push_str("\n");
        content
    }

    /// Determine appropriate language identifier based on file type
    fn determine_code_language(&self, file_path: &std::path::Path) -> &str {
        let path = file_path.to_string_lossy();

        if path.ends_with(".sql") || path.ends_with(".sqlproj") {
            "sql"
        } else if path.ends_with(".java") {
            "java"
        } else if path.ends_with(".py") || path.ends_with(".pyw") {
            "python"
        } else if path.ends_with(".cs") {
            "csharp"
        } else if path.ends_with(".js") {
            "javascript"
        } else if path.ends_with(".ts") {
            "typescript"
        } else if path.ends_with(".go") {
            "go"
        } else if path.ends_with(".rs") {
            "rust"
        } else if path.ends_with(".xml") || path.ends_with(".config") {
            "xml"
        } else if path.ends_with(".json") {
            "json"
        } else if path.ends_with(".yaml") || path.ends_with(".yml") {
            "yaml"
        } else {
            "source file"
        }
    }

    /// Add single insight item to content
    fn add_insight_item(&self, content: &mut String, fi: &FileInsight) {
        content.push_str(&format!(
            "- **{}** (`{}`)\n",
            fi.name,
            fi.file_path.display()
        ));

        if !fi.summary.is_empty() {
            content.push_str(&format!("  - Description: {}\n", fi.summary));
        }

        // Add source summary if available
        if !fi.source_summary.is_empty() {
            content.push_str("  - Source Preview:\n");
            content.push_str(&format!("    ```{}\n", self.determine_code_language(&fi.file_path)));
            // Limit to first 150 chars to reduce token usage
            let preview: String = fi.source_summary.chars().take(150).collect();
            for line in preview.lines().take(8) {
                content.push_str(&format!("    {}\n", line));
            }
            if fi.source_summary.len() > 150 {
                content.push_str("    ...\n");
            }
            content.push_str("    ```\n");
        }

        content.push_str("\n");
    }
}
