use serde::{Deserialize, Serialize};
use std::fmt::Display;

/// Agent type enumeration
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AgentType {
    Overview,
    Architecture,
    Workflow,
    Boundary,
    Database,
}

impl Display for AgentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = match self {
            AgentType::Overview => "Project Overview",
            AgentType::Architecture => "Architecture Description",
            AgentType::Workflow => "Core Workflows",
            AgentType::Boundary => "Boundary Interfaces",
            AgentType::Database => "Database Overview",
        };
        write!(f, "{}", str)
    }
}
