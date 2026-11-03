//! Agent builder - Responsible for building and configuring LLM Agent

use crate::llm::tools::parallel::{ParallelClient, ParallelConversation};
use std::sync::Arc;

use crate::{
    config::Config,
    llm::client::providers::{ProviderAgent, ProviderClient},
    llm::tools::{file_explorer::AgentToolFileExplorer, file_reader::AgentToolFileReader},
};

/// Agent builder
pub struct AgentBuilder<'a> {
    client: &'a ProviderClient,
    config: &'a Config,
    web_search: Option<&'a Arc<ParallelClient>>,
}

impl<'a> AgentBuilder<'a> {
    /// Create a new Agent builder
    pub fn new(
        client: &'a ProviderClient,
        config: &'a Config,
        web_search: Option<&'a Arc<ParallelClient>>,
    ) -> Self {
        Self {
            client,
            config,
            web_search,
        }
    }

    /// Build Agent with built-in preset tools
    pub fn build_agent_with_tools(&self, system_prompt: &str, model: &str) -> ProviderAgent {
        let llm_config = &self.config.llm;

        if !llm_config.disable_preset_tools {
            let file_explorer = AgentToolFileExplorer::new(self.config.clone());
            let file_reader = AgentToolFileReader::new(self.config.clone());

            let system_prompt_with_tools = format!(
                "{}\nDo not fabricate non-existent code. If you need to learn more about the project structure and source code content, actively call tools to obtain more contextual information",
                system_prompt
            );

            let web_search = self
                .web_search
                .map(|client| ParallelConversation::new(client.clone()));
            self.client.create_agent_with_tools(
                model,
                &system_prompt_with_tools,
                llm_config,
                &file_explorer,
                &file_reader,
                web_search.as_ref(),
            )
        } else {
            self.client.create_agent(model, system_prompt, llm_config)
        }
    }

    /// Build Agent without tools
    pub fn build_agent_without_tools(&self, system_prompt: &str, model: &str) -> ProviderAgent {
        let llm_config = &self.config.llm;
        self.client.create_agent(model, system_prompt, llm_config)
    }
}
