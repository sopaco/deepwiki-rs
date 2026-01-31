use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Code basic information
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct CodeDossier {
    /// Code file name
    pub name: String,
    /// File path
    pub file_path: PathBuf,
    /// Source code summary
    #[schemars(skip)]
    #[serde(default)]
    pub source_summary: String,
    /// Purpose type
    pub code_purpose: CodePurpose,
    /// Importance score
    pub importance_score: f64,
    pub description: Option<String>,
    pub functions: Vec<String>,
    /// Interfaces list
    pub interfaces: Vec<String>,
}

/// Intelligent insight information of code file
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct CodeInsight {
    /// Code basic information
    pub code_dossier: CodeDossier,
    pub detailed_description: String,
    /// Responsibilities
    pub responsibilities: Vec<String>,
    /// Contained interfaces
    pub interfaces: Vec<InterfaceInfo>,
    /// Dependency information
    pub dependencies: Vec<Dependency>,
    pub complexity_metrics: CodeComplexity,
}

/// Interface information
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct InterfaceInfo {
    pub name: String,
    pub interface_type: String, // "function", "method", "class", "trait", etc.
    pub visibility: String,     // "public", "private", "protected"
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub description: Option<String>,
}

/// Parameter information
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct ParameterInfo {
    pub name: String,
    pub param_type: String,
    pub is_optional: bool,
    pub description: Option<String>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Dependency {
    pub name: String,
    pub path: Option<String>,
    pub is_external: bool,
    pub line_number: Option<usize>,
    pub dependency_type: String, // "import", "use", "include", "require", etc.
    pub version: Option<String>,
}

impl Display for Dependency {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            format!(
                "(name={}, path={}, is_external={},dependency_type={})",
                self.name,
                self.path.as_deref().unwrap_or_default(),
                self.is_external,
                self.dependency_type
            )
        )
    }
}

/// Component complexity metrics
#[derive(Debug, Serialize, Deserialize, Clone, JsonSchema)]
pub struct CodeComplexity {
    pub cyclomatic_complexity: f64,
    pub lines_of_code: usize,
    pub number_of_functions: usize,
    pub number_of_classes: usize,
}

/// Code functionality classification enum
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq, Hash, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum CodePurpose {
    /// Project execution entry
    #[serde(alias = "Project execution entry")]
    Entry,
    /// Intelligent Agent
    #[serde(alias = "Intelligent Agent")]
    Agent,
    /// Frontend UI page
    #[serde(alias = "Frontend UI page")]
    Page,
    /// Frontend UI component
    #[serde(alias = "Frontend UI component")]
    Widget,
    /// Code module for implementing specific logical functionality
    #[serde(alias = "feature", alias = "specific_feature", alias = "specific-feature", alias = "Code module for implementing specific logical functionality")]
    SpecificFeature,
    /// Data type or model
    #[serde(alias = "Data type or model")]
    Model,
    /// Program internal interface definition
    #[serde(alias = "Program internal interface definition")]
    Types,
    /// Functional tool code for specific scenarios
    #[serde(alias = "Functional tool code for specific scenarios")]
    Tool,
    /// Common, basic utility functions and classes, providing low-level auxiliary functions unrelated to business logic
    #[serde(alias = "Common, basic utility functions and classes, providing low-level auxiliary functions unrelated to business logic")]
    Util,
    /// Configuration
    #[serde(alias = "configuration", alias = "Configuration")]
    Config,
    /// Middleware
    #[serde(alias = "Middleware")]
    Middleware,
    /// Plugin
    #[serde(alias = "Plugin")]
    Plugin,
    /// Router in frontend or backend system
    #[serde(alias = "Router in frontend or backend system")]
    Router,
    /// Database component
    #[serde(alias = "Database component")]
    Database,
    /// Service API for external calls, providing calling capabilities based on HTTP, RPC, IPC and other protocols.
    #[serde(alias = "Service API for external calls, providing calling capabilities based on HTTP, RPC, IPC and other protocols.")]
    Api,
    /// Controller component in MVC architecture, responsible for handling business logic
    #[serde(alias = "Controller component in MVC architecture, responsible for handling business logic")]
    Controller,
    /// Service component in MVC architecture, responsible for handling business rules
    #[serde(alias = "Service component in MVC architecture, responsible for handling business rules")]
    Service,
    /// Collection of related code (functions, classes, resources) with clear boundaries and responsibilities
    #[serde(alias = "Collection of related code (functions, classes, resources) with clear boundaries and responsibilities")]
    Module,
    /// Dependency library
    #[serde(alias = "library", alias = "package", alias = "Dependency library")]
    Lib,
    /// Test component
    #[serde(alias = "testing", alias = "tests", alias = "Test component")]
    Test,
    /// Documentation component
    #[serde(alias = "documentation", alias = "docs", alias = "Documentation component")]
    Doc,
    /// Data Access Layer component
    #[serde(alias = "Data Access Layer component")]
    Dao,
    /// Context component
    #[serde(alias = "Context component")]
    Context,
    /// command-line interface (CLI) commands or message/request handlers
    #[serde(alias = "command-line interface (CLI) commands or message/request handlers", alias = "command-line interface (CLI) commands or message/request handlers")]
    Command,
    /// Other uncategorized or unknown
    #[serde(alias = "unknown", alias = "misc", alias = "miscellaneous", alias = "Other uncategorized or unknown")]
    Other,
}

