use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use std::fmt::Display;

use crate::i18n::TargetLanguage;

pub(crate) fn any_to_string(value: serde_json::Value) -> String {
    match value {
        serde_json::Value::Null => String::new(),
        serde_json::Value::String(s) => s,
        serde_json::Value::Bool(v) => v.to_string(),
        serde_json::Value::Number(v) => v.to_string(),
        serde_json::Value::Array(v) => serde_json::to_string(&v).unwrap_or_default(),
        serde_json::Value::Object(v) => serde_json::to_string(&v).unwrap_or_default(),
    }
}

pub(crate) fn deserialize_string_lenient<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(any_to_string(value))
}

pub(crate) fn deserialize_f64_lenient<'de, D>(deserializer: D) -> Result<f64, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let result = match value {
        serde_json::Value::Number(n) => n.as_f64().unwrap_or(0.0),
        serde_json::Value::String(s) => s.parse::<f64>().unwrap_or(0.0),
        serde_json::Value::Bool(v) => {
            if v {
                1.0
            } else {
                0.0
            }
        }
        _ => 0.0,
    };
    Ok(result)
}

pub(crate) fn deserialize_usize_lenient<'de, D>(deserializer: D) -> Result<usize, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let result = match value {
        serde_json::Value::Number(n) => n.as_u64().unwrap_or(0) as i64,
        serde_json::Value::String(s) => s.parse::<i64>().unwrap_or(0),
        serde_json::Value::Bool(v) => {
            if v {
                1
            } else {
                0
            }
        }
        _ => 0,
    };
    Ok(result.max(0) as usize)
}

pub(crate) fn deserialize_vec_string_lenient<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => Ok(items
            .into_iter()
            .map(any_to_string)
            .filter(|s| !s.trim().is_empty())
            .collect()),
        other => {
            let one = any_to_string(other);
            if one.trim().is_empty() {
                Ok(Vec::new())
            } else {
                Ok(vec![one])
            }
        }
    }
}

fn deserialize_project_type_lenient<'de, D>(deserializer: D) -> Result<ProjectType, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    Ok(ProjectType::map_from_raw(&any_to_string(value)))
}

