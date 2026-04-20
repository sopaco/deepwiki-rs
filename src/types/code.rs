use std::{
    fmt::{Display, Formatter},
    path::PathBuf,
};

use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

fn json_value_to_string(value: serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s,
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::Array(v) => serde_json::to_string(&v).unwrap_or_default(),
        serde_json::Value::Object(v) => {
            for key in [
                "name",
                "module",
                "path",
                "summary",
                "description",
                "title",
                "value",
                "text",
                "id",
            ] {
                if let Some(inner) = v.get(key) {
                    let text = json_value_to_string(inner.clone());
                    if !text.is_empty() {
                        return text;
                    }
                }
            }
            serde_json::to_string(&v).unwrap_or_default()
        }
    }
}

fn deserialize_string_lenient<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(json_value_to_string(value))
}

fn deserialize_option_string_lenient<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(None),
        other => Ok(Some(json_value_to_string(other))),
    }
}

fn deserialize_vec_string_lenient<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let values = match value {
        serde_json::Value::Null => Vec::new(),
        serde_json::Value::Array(items) => items
            .into_iter()
            .map(json_value_to_string)
            .filter(|v| !v.is_empty())
            .collect(),
        other => {
            let one = json_value_to_string(other);
            if one.is_empty() {
                Vec::new()
            } else {
                vec![one]
            }
        }
    };

    Ok(values)
}

fn deserialize_f64_lenient<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let result = match value {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        serde_json::Value::Bool(v) => if v { 1.0 } else { 0.0 },
        _ => 0.0,
    };
    Ok(result)
}

fn deserialize_usize_lenient<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let result = match value {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0) as usize,
        serde_json::Value::String(s) => s.parse::<usize>().unwrap_or(0),
        serde_json::Value::Bool(v) => if v { 1 } else { 0 },
        _ => 0,
    };
    Ok(result)
}

fn deserialize_code_complexity_lenient<'de, D>(deserializer: D) -> Result<CodeComplexity, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    
    // Handle the case where LLM returns a JSON object as a string
    let json_value = match value {
        serde_json::Value::String(s) => {
            // Try to parse the string as JSON
            serde_json::from_str::<serde_json::Value>(&s).unwrap_or(serde_json::Value::Object(serde_json::Map::new()))
        }
        other => other,
    };
    
    // Extract fields with lenient handling
    let cyclomatic_complexity = match json_value.get("cyclomatic_complexity") {
        Some(serde_json::Value::Number(n)) => n.as_f64().unwrap_or(0.0),
        Some(serde_json::Value::String(s)) => s.parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    };
    
    let lines_of_code = match json_value.get("lines_of_code") {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as usize,
        Some(serde_json::Value::String(s)) => s.parse::<usize>().unwrap_or(0),
        _ => 0,
    };
    
    let number_of_functions = match json_value.get("number_of_functions") {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as usize,
        Some(serde_json::Value::String(s)) => s.parse::<usize>().unwrap_or(0),
        _ => 0,
    };
    
    let number_of_classes = match json_value.get("number_of_classes") {
        Some(serde_json::Value::Number(n)) => n.as_u64().unwrap_or(0) as usize,
        Some(serde_json::Value::String(s)) => s.parse::<usize>().unwrap_or(0),
        _ => 0,
    };
    
    Ok(CodeComplexity {
        cyclomatic_complexity,
        lines_of_code,
        number_of_functions,
        number_of_classes,
    })
}

fn deserialize_interfaces_lenient<'de, D>(deserializer: D) -> Result<Vec<InterfaceInfo>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let mut interfaces = Vec::new();

    let items = match value {
        serde_json::Value::Null => Vec::new(),
        serde_json::Value::Array(items) => items,
        other => vec![other],
    };

    for item in items {
        match item {
            serde_json::Value::String(name) => interfaces.push(InterfaceInfo {
                name,
                ..Default::default()
            }),
            serde_json::Value::Object(_) => {
                if let Ok(interface) = serde_json::from_value::<InterfaceInfo>(item.clone()) {
                    interfaces.push(interface);
                } else {
                    interfaces.push(InterfaceInfo {
                        name: json_value_to_string(item),
                        ..Default::default()
                    });
                }
            }
            other => interfaces.push(InterfaceInfo {
                name: json_value_to_string(other),
                ..Default::default()
            }),
        }
    }

    Ok(interfaces)
}

