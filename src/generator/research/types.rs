use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::Display;

use crate::i18n::TargetLanguage;

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

/// User persona
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserPersona {
    pub name: String,
    pub description: String,
    pub needs: Vec<String>,
}

/// External system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExternalSystem {
    pub name: String,
    pub description: String,
    pub interaction_type: String,
}

/// System boundary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemBoundary {
    pub scope: String,
    pub included_components: Vec<String>,
    pub excluded_components: Vec<String>,
}

/// Project objective research result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SystemContextReport {
    pub project_name: String,
    pub project_description: String,
    pub project_type: ProjectType,
    pub business_value: String,
    pub target_users: Vec<UserPersona>,
    pub external_systems: Vec<ExternalSystem>,
    pub system_boundary: SystemBoundary,
    pub confidence_score: f64,
}

/// Sub-module definition - represents specific implementation modules within a larger module
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SubModule {
    /// Sub-module name, should be concise and clear, reflecting specific functionality
    pub name: String,
    /// Sub-module function description, explaining the specific role and responsibilities
    pub description: String,
    /// Related code file path list, containing all code files implementing this sub-module's functionality
    pub code_paths: Vec<String>,
    /// Core function list, listing the main functions and operations provided by this sub-module
    pub key_functions: Vec<String>,
    /// Importance score (1-10), assessing the importance of this sub-module in the overall system
    pub importance: f64,
}

/// Functional domain module - represents high-level business domain or functional domain
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DomainModule {
    /// Domain module name, should reflect high-level business or functional domain, e.g., "User Management Domain", "Data Processing Domain", "Configuration Management Domain"
    pub name: String,
    /// Domain module description, detailing the domain's responsibilities, core value, and role in the system
    pub description: String,
    /// Domain type, identifying the domain's layer in system architecture, e.g., "Core Business Domain", "Infrastructure Domain", "Tool Support Domain"
    pub domain_type: String,
    /// Sub-module list, containing all specific implementation modules under this domain, reflecting functional decomposition within the domain
    pub sub_modules: Vec<SubModule>,
    /// Related code file path list, containing all code files implementing this domain module's functionality
    pub code_paths: Vec<String>,
    /// Domain importance score (1-10), assessing the strategic importance of this domain in the overall system
    pub importance: f64,
    /// Domain complexity score (1-10), assessing the technical complexity and implementation difficulty of this domain
    pub complexity: f64,
}

/// Inter-domain relationship - represents dependency and collaboration relationships between different domain modules
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DomainRelation {
    /// Source domain module name, representing the initiator of the dependency relationship
    pub from_domain: String,
    /// Target domain module name, representing the receiver of the dependency relationship
    pub to_domain: String,
    /// Relationship type, describing the specific relationship between two domains, e.g., "Data Dependency", "Service Call", "Configuration Dependency", "Tool Support"
    pub relation_type: String,
    /// Dependency strength (1-10), assessing the coupling degree between two domains, 10 indicates strong dependency, 1 indicates weak dependency
    pub strength: f64,
    /// Relationship description, detailing the specific interaction methods and dependency content between two domains
    pub description: String,
}

/// Process step - represents a single execution step in the workflow
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BusinessFlowStep {
    /// Step number, indicating the execution order of this step in the overall process
    pub step: usize,
    /// Involved domain module name, identifying the primary domain executing this step
    pub domain_module: String,
    /// Involved sub-module name (optional), if the step involves a specific sub-module, specify the particular sub-module
    pub sub_module: Option<String>,
    /// Specific operation description, explaining the specific functional operation or technical action executed in this step
    pub operation: String,
    /// Code entry point (optional), pointing to the main code location or function implementing this step
    pub code_entry_point: Option<String>,
}

/// Core process - represents key functional scenarios and execution paths in the system
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BusinessFlow {
    /// Process name, should reflect specific functional scenario, e.g., "Project Analysis Process", "Code Insight Generation Process"
    pub name: String,
    /// Process description, detailing the functional process's objectives, trigger conditions, and expected results
    pub description: String,
    /// Process step list, steps arranged in execution order, reflecting the complete functional execution path
    pub steps: Vec<BusinessFlowStep>,
    /// Process entry point, explaining the startup method or trigger condition of this functional process
    pub entry_point: String,
    /// Process importance score (1-10), assessing the importance of this functional process in the system
    pub importance: f64,
    /// Number of involved domains, counting the number of domain modules this process spans, reflecting process complexity
    pub involved_domains_count: usize,
}