impl CodePurpose {
    /// Get component type display name
    pub fn display_name(&self) -> &'static str {
        match self {
            CodePurpose::Entry => "Project Execution Entry",
            CodePurpose::Agent => "Intelligent Agent",
            CodePurpose::Page => "Frontend UI Page",
            CodePurpose::Widget => "Frontend UI Component",
            CodePurpose::SpecificFeature => "Specific Logic Functionality",
            CodePurpose::Model => "Data Type or Model",
            CodePurpose::Util => "Basic Utility Functions",
            CodePurpose::Tool => "Functional Tool Code for Specific Scenarios",
            CodePurpose::Config => "Configuration",
            CodePurpose::Middleware => "Middleware",
            CodePurpose::Plugin => "Plugin",
            CodePurpose::Router => "Router Component",
            CodePurpose::Database => "Database Component",
            CodePurpose::Api => "Various Interface Definitions",
            CodePurpose::Controller => "Controller Component",
            CodePurpose::Service => "Service Component",
            CodePurpose::Module => "Module Component",
            CodePurpose::Lib => "Dependency Library",
            CodePurpose::Test => "Test Component",
            CodePurpose::Doc => "Documentation Component",
            CodePurpose::Other => "Other Component",
            CodePurpose::Types => "Program Interface Definition",
            CodePurpose::Dao => "Data Access Layer Component",
            CodePurpose::Context => "Context Component",
            CodePurpose::Command => "Command",
        }
    }
}

impl Display for CodePurpose {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_name())
    }
}

impl Default for CodePurpose {
    fn default() -> Self {
        CodePurpose::Other
    }
}

/// Component type mapper, used to map original string types to new enum types
pub struct CodePurposeMapper;