fn deserialize_dependencies_lenient<'de, D>(deserializer: D) -> Result<Vec<Dependency>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let mut dependencies = Vec::new();

    let items = match value {
        serde_json::Value::Null => Vec::new(),
        serde_json::Value::Array(items) => items,
        other => vec![other],
    };

    for item in items {
        match item {
            serde_json::Value::String(name) => dependencies.push(Dependency {
                name,
                ..Default::default()
            }),
            serde_json::Value::Object(_) => {
                if let Ok(dependency) = serde_json::from_value::<Dependency>(item.clone()) {
                    dependencies.push(dependency);
                } else {
                    dependencies.push(Dependency {
                        name: json_value_to_string(item),
                        ..Default::default()
                    });
                }
            }
            other => dependencies.push(Dependency {
                name: json_value_to_string(other),
                ..Default::default()
            }),
        }
    }

    Ok(dependencies)
}

fn deserialize_code_purpose_lenient<'de, D>(deserializer: D) -> Result<CodePurpose, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let raw = json_value_to_string(value);
    Ok(CodePurposeMapper::map_from_raw(&raw))
}

/// Code basic information
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct CodeDossier {
    /// Code file name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// File path
    pub file_path: PathBuf,
    /// Source code summary
    #[schemars(skip)]
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub source_summary: String,
    /// Purpose type
    #[serde(default, deserialize_with = "deserialize_code_purpose_lenient")]
    pub code_purpose: CodePurpose,
    /// Importance score
    pub importance_score: f64,
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub functions: Vec<String>,
    /// Interfaces list
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub interfaces: Vec<String>,
}

/// Intelligent insight information of code file
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct CodeInsight {
    /// Code basic information
    pub code_dossier: CodeDossier,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub detailed_description: String,
    /// Responsibilities
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub responsibilities: Vec<String>,
    /// Contained interfaces
    #[serde(default, deserialize_with = "deserialize_interfaces_lenient")]
    pub interfaces: Vec<InterfaceInfo>,
    /// Dependency information
    #[serde(default, deserialize_with = "deserialize_dependencies_lenient")]
    pub dependencies: Vec<Dependency>,
    #[serde(default, deserialize_with = "deserialize_code_complexity_lenient")]
    pub complexity_metrics: CodeComplexity,
}

/// Simplified LLM output structure for code insight
/// Only contains fields that need LLM enhancement, reducing JSON schema complexity
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
#[allow(dead_code)]
pub struct CodeInsightLLMOutput {
    /// Detailed description of the code component
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub detailed_description: String,
    /// Core responsibilities (3-5 items recommended)
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub responsibilities: Vec<String>,
}

impl CodeInsight {
    /// Merge LLM output into this CodeInsight
    #[allow(dead_code)]
    pub fn merge_llm_output(&mut self, llm_output: CodeInsightLLMOutput) {
        if !llm_output.detailed_description.is_empty() {
            self.detailed_description = llm_output.detailed_description;
        }
        if !llm_output.responsibilities.is_empty() {
            self.responsibilities = llm_output.responsibilities;
        }
    }
}

/// Interface information
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct InterfaceInfo {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub interface_type: String, // "function", "method", "class", "trait", etc.
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub visibility: String, // "public", "private", "protected"
    pub parameters: Vec<ParameterInfo>,
    pub return_type: Option<String>,
    pub description: Option<String>,
}

/// Parameter information
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct ParameterInfo {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub param_type: String,
    pub is_optional: bool,
    pub description: Option<String>,
}