/// Core component analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KeyModuleReport {
    /// Domain name
    pub domain_name: String,
    /// Module name
    pub module_name: String,
    /// Explain the project's current technical solution
    pub module_description: String,
    /// Explain the defined interfaces and interaction methods
    pub interaction: String,
    /// Explain technical details
    pub implementation: String,
    pub associated_files: Vec<String>,
    pub flowchart_mermaid: String,
    pub sequence_diagram_mermaid: String,
}

/// Domain module analysis result from high-level architecture perspective
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DomainModulesReport {
    /// Identified domain module list, high-level functional modules divided by domain, each domain can contain multiple sub-modules
    pub domain_modules: Vec<DomainModule>,
    /// Inter-domain relationship list, describing dependencies, collaboration, and interaction relationships between different domain modules
    pub domain_relations: Vec<DomainRelation>,
    /// Core business process list, identifying important functional scenarios and execution paths in the system
    pub business_flows: Vec<BusinessFlow>,
    /// Architecture layer summary, summarizing the overall architectural characteristics and technology selection from a macro perspective
    pub architecture_summary: String,
    /// Analysis confidence score (1-10), assessing the credibility and accuracy of this analysis result
    pub confidence_score: f64,
}

/// Workflow research result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct WorkflowReport {
    // System main workflow
    pub main_workflow: Workflow,
    // Other important workflows
    pub other_important_workflows: Vec<Workflow>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct Workflow {
    pub name: String,
    pub description: String,
    pub flowchart_mermaid: String,
}