impl CodePurposeMapper {
    /// Intelligent mapping based on file path and name
    pub fn map_by_path_and_name(file_path: &str, file_name: &str) -> CodePurpose {
        let path_lower = file_path.to_lowercase();
        let name_lower = file_name.to_lowercase();

        // Extension-based mapping for SQL-related files
        if name_lower.ends_with(".sqlproj") || name_lower.ends_with(".sql") {
            return CodePurpose::Database;
        }

        // Path-based mapping
        if path_lower.contains("/pages/")
            || path_lower.contains("/views/")
            || path_lower.contains("/screens/")
        {
            return CodePurpose::Page;
        }
        if path_lower.contains("/components/")
            || path_lower.contains("/widgets/")
            || path_lower.contains("/ui/")
        {
            return CodePurpose::Widget;
        }
        if path_lower.contains("/models/")
            || path_lower.contains("/entities/")
            || path_lower.contains("/data/")
        {
            return CodePurpose::Model;
        }
        if path_lower.contains("/utils/")
            || path_lower.contains("/utilities/")
            || path_lower.contains("/helpers/")
        {
            return CodePurpose::Util;
        }
        if path_lower.contains("/config/")
            || path_lower.contains("/configs/")
            || path_lower.contains("/settings/")
        {
            return CodePurpose::Config;
        }
        if path_lower.contains("/middleware/") || path_lower.contains("/middlewares/") {
            return CodePurpose::Middleware;
        }
        if path_lower.contains("/plugin/") {
            return CodePurpose::Plugin;
        }
        if path_lower.contains("/routes/")
            || path_lower.contains("/router/")
            || path_lower.contains("/routing/")
        {
            return CodePurpose::Router;
        }
        if path_lower.contains("/database/")
            || path_lower.contains("/db/")
            || path_lower.contains("/storage/")
        {
            return CodePurpose::Database;
        }
        if path_lower.contains("/dao/")
            || path_lower.contains("/repository/")
            || path_lower.contains("/persistence/")
        {
            return CodePurpose::Dao;
        }
        if path_lower.contains("/context") || path_lower.contains("/ctx/") {
            return CodePurpose::Context;
        }
        if path_lower.contains("/api")
            || path_lower.contains("/endpoint")
            || path_lower.contains("/controller")
            || path_lower.contains("/native_module")
            || path_lower.contains("/bridge")
        {
            return CodePurpose::Api;
        }
        if path_lower.contains("/test/")
            || path_lower.contains("/tests/")
            || path_lower.contains("/__tests__/")
        {
            return CodePurpose::Test;
        }
        if path_lower.contains("/docs/")
            || path_lower.contains("/doc/")
            || path_lower.contains("/documentation/")
        {
            return CodePurpose::Doc;
        }

        // Filename-based mapping
        if name_lower.contains("main") || name_lower.contains("index") || name_lower.contains("app")
        {
            return CodePurpose::Entry;
        }
        if name_lower.contains("page")
            || name_lower.contains("view")
            || name_lower.contains("screen")
        {
            return CodePurpose::Page;
        }
        if name_lower.contains("component") || name_lower.contains("widget") {
            return CodePurpose::Widget;
        }
        if name_lower.contains("model") || name_lower.contains("entity") {
            return CodePurpose::Model;
        }
        if name_lower.contains("util") {
            return CodePurpose::Util;
        }
        if name_lower.contains("config") || name_lower.contains("setting") {
            return CodePurpose::Config;
        }
        if name_lower.contains("middleware") {
            return CodePurpose::Middleware;
        }
        if name_lower.contains("plugin") {
            return CodePurpose::Plugin;
        }
        if name_lower.contains("route") {
            return CodePurpose::Router;
        }
        if name_lower.contains("database") {
            return CodePurpose::Database;
        }
        if name_lower.contains("repository") || name_lower.contains("persistence") {
            return CodePurpose::Dao;
        }
        if name_lower.contains("context") || name_lower.contains("ctx") {
            return CodePurpose::Context;
        }
        if name_lower.contains("api") || name_lower.contains("endpoint") {
            return CodePurpose::Api;
        }
        if name_lower.contains("test") || name_lower.contains("spec") {
            return CodePurpose::Test;
        }
        if name_lower.contains("readme") || name_lower.contains("doc") {
            return CodePurpose::Doc;
        }
        if name_lower.contains("cli") || name_lower.contains("commands") {
            return CodePurpose::Command;
        }

        CodePurpose::Other
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sql_file_classification() {
        // .sqlproj files should always be classified as Database
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name(
                "/src/MyProject.sqlproj",
                "MyProject.sqlproj"
            ),
            CodePurpose::Database
        );

        // .sql files should always be classified as Database
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name(
                "/src/CreateTable.sql",
                "CreateTable.sql"
            ),
            CodePurpose::Database
        );

        // Even in root directory
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name(
                "/Schema.sql",
                "Schema.sql"
            ),
            CodePurpose::Database
        );

        // Even with mixed case
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name(
                "/src/StoredProcedures.SQL",
                "StoredProcedures.SQL"
            ),
            CodePurpose::Database
        );
    }

    #[test]
    fn test_sql_file_in_database_folder() {
        // SQL files in /database/ folder should still be Database
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name(
                "/src/database/schema.sql",
                "schema.sql"
            ),
            CodePurpose::Database
        );
    }

    #[test]
    fn test_path_based_classification() {
        // Files in /database/ folder
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name(
                "/src/database/connection.cs",
                "connection.cs"
            ),
            CodePurpose::Database
        );

        // Files in /repository/ folder
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name(
                "/src/repository/UserRepository.cs",
                "UserRepository.cs"
            ),
            CodePurpose::Dao
        );
    }
}