fn deserialize_vec_user_persona_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<UserPersona>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) =
                            serde_json::from_value::<UserPersona>(serde_json::Value::Object(map))
                        {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let name = any_to_string(other);
                        if !name.trim().is_empty() {
                            out.push(UserPersona {
                                name,
                                description: String::new(),
                                needs: Vec::new(),
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        other => {
            let name = any_to_string(other);
            if name.trim().is_empty() {
                Ok(Vec::new())
            } else {
                Ok(vec![UserPersona {
                    name,
                    description: String::new(),
                    needs: Vec::new(),
                }])
            }
        }
    }
}

fn deserialize_vec_external_system_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<ExternalSystem>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) =
                            serde_json::from_value::<ExternalSystem>(serde_json::Value::Object(map))
                        {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let name = any_to_string(other);
                        if !name.trim().is_empty() {
                            out.push(ExternalSystem {
                                name,
                                description: String::new(),
                                interaction_type: "unknown".to_string(),
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        other => {
            let name = any_to_string(other);
            if name.trim().is_empty() {
                Ok(Vec::new())
            } else {
                Ok(vec![ExternalSystem {
                    name,
                    description: String::new(),
                    interaction_type: "unknown".to_string(),
                }])
            }
        }
    }
}

fn deserialize_vec_submodule_lenient<'de, D>(deserializer: D) -> Result<Vec<SubModule>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<SubModule>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_domain_module_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<DomainModule>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<DomainModule>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_domain_relation_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<DomainRelation>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<DomainRelation>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_business_flow_step_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<BusinessFlowStep>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for (idx, item) in items.into_iter().enumerate() {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) = serde_json::from_value::<BusinessFlowStep>(
                            serde_json::Value::Object(map),
                        ) {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let operation = any_to_string(other);
                        if !operation.trim().is_empty() {
                            out.push(BusinessFlowStep {
                                step: idx + 1,
                                domain_module: String::new(),
                                sub_module: None,
                                operation,
                                code_entry_point: None,
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_business_flow_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<BusinessFlow>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<BusinessFlow>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_bool_lenient<'de, D>(deserializer: D) -> Result<bool, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    let result = match value {
        serde_json::Value::Bool(v) => v,
        serde_json::Value::Number(n) => n.as_i64().unwrap_or(0) != 0,
        serde_json::Value::String(s) => {
            let normalized = s.trim().to_lowercase();
            matches!(normalized.as_str(), "true" | "1" | "yes" | "y")
        }
        _ => false,
    };
    Ok(result)
}

fn deserialize_opt_string_lenient<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    if value.is_null() {
        return Ok(None);
    }

    let text = any_to_string(value);
    if text.trim().is_empty() {
        Ok(None)
    } else {
        Ok(Some(text))
    }
}

fn deserialize_vec_cli_argument_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<CLIArgument>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) =
                            serde_json::from_value::<CLIArgument>(serde_json::Value::Object(map))
                        {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let name = any_to_string(other);
                        if !name.trim().is_empty() {
                            out.push(CLIArgument {
                                name,
                                description: String::new(),
                                required: false,
                                default_value: None,
                                value_type: String::new(),
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_cli_option_lenient<'de, D>(deserializer: D) -> Result<Vec<CLIOption>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) =
                            serde_json::from_value::<CLIOption>(serde_json::Value::Object(map))
                        {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let name = any_to_string(other);
                        if !name.trim().is_empty() {
                            out.push(CLIOption {
                                name,
                                short_name: None,
                                description: String::new(),
                                required: false,
                                default_value: None,
                                value_type: String::new(),
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_router_param_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<RouterParam>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) =
                            serde_json::from_value::<RouterParam>(serde_json::Value::Object(map))
                        {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let key = any_to_string(other);
                        if !key.trim().is_empty() {
                            out.push(RouterParam {
                                key,
                                value_type: String::new(),
                                description: String::new(),
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_cli_boundary_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<CLIBoundary>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) =
                            serde_json::from_value::<CLIBoundary>(serde_json::Value::Object(map))
                        {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let command = any_to_string(other);
                        if !command.trim().is_empty() {
                            out.push(CLIBoundary {
                                command,
                                description: String::new(),
                                arguments: Vec::new(),
                                options: Vec::new(),
                                examples: Vec::new(),
                                source_location: String::new(),
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_api_boundary_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<APIBoundary>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) =
                            serde_json::from_value::<APIBoundary>(serde_json::Value::Object(map))
                        {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let endpoint = any_to_string(other);
                        if !endpoint.trim().is_empty() {
                            out.push(APIBoundary {
                                endpoint,
                                method: String::new(),
                                description: String::new(),
                                request_format: None,
                                response_format: None,
                                authentication: None,
                                source_location: String::new(),
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_router_boundary_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<RouterBoundary>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) =
                            serde_json::from_value::<RouterBoundary>(serde_json::Value::Object(map))
                        {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let path = any_to_string(other);
                        if !path.trim().is_empty() {
                            out.push(RouterBoundary {
                                path,
                                description: String::new(),
                                source_location: String::new(),
                                params: Vec::new(),
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_integration_suggestion_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<IntegrationSuggestion>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                match item {
                    serde_json::Value::Object(map) => {
                        if let Ok(parsed) = serde_json::from_value::<IntegrationSuggestion>(
                            serde_json::Value::Object(map),
                        ) {
                            out.push(parsed);
                        }
                    }
                    other => {
                        let description = any_to_string(other);
                        if !description.trim().is_empty() {
                            out.push(IntegrationSuggestion {
                                integration_type: String::new(),
                                description,
                                example_code: String::new(),
                                best_practices: Vec::new(),
                            });
                        }
                    }
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_database_project_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<DatabaseProject>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<DatabaseProject>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_table_column_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<TableColumn>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<TableColumn>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_database_table_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<DatabaseTable>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<DatabaseTable>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_database_view_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<DatabaseView>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<DatabaseView>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_procedure_param_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<ProcedureParameter>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<ProcedureParameter>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_stored_procedure_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<StoredProcedure>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<StoredProcedure>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_database_function_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<DatabaseFunction>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<DatabaseFunction>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_table_relationship_lenient<'de, D>(
    deserializer: D,
) -> Result<Vec<TableRelationship>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<TableRelationship>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

fn deserialize_vec_data_flow_lenient<'de, D>(deserializer: D) -> Result<Vec<DataFlow>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = serde_json::Value::deserialize(deserializer)?;
    match value {
        serde_json::Value::Null => Ok(Vec::new()),
        serde_json::Value::Array(items) => {
            let mut out = Vec::new();
            for item in items {
                if let Ok(parsed) = serde_json::from_value::<DataFlow>(item) {
                    out.push(parsed);
                }
            }
            Ok(out)
        }
        _ => Ok(Vec::new()),
    }
}

/// Agent type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    SystemContextResearcher,
    DomainModulesDetector,
    ArchitectureResearcher,
    WorkflowResearcher,
    KeyModulesInsight,
    BoundaryAnalyzer,
    DatabaseOverviewAnalyzer,
}

impl AgentType {
    /// Get localized display name for the agent type
    pub fn display_name(&self, target_language: &TargetLanguage) -> String {
        match self {
            AgentType::SystemContextResearcher => target_language.msg_agent_type("system_context"),
            AgentType::DomainModulesDetector => target_language.msg_agent_type("domain_modules"),
            AgentType::ArchitectureResearcher => target_language.msg_agent_type("architecture"),
            AgentType::WorkflowResearcher => target_language.msg_agent_type("workflow"),
            AgentType::KeyModulesInsight => target_language.msg_agent_type("key_modules"),
            AgentType::BoundaryAnalyzer => target_language.msg_agent_type("boundary"),
            AgentType::DatabaseOverviewAnalyzer => target_language.msg_agent_type("database"),
        }
    }
}

impl Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // Use English as default for Display trait (used for keys/internal purposes)
        let str = match self {
            AgentType::SystemContextResearcher => "System Context Research Report",
            AgentType::DomainModulesDetector => "Domain Modules Research Report",
            AgentType::ArchitectureResearcher => "System Architecture Research Report",
            AgentType::WorkflowResearcher => "Workflow Research Report",
            AgentType::KeyModulesInsight => "Key Modules and Components Research Report",
            AgentType::BoundaryAnalyzer => "Boundary Interface Research Report",
            AgentType::DatabaseOverviewAnalyzer => "Database Overview Research Report",
        };
        write!(f, "{}", str)
    }
}

// =========================== Specific Agent Result Types ===========================

/// Project type
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ProjectType {
    FrontendApp,
    BackendService,
    FullStackApp,
    ComponentLibrary,
    Framework,
    CLITool,
    MobileApp,
    DesktopApp,
    Other,
}

impl Default for ProjectType {
    fn default() -> Self {
        Self::Other
    }
}

impl ProjectType {
    pub fn map_from_raw(raw: &str) -> Self {
        match raw.trim().to_lowercase().as_str() {
            "frontendapp" | "frontend" | "frontend_app" | "web" => Self::FrontendApp,
            "backendservice" | "backend" | "backend_service" | "service" => Self::BackendService,
            "fullstackapp" | "fullstack" | "full_stack" => Self::FullStackApp,
            "componentlibrary" | "component_library" | "library" => Self::ComponentLibrary,
            "framework" => Self::Framework,
            "clitool" | "cli_tool" | "cli" => Self::CLITool,
            "mobileapp" | "mobile" | "mobile_app" => Self::MobileApp,
            "desktopapp" | "desktop" | "desktop_app" => Self::DesktopApp,
            _ => Self::Other,
        }
    }
}

/// User persona
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct UserPersona {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub needs: Vec<String>,
}

/// External system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct ExternalSystem {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub interaction_type: String,
}

/// System boundary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct SystemBoundary {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub scope: String,
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub included_components: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub excluded_components: Vec<String>,
}

/// Project objective research result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct SystemContextReport {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub project_name: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub project_description: String,
    #[serde(default, deserialize_with = "deserialize_project_type_lenient")]
    pub project_type: ProjectType,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub business_value: String,
    #[serde(default, deserialize_with = "deserialize_vec_user_persona_lenient")]
    pub target_users: Vec<UserPersona>,
    #[serde(default, deserialize_with = "deserialize_vec_external_system_lenient")]
    pub external_systems: Vec<ExternalSystem>,
    #[serde(default)]
    pub system_boundary: SystemBoundary,
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub confidence_score: f64,
}

/// Sub-module definition - represents specific implementation modules within a larger module
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct SubModule {
    /// Sub-module name, should be concise and clear, reflecting specific functionality
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Sub-module function description, explaining the specific role and responsibilities
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    /// Related code file path list, containing all code files implementing this sub-module's functionality
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub code_paths: Vec<String>,
    /// Core function list, listing the main functions and operations provided by this sub-module
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub key_functions: Vec<String>,
    /// Importance score (1-10), assessing the importance of this sub-module in the overall system
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub importance: f64,
}

/// Functional domain module - represents high-level business domain or functional domain
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct DomainModule {
    /// Domain module name, should reflect high-level business or functional domain, e.g., "User Management Domain", "Data Processing Domain", "Configuration Management Domain"
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Domain module description, detailing the domain's responsibilities, core value, and role in the system
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    /// Domain type, identifying the domain's layer in system architecture, e.g., "Core Business Domain", "Infrastructure Domain", "Tool Support Domain"
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub domain_type: String,
    /// Sub-module list, containing all specific implementation modules under this domain, reflecting functional decomposition within the domain
    #[serde(default, deserialize_with = "deserialize_vec_submodule_lenient")]
    pub sub_modules: Vec<SubModule>,
    /// Related code file path list, containing all code files implementing this domain module's functionality
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub code_paths: Vec<String>,
    /// Domain importance score (1-10), assessing the strategic importance of this domain in the overall system
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub importance: f64,
    /// Domain complexity score (1-10), assessing the technical complexity and implementation difficulty of this domain
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub complexity: f64,
}

/// Inter-domain relationship - represents dependency and collaboration relationships between different domain modules
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct DomainRelation {
    /// Source domain module name, representing the initiator of the dependency relationship
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub from_domain: String,
    /// Target domain module name, representing the receiver of the dependency relationship
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub to_domain: String,
    /// Relationship type, describing the specific relationship between two domains, e.g., "Data Dependency", "Service Call", "Configuration Dependency", "Tool Support"
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub relation_type: String,
    /// Dependency strength (1-10), assessing the coupling degree between two domains, 10 indicates strong dependency, 1 indicates weak dependency
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub strength: f64,
    /// Relationship description, detailing the specific interaction methods and dependency content between two domains
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
}

/// Process step - represents a single execution step in the workflow
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct BusinessFlowStep {
    /// Step number, indicating the execution order of this step in the overall process
    #[serde(default, deserialize_with = "deserialize_usize_lenient")]
    pub step: usize,
    /// Involved domain module name, identifying the primary domain executing this step
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub domain_module: String,
    /// Involved sub-module name (optional), if the step involves a specific sub-module, specify the particular sub-module
    #[serde(default)]
    pub sub_module: Option<String>,
    /// Specific operation description, explaining the specific functional operation or technical action executed in this step
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub operation: String,
    /// Code entry point (optional), pointing to the main code location or function implementing this step
    #[serde(default)]
    pub code_entry_point: Option<String>,
}

/// Core process - represents key functional scenarios and execution paths in the system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct BusinessFlow {
    /// Process name, should reflect specific functional scenario, e.g., "Project Analysis Process", "Code Insight Generation Process"
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Process description, detailing the functional process's objectives, trigger conditions, and expected results
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    /// Process step list, steps arranged in execution order, reflecting the complete functional execution path
    #[serde(
        default,
        deserialize_with = "deserialize_vec_business_flow_step_lenient"
    )]
    pub steps: Vec<BusinessFlowStep>,
    /// Process entry point, explaining the startup method or trigger condition of this functional process
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub entry_point: String,
    /// Process importance score (1-10), assessing the importance of this functional process in the system
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub importance: f64,
    /// Number of involved domains, counting the number of domain modules this process spans, reflecting process complexity
    #[serde(default, deserialize_with = "deserialize_usize_lenient")]
    pub involved_domains_count: usize,
}

/// Core component analysis result
#[derive(Debug, Clone, Serialize, Deserialize, Default, JsonSchema)]
#[serde(default)]
pub struct KeyModuleReport {
    /// Domain name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub domain_name: String,
    /// Module name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub module_name: String,
    /// Explain the project's current technical solution
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub module_description: String,
    /// Explain the defined interfaces and interaction methods
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub interaction: String,
    /// Explain technical details
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub implementation: String,
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub associated_files: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub flowchart_mermaid: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub sequence_diagram_mermaid: String,
}

/// Domain module analysis result from high-level architecture perspective
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct DomainModulesReport {
    /// Identified domain module list, high-level functional modules divided by domain, each domain can contain multiple sub-modules
    #[serde(default, deserialize_with = "deserialize_vec_domain_module_lenient")]
    pub domain_modules: Vec<DomainModule>,
    /// Inter-domain relationship list, describing dependencies, collaboration, and interaction relationships between different domain modules
    #[serde(default, deserialize_with = "deserialize_vec_domain_relation_lenient")]
    pub domain_relations: Vec<DomainRelation>,
    /// Core business process list, identifying important functional scenarios and execution paths in the system
    #[serde(default, deserialize_with = "deserialize_vec_business_flow_lenient")]
    pub business_flows: Vec<BusinessFlow>,
    /// Architecture layer summary, summarizing the overall architectural characteristics and technology selection from a macro perspective
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub architecture_summary: String,
    /// Analysis confidence score (1-10), assessing the credibility and accuracy of this analysis result
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub confidence_score: f64,
}

/// Boundary interface analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct BoundaryAnalysisReport {
    /// CLI boundary interface
    #[serde(default, deserialize_with = "deserialize_vec_cli_boundary_lenient")]
    pub cli_boundaries: Vec<CLIBoundary>,
    /// Network API boundary interface for external invocation (including HTTP, RPC, and other protocols)
    #[serde(default, deserialize_with = "deserialize_vec_api_boundary_lenient")]
    pub api_boundaries: Vec<APIBoundary>,
    /// Page routing
    #[serde(default, deserialize_with = "deserialize_vec_router_boundary_lenient")]
    pub router_boundaries: Vec<RouterBoundary>,
    /// Integration suggestions
    #[serde(
        default,
        deserialize_with = "deserialize_vec_integration_suggestion_lenient"
    )]
    pub integration_suggestions: Vec<IntegrationSuggestion>,
    /// Analysis confidence score (1-10)
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct CLIBoundary {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub command: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_vec_cli_argument_lenient")]
    pub arguments: Vec<CLIArgument>,
    #[serde(default, deserialize_with = "deserialize_vec_cli_option_lenient")]
    pub options: Vec<CLIOption>,
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub examples: Vec<String>,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub source_location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct CLIArgument {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_bool_lenient")]
    pub required: bool,
    #[serde(default, deserialize_with = "deserialize_opt_string_lenient")]
    pub default_value: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct CLIOption {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_opt_string_lenient")]
    pub short_name: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_bool_lenient")]
    pub required: bool,
    #[serde(default, deserialize_with = "deserialize_opt_string_lenient")]
    pub default_value: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct APIBoundary {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub endpoint: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub method: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_opt_string_lenient")]
    pub request_format: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_lenient")]
    pub response_format: Option<String>,
    #[serde(default, deserialize_with = "deserialize_opt_string_lenient")]
    pub authentication: Option<String>,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub source_location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct RouterBoundary {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub path: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub source_location: String,
    #[serde(default, deserialize_with = "deserialize_vec_router_param_lenient")]
    pub params: Vec<RouterParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct RouterParam {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub key: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub value_type: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct IntegrationSuggestion {
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub integration_type: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub example_code: String,
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub best_practices: Vec<String>,
}

/// Database Overview analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct DatabaseOverviewReport {
    /// Database projects found in the solution
    #[serde(default, deserialize_with = "deserialize_vec_database_project_lenient")]
    pub database_projects: Vec<DatabaseProject>,
    /// All tables discovered across all database projects
    #[serde(default, deserialize_with = "deserialize_vec_database_table_lenient")]
    pub tables: Vec<DatabaseTable>,
    /// All views discovered across all database projects
    #[serde(default, deserialize_with = "deserialize_vec_database_view_lenient")]
    pub views: Vec<DatabaseView>,
    /// All stored procedures discovered across all database projects
    #[serde(default, deserialize_with = "deserialize_vec_stored_procedure_lenient")]
    pub stored_procedures: Vec<StoredProcedure>,
    /// All functions discovered across all database projects
    #[serde(
        default,
        deserialize_with = "deserialize_vec_database_function_lenient"
    )]
    pub database_functions: Vec<DatabaseFunction>,
    /// Table relationships (foreign keys, references)
    #[serde(
        default,
        deserialize_with = "deserialize_vec_table_relationship_lenient"
    )]
    pub table_relationships: Vec<TableRelationship>,
    /// Data flow patterns identified
    #[serde(default, deserialize_with = "deserialize_vec_data_flow_lenient")]
    pub data_flows: Vec<DataFlow>,
    /// Analysis confidence score (1-10)
    #[serde(default, deserialize_with = "deserialize_f64_lenient")]
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct DatabaseProject {
    /// Project name (from .sqlproj)
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Project file path
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub project_path: String,
    /// Target database platform (SQL Server, etc.)
    #[serde(default, deserialize_with = "deserialize_opt_string_lenient")]
    pub target_platform: Option<String>,
    /// Number of tables
    #[serde(default, deserialize_with = "deserialize_usize_lenient")]
    pub table_count: usize,
    /// Number of views
    #[serde(default, deserialize_with = "deserialize_usize_lenient")]
    pub view_count: usize,
    /// Number of stored procedures
    #[serde(default, deserialize_with = "deserialize_usize_lenient")]
    pub procedure_count: usize,
    /// Number of functions
    #[serde(default, deserialize_with = "deserialize_usize_lenient")]
    pub function_count: usize,
    /// Referenced database projects or DACPACs
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct DatabaseTable {
    /// Schema name (e.g., dbo)
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub schema: String,
    /// Table name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Column definitions
    #[serde(default, deserialize_with = "deserialize_vec_table_column_lenient")]
    pub columns: Vec<TableColumn>,
    /// Primary key columns
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub primary_key: Vec<String>,
    /// Description/purpose of the table
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    /// Source file path
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct TableColumn {
    /// Column name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Data type (e.g., INT, NVARCHAR(100))
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub data_type: String,
    /// Whether the column allows NULL
    #[serde(default, deserialize_with = "deserialize_bool_lenient")]
    pub nullable: bool,
    /// Whether this is an identity/auto-increment column
    #[serde(default, deserialize_with = "deserialize_bool_lenient")]
    pub is_identity: bool,
    /// Default value if any
    #[serde(default, deserialize_with = "deserialize_opt_string_lenient")]
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct DatabaseView {
    /// Schema name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub schema: String,
    /// View name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Description of what the view does
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    /// Tables referenced by this view
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub referenced_tables: Vec<String>,
    /// Source file path
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct StoredProcedure {
    /// Schema name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub schema: String,
    /// Procedure name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Parameters
    #[serde(default, deserialize_with = "deserialize_vec_procedure_param_lenient")]
    pub parameters: Vec<ProcedureParameter>,
    /// Description of what the procedure does
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    /// Tables referenced (SELECT, INSERT, UPDATE, DELETE)
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub referenced_tables: Vec<String>,
    /// Source file path
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct ProcedureParameter {
    /// Parameter name (including @)
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Data type
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub data_type: String,
    /// Whether it has a default value (is optional)
    #[serde(default, deserialize_with = "deserialize_bool_lenient")]
    pub is_optional: bool,
    /// Direction: INPUT, OUTPUT, INOUT
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct DatabaseFunction {
    /// Schema name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub schema: String,
    /// Function name
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Function type: Scalar, Table-valued, etc.
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub function_type: String,
    /// Parameters
    #[serde(default, deserialize_with = "deserialize_vec_procedure_param_lenient")]
    pub parameters: Vec<ProcedureParameter>,
    /// Return type
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub return_type: String,
    /// Description
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub description: String,
    /// Source file path
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct TableRelationship {
    /// Source table (schema.table)
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub from_table: String,
    /// Source column(s)
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub from_columns: Vec<String>,
    /// Target table (schema.table)
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub to_table: String,
    /// Target column(s)
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub to_columns: Vec<String>,
    /// Relationship type: ForeignKey, Reference, Implicit
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub relationship_type: String,
    /// Constraint name if explicit FK
    #[serde(default, deserialize_with = "deserialize_opt_string_lenient")]
    pub constraint_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, Default)]
#[serde(default)]
pub struct DataFlow {
    /// Flow name/description
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub name: String,
    /// Source (table, external system, or procedure)
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub source: String,
    /// Destination (table, external system, or procedure)
    #[serde(default, deserialize_with = "deserialize_string_lenient")]
    pub destination: String,
    /// Operations involved (INSERT, UPDATE, MERGE, etc.)
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub operations: Vec<String>,
    /// Procedures involved in this flow
    #[serde(default, deserialize_with = "deserialize_vec_string_lenient")]
    pub procedures_involved: Vec<String>,
}

// https://c4model.com/abstractions/software-system
// System name, project's role and value, system type, who is using it, how to use, which external systems it interacts with, diagram

#[cfg(test)]
mod tests {
    use super::{KeyModuleReport, SystemContextReport};

    #[test]
    fn test_key_module_report_deserialize_with_missing_module_name() {
        let payload = serde_json::json!({
            "domain_name": "Tài liệu & IaC",
            "module_description": "Infrastructure and documentation module"
        });

        let report: KeyModuleReport = serde_json::from_value(payload)
            .expect("KeyModuleReport should deserialize when module_name is missing");

        assert_eq!(report.module_name, "");
        assert_eq!(report.domain_name, "Tài liệu & IaC");
    }

    #[test]
    fn test_system_context_report_deserialize_lenient_mixed_types() {
        let payload = serde_json::json!({
            "project_name": "telemetry-processor",
            "project_description": "Processes telemetry",
            "project_type": "BackendService",
            "business_value": "Provides pipeline reliability",
            "target_users": [
                {"name": "Platform engineer"},
                "Operations team"
            ],
            "external_systems": [
                "MaxMind GeoIP database files (required for geo-location lookups)",
                {"name": "Redis"}
            ],
            "system_boundary": {
                "scope": "Telemetry ingestion",
                "included_components": "reader",
                "excluded_components": ["ui"]
            },
            "confidence_score": "8.4"
        });

        let report: SystemContextReport = serde_json::from_value(payload)
            .expect("SystemContextReport should deserialize with lenient field handling");

        assert_eq!(report.project_name, "telemetry-processor");
        assert_eq!(report.target_users.len(), 2);
        assert_eq!(report.external_systems.len(), 2);
        assert_eq!(report.system_boundary.included_components.len(), 1);
    }

    #[test]
    fn test_domain_modules_report_deserialize_lenient_business_flow_steps() {
        let payload = serde_json::json!({
            "domain_modules": [
                {
                    "name": "Ingestion",
                    "description": "Ingests telemetry",
                    "sub_modules": [
                        {
                            "name": "Reader"
                        }
                    ]
                }
            ],
            "domain_relations": [],
            "business_flows": [
                {
                    "name": "Tick flow",
                    "steps": [
                        "Load Config from environment (Config::from_env)",
                        {"step": "2", "operation": "Process event"}
                    ]
                }
            ],
            "architecture_summary": "Layered",
            "confidence_score": "7.2"
        });

        let report: super::DomainModulesReport = serde_json::from_value(payload)
            .expect("DomainModulesReport should deserialize with lenient step handling");

        assert_eq!(report.business_flows.len(), 1);
        assert_eq!(report.business_flows[0].steps.len(), 2);
        assert_eq!(report.business_flows[0].steps[0].step, 1);
        assert_eq!(report.business_flows[0].steps[1].step, 2);
    }

    #[test]
    fn test_key_module_report_deserialize_lenient_mixed_types() {
        let payload = serde_json::json!({
            "domain_name": "Caching",
            "module_name": 42,
            "module_description": {"summary": "Cache orchestration module"},
            "interaction": ["reader->cache", "cache->redis"],
            "implementation": true,
            "associated_files": "src/cache.rs",
            "flowchart_mermaid": {"text": "flowchart TD; A-->B;"},
            "sequence_diagram_mermaid": null
        });

        let report: KeyModuleReport = serde_json::from_value(payload)
            .expect("KeyModuleReport should deserialize with lenient mixed types");

        assert_eq!(report.domain_name, "Caching");
        assert_eq!(report.module_name, "42");
        assert_eq!(report.associated_files.len(), 1);
    }

    #[test]
    fn test_boundary_analysis_report_deserialize_lenient_shapes() {
        let payload = serde_json::json!({
            "cli_boundaries": [
                {"description": "main cli"},
                "telemetry-processor"
            ],
            "api_boundaries": [
                {"method": "GET"},
                "/health"
            ],
            "router_boundaries": [
                {"path": "/"},
                "/status"
            ],
            "integration_suggestions": [
                {"description": "Use retries"},
                "Prefer idempotent operations"
            ],
            "confidence_score": "7.0"
        });

        let report: super::BoundaryAnalysisReport = serde_json::from_value(payload)
            .expect("BoundaryAnalysisReport should deserialize with lenient boundary shapes");

        assert_eq!(report.cli_boundaries.len(), 2);
        assert_eq!(report.api_boundaries.len(), 2);
        assert_eq!(report.router_boundaries.len(), 2);
        assert_eq!(report.integration_suggestions.len(), 2);
        assert!(
            report.cli_boundaries[0].command.is_empty()
                || report.cli_boundaries[1].command.is_empty()
        );
    }

    #[test]
    fn test_database_overview_report_deserialize_lenient_dataflow_source() {
        let payload = serde_json::json!({
            "database_projects": [],
            "tables": [],
            "views": [],
            "stored_procedures": [],
            "database_functions": [],
            "table_relationships": [],
            "data_flows": [
                {
                    "name": "ingest",
                    "destination": "events",
                    "operations": ["INSERT"]
                },
                "fallback flow"
            ],
            "confidence_score": "6.5"
        });

        let report: super::DatabaseOverviewReport = serde_json::from_value(payload)
            .expect("DatabaseOverviewReport should deserialize with missing dataflow source");

        assert_eq!(report.data_flows.len(), 1);
        assert_eq!(report.data_flows[0].name, "ingest");
        assert_eq!(report.data_flows[0].source, "");
    }
}