/// Boundary interface analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BoundaryAnalysisReport {
    /// CLI boundary interface
    pub cli_boundaries: Vec<CLIBoundary>,
    /// Network API boundary interface for external invocation (including HTTP, RPC, and other protocols)
    pub api_boundaries: Vec<APIBoundary>,
    /// Page routing
    pub router_boundaries: Vec<RouterBoundary>,
    /// Integration suggestions
    pub integration_suggestions: Vec<IntegrationSuggestion>,
    /// Analysis confidence score (1-10)
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CLIBoundary {
    pub command: String,
    pub description: String,
    pub arguments: Vec<CLIArgument>,
    pub options: Vec<CLIOption>,
    pub examples: Vec<String>,
    pub source_location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CLIArgument {
    pub name: String,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CLIOption {
    pub name: String,
    pub short_name: Option<String>,
    pub description: String,
    pub required: bool,
    pub default_value: Option<String>,
    pub value_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct APIBoundary {
    pub endpoint: String,
    pub method: String,
    pub description: String,
    pub request_format: Option<String>,
    pub response_format: Option<String>,
    pub authentication: Option<String>,
    pub source_location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RouterBoundary {
    pub path: String,
    pub description: String,
    pub source_location: String,
    pub params: Vec<RouterParam>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RouterParam {
    pub key: String,
    pub value_type: String,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct IntegrationSuggestion {
    pub integration_type: String,
    pub description: String,
    pub example_code: String,
    pub best_practices: Vec<String>,
}

impl Default for BoundaryAnalysisReport {
    fn default() -> Self {
        Self {
            cli_boundaries: Vec::new(),
            api_boundaries: Vec::new(),
            integration_suggestions: Vec::new(),
            confidence_score: 0.0,
            router_boundaries: Vec::new(),
        }
    }
}

/// Database Overview analysis result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseOverviewReport {
    /// Database projects found in the solution
    pub database_projects: Vec<DatabaseProject>,
    /// All tables discovered across all database projects
    pub tables: Vec<DatabaseTable>,
    /// All views discovered across all database projects
    pub views: Vec<DatabaseView>,
    /// All stored procedures discovered across all database projects
    pub stored_procedures: Vec<StoredProcedure>,
    /// All functions discovered across all database projects
    pub database_functions: Vec<DatabaseFunction>,
    /// Table relationships (foreign keys, references)
    pub table_relationships: Vec<TableRelationship>,
    /// Data flow patterns identified
    pub data_flows: Vec<DataFlow>,
    /// Analysis confidence score (1-10)
    pub confidence_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseProject {
    /// Project name (from .sqlproj)
    pub name: String,
    /// Project file path
    pub project_path: String,
    /// Target database platform (SQL Server, etc.)
    pub target_platform: Option<String>,
    /// Number of tables
    pub table_count: usize,
    /// Number of views
    pub view_count: usize,
    /// Number of stored procedures
    pub procedure_count: usize,
    /// Number of functions
    pub function_count: usize,
    /// Referenced database projects or DACPACs
    pub references: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseTable {
    /// Schema name (e.g., dbo)
    pub schema: String,
    /// Table name
    pub name: String,
    /// Column definitions
    pub columns: Vec<TableColumn>,
    /// Primary key columns
    pub primary_key: Vec<String>,
    /// Description/purpose of the table
    pub description: String,
    /// Source file path
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TableColumn {
    /// Column name
    pub name: String,
    /// Data type (e.g., INT, NVARCHAR(100))
    pub data_type: String,
    /// Whether the column allows NULL
    pub nullable: bool,
    /// Whether this is an identity/auto-increment column
    pub is_identity: bool,
    /// Default value if any
    pub default_value: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseView {
    /// Schema name
    pub schema: String,
    /// View name
    pub name: String,
    /// Description of what the view does
    pub description: String,
    /// Tables referenced by this view
    pub referenced_tables: Vec<String>,
    /// Source file path
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct StoredProcedure {
    /// Schema name
    pub schema: String,
    /// Procedure name
    pub name: String,
    /// Parameters
    pub parameters: Vec<ProcedureParameter>,
    /// Description of what the procedure does
    pub description: String,
    /// Tables referenced (SELECT, INSERT, UPDATE, DELETE)
    pub referenced_tables: Vec<String>,
    /// Source file path
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ProcedureParameter {
    /// Parameter name (including @)
    pub name: String,
    /// Data type
    pub data_type: String,
    /// Whether it has a default value (is optional)
    pub is_optional: bool,
    /// Direction: INPUT, OUTPUT, INOUT
    pub direction: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DatabaseFunction {
    /// Schema name
    pub schema: String,
    /// Function name
    pub name: String,
    /// Function type: Scalar, Table-valued, etc.
    pub function_type: String,
    /// Parameters
    pub parameters: Vec<ProcedureParameter>,
    /// Return type
    pub return_type: String,
    /// Description
    pub description: String,
    /// Source file path
    pub source_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TableRelationship {
    /// Source table (schema.table)
    pub from_table: String,
    /// Source column(s)
    pub from_columns: Vec<String>,
    /// Target table (schema.table)
    pub to_table: String,
    /// Target column(s)
    pub to_columns: Vec<String>,
    /// Relationship type: ForeignKey, Reference, Implicit
    pub relationship_type: String,
    /// Constraint name if explicit FK
    pub constraint_name: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DataFlow {
    /// Flow name/description
    pub name: String,
    /// Source (table, external system, or procedure)
    pub source: String,
    /// Destination (table, external system, or procedure)
    pub destination: String,
    /// Operations involved (INSERT, UPDATE, MERGE, etc.)
    pub operations: Vec<String>,
    /// Procedures involved in this flow
    pub procedures_involved: Vec<String>,
}

impl Default for DatabaseOverviewReport {
    fn default() -> Self {
        Self {
            database_projects: Vec::new(),
            tables: Vec::new(),
            views: Vec::new(),
            stored_procedures: Vec::new(),
            database_functions: Vec::new(),
            table_relationships: Vec::new(),
            data_flows: Vec::new(),
            confidence_score: 0.0,
        }
    }
}

// https://c4model.com/abstractions/software-system
// System name, project's role and value, system type, who is using it, how to use, which external systems it interacts with, diagram