/// Dependency information
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(default)]
pub struct Dependency {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_option_string_lenient")]
    pub path: Option<String>,
    pub is_external: bool,
    pub line_number: Option<usize>,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub dependency_type: String, // "import", "use", "include", "require", etc.
    #[serde(default, deserialize_with = "deserialize_option_string_lenient")]
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
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(default)]
pub struct CodeComplexity {
    #[serde(deserialize_with = "deserialize_f64_lenient")]
    pub cyclomatic_complexity: f64,
    #[serde(deserialize_with = "deserialize_usize_lenient")]
    pub lines_of_code: usize,
    #[serde(deserialize_with = "deserialize_usize_lenient")]
    pub number_of_functions: usize,
    #[serde(deserialize_with = "deserialize_usize_lenient")]
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
    #[serde(
        alias = "feature",
        alias = "specific_feature",
        alias = "specific-feature",
        alias = "Specificfeature",
        alias = "SpecificFeature",
        alias = "Code module for implementing specific logical functionality"
    )]
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
    #[serde(
        alias = "Common, basic utility functions and classes, providing low-level auxiliary functions unrelated to business logic"
    )]
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
    #[serde(
        alias = "Service API for external calls, providing calling capabilities based on HTTP, RPC, IPC and other protocols."
    )]
    Api,
    /// Controller component in MVC architecture, responsible for handling business logic
    #[serde(
        alias = "Controller component in MVC architecture, responsible for handling business logic"
    )]
    Controller,
    /// Service component in MVC architecture, responsible for handling business rules
    #[serde(
        alias = "Service component in MVC architecture, responsible for handling business rules"
    )]
    Service,
    /// Collection of related code (functions, classes, resources) with clear boundaries and responsibilities
    #[serde(
        alias = "Collection of related code (functions, classes, resources) with clear boundaries and responsibilities"
    )]
    Module,
    /// Dependency library
    #[serde(alias = "library", alias = "package", alias = "Dependency library")]
    Lib,
    /// Test component
    #[serde(alias = "testing", alias = "tests", alias = "Test component")]
    Test,
    /// Documentation component
    #[serde(
        alias = "documentation",
        alias = "docs",
        alias = "Documentation component"
    )]
    Doc,
    /// Data Access Layer component
    #[serde(alias = "Data Access Layer component")]
    Dao,
    /// Context component
    #[serde(alias = "Context component")]
    Context,
    /// command-line interface (CLI) commands or message/request handlers
    #[serde(
        alias = "command-line interface (CLI) commands or message/request handlers",
        alias = "command-line interface (CLI) commands or message/request handlers"
    )]
    Command,
    /// Other uncategorized or unknown
    #[serde(
        alias = "unknown",
        alias = "misc",
        alias = "miscellaneous",
        alias = "Other uncategorized or unknown"
    )]
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
    pub fn map_from_raw(raw: &str) -> CodePurpose {
        let normalized = raw
            .to_lowercase()
            .chars()
            .filter(char::is_ascii_alphanumeric)
            .collect::<String>();

        if normalized.is_empty() {
            return CodePurpose::Other;
        }
        if normalized.contains("specificfeature") || normalized == "feature" {
            return CodePurpose::SpecificFeature;
        }
        if normalized.contains("frontenduicomponent") || normalized == "widget" {
            return CodePurpose::Widget;
        }
        if normalized.contains("frontenduipage") || normalized == "page" {
            return CodePurpose::Page;
        }
        if normalized.contains("agent") {
            return CodePurpose::Agent;
        }
        if normalized.contains("entry") {
            return CodePurpose::Entry;
        }
        if normalized.contains("database") {
            return CodePurpose::Database;
        }
        if normalized.contains("config") {
            return CodePurpose::Config;
        }
        if normalized.contains("context") {
            return CodePurpose::Context;
        }
        if normalized.contains("router") {
            return CodePurpose::Router;
        }
        if normalized.contains("serviceapi") {
            return CodePurpose::Api;
        }
        if normalized.contains("service") {
            return CodePurpose::Service;
        }
        if normalized.contains("controller") {
            return CodePurpose::Controller;
        }
        if normalized.contains("api") {
            return CodePurpose::Api;
        }
        if normalized.contains("model") {
            return CodePurpose::Model;
        }
        if normalized.contains("types") {
            return CodePurpose::Types;
        }
        if normalized.contains("util") || normalized.contains("helper") {
            return CodePurpose::Util;
        }
        if normalized.contains("tool") {
            return CodePurpose::Tool;
        }
        if normalized.contains("module") {
            return CodePurpose::Module;
        }
        if normalized.contains("dao") || normalized.contains("repository") {
            return CodePurpose::Dao;
        }
        if normalized.contains("test") {
            return CodePurpose::Test;
        }
        if normalized.contains("doc") {
            return CodePurpose::Doc;
        }
        if normalized.contains("command") || normalized.contains("cli") {
            return CodePurpose::Command;
        }
        if normalized.contains("library") || normalized.contains("package") || normalized == "lib" {
            return CodePurpose::Lib;
        }

        CodePurpose::Other
    }

    /// Intelligent mapping based on file path and name
    #[allow(dead_code)]
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
            CodePurposeMapper::map_by_path_and_name("/src/MyProject.sqlproj", "MyProject.sqlproj"),
            CodePurpose::Database
        );

        // .sql files should always be classified as Database
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name("/src/CreateTable.sql", "CreateTable.sql"),
            CodePurpose::Database
        );

        // Even in root directory
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name("/Schema.sql", "Schema.sql"),
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
            CodePurposeMapper::map_by_path_and_name("/src/database/schema.sql", "schema.sql"),
            CodePurpose::Database
        );
    }

    #[test]
    fn test_path_based_classification() {
        // Files in /database/ folder
        assert_eq!(
            CodePurposeMapper::map_by_path_and_name("/src/database/connection.cs", "connection.cs"),
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

    #[test]
    fn test_code_insight_deserialize_with_missing_fields() {
        let payload = serde_json::json!({
            "code_dossier": {
                "name": "chat-window.tsx",
                "file_path": "src/chat-window.tsx"
            },
            "interfaces": [
                {
                    "interface_type": "function"
                }
            ],
            "dependencies": [
                {
                    "path": "react",
                    "is_external": true
                }
            ]
        });

        let insight: CodeInsight = serde_json::from_value(payload).expect(
            "CodeInsight should support partial model outputs and fill defaults for missing fields",
        );

        assert_eq!(insight.code_dossier.name, "chat-window.tsx");
        assert_eq!(
            insight.code_dossier.file_path,
            PathBuf::from("src/chat-window.tsx")
        );
        assert_eq!(insight.code_dossier.code_purpose, CodePurpose::Other);
        assert!(insight.responsibilities.is_empty());
        assert_eq!(insight.interfaces[0].name, "");
        assert_eq!(insight.dependencies[0].name, "");
    }

    #[test]
    fn test_code_insight_deserialize_with_specificfeature_variant() {
        let payload = serde_json::json!({
            "code_dossier": {
                "name": "connect-button.tsx",
                "file_path": "src/connect-button.tsx",
                "code_purpose": "Specificfeature"
            }
        });

        let insight: CodeInsight = serde_json::from_value(payload).expect(
            "CodeInsight should accept common model variant typo `Specificfeature` for code_purpose",
        );

        assert_eq!(
            insight.code_dossier.code_purpose,
            CodePurpose::SpecificFeature
        );
    }

    #[test]
    fn test_code_insight_deserialize_with_loose_schema_values() {
        let payload = serde_json::json!({
            "code_dossier": {
                "name": "use-toast.ts",
                "file_path": "src/use-toast.ts",
                "code_purpose": "widget"
            },
            "detailed_description": {
                "summary": "hook for toast state"
            },
            "interfaces": [
                "State interface"
            ],
            "dependencies": [
                {
                    "name": {"module": "react"},
                    "is_external": true
                }
            ]
        });

        let insight: CodeInsight = serde_json::from_value(payload)
            .expect("CodeInsight should tolerate loose schema values from LLM output");

        assert_eq!(insight.code_dossier.name, "use-toast.ts");
        assert_eq!(insight.detailed_description, "hook for toast state");
        assert_eq!(insight.interfaces.len(), 1);
        assert_eq!(insight.interfaces[0].name, "State interface");
        assert_eq!(insight.dependencies.len(), 1);
        assert_eq!(insight.dependencies[0].name, "react");
    }
}
