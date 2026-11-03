//! Anonymous Parallel Search MCP tools for one research conversation.

use std::{sync::Arc, time::Duration};

use anyhow::{Context, Result, ensure};
use rig::{completion::ToolDefinition, tool::Tool};
use rmcp::{
    RoleClient, ServiceExt,
    model::{
        CallToolRequestParams, CallToolResult, ClientCapabilities, Implementation,
        InitializeRequestParams,
    },
    service::RunningService,
    transport::{
        StreamableHttpClientTransport, streamable_http_client::StreamableHttpClientTransportConfig,
    },
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::sync::OnceCell;

pub const DEFAULT_MCP_URL: &str = "https://search.parallel.ai/mcp";
const USER_AGENT: &str = concat!("deepwiki-rs/", env!("CARGO_PKG_VERSION"));

/// Shared transport only. Conversation identity belongs to `ParallelConversation`.
pub struct ParallelClient {
    http: mcp_http::Client,
    url: String,
    timeout: Duration,
    service: OnceCell<RunningService<RoleClient, InitializeRequestParams>>,
}

impl ParallelClient {
    pub fn new(url: &str, timeout_seconds: u64) -> Result<Self> {
        let url = mcp_http::Url::parse(url).context("Invalid web search MCP URL")?;
        ensure!(
            matches!(url.scheme(), "http" | "https"),
            "Web search MCP URL must use HTTP or HTTPS"
        );
        ensure!(
            url.username().is_empty() && url.password().is_none(),
            "Web search MCP URL must not contain credentials"
        );
        let timeout = Duration::from_secs(timeout_seconds.max(1));
        let http = mcp_http::Client::builder()
            .user_agent(USER_AGENT)
            .connect_timeout(timeout)
            .timeout(timeout)
            .redirect(mcp_http::redirect::Policy::none())
            .build()?;
        Ok(Self {
            http,
            url: url.into(),
            timeout,
            service: OnceCell::new(),
        })
    }

    async fn service(&self) -> Result<&RunningService<RoleClient, InitializeRequestParams>> {
        self.service
            .get_or_try_init(|| async {
                let transport = StreamableHttpClientTransport::with_client(
                    self.http.clone(),
                    StreamableHttpClientTransportConfig::with_uri(self.url.clone()),
                );
                let info = InitializeRequestParams::new(
                    ClientCapabilities::default(),
                    Implementation::new("deepwiki-rs", env!("CARGO_PKG_VERSION")),
                );
                let service = info
                    .serve(transport)
                    .await
                    .context("Connect to web search MCP")?;
                let tools = service
                    .list_all_tools()
                    .await
                    .context("Discover web search MCP tools")?;
                for name in ["web_search", "web_fetch"] {
                    ensure!(
                        tools.iter().any(|tool| tool.name == name),
                        "Web search MCP is missing {name}"
                    );
                }
                Ok(service)
            })
            .await
    }
}

#[derive(Clone)]
pub struct ParallelConversation {
    client: Arc<ParallelClient>,
    session_id: String,
}

impl ParallelConversation {
    /// Called once when building the agent for a standalone prompt, before retries.
    pub fn new(client: Arc<ParallelClient>) -> Self {
        Self {
            client,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }

    async fn call(
        &self,
        name: &str,
        args: impl Serialize,
    ) -> Result<CallToolResult, ParallelError> {
        let operation = async {
            let mut args = serde_json::to_value(args)?;
            args["session_id"] = json!(self.session_id);
            let service = self.client.service().await?;
            let result = service
                .call_tool(
                    CallToolRequestParams::new(name.to_owned()).with_arguments(
                        args.as_object()
                            .context("Invalid web tool arguments")?
                            .clone(),
                    ),
                )
                .await
                .context("Web search MCP tool call failed")?;
            ensure!(
                result.is_error != Some(true),
                "Web search MCP returned a tool error: {}",
                serde_json::to_string(&result)?
            );
            Ok(result)
        };
        tokio::time::timeout(self.client.timeout, operation)
            .await
            .map_err(|_| ParallelError(anyhow::anyhow!("Web search MCP timed out")))?
            .map_err(ParallelError)
    }
}

#[derive(Debug, thiserror::Error)]
#[error("{0:#}")]
pub struct ParallelError(#[from] anyhow::Error);

#[derive(Debug, Deserialize, Serialize)]
pub struct SearchArgs {
    pub objective: String,
    pub search_queries: Vec<String>,
}

#[derive(Clone)]
pub struct WebSearch(pub ParallelConversation);

impl Tool for WebSearch {
    const NAME: &'static str = "web_search";
    type Error = ParallelError;
    type Args = SearchArgs;
    type Output = CallToolResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Search public web documentation with Parallel. Use only for external framework/API facts, never private repository content. Prefer official sources, cite returned URLs, and ground repository-specific claims in local files. Excerpts are usually sufficient without fetching each result.".into(),
            parameters: json!({"type":"object","properties":{
                "objective":{"type":"string","description":"One focused public-information research goal."},
                "search_queries":{"type":"array","items":{"type":"string"},"minItems":1,"description":"Related keyword queries, preferably 3–6 words each."}
            },"required":["objective","search_queries"]}),
        }
    }

    async fn call(&self, args: SearchArgs) -> Result<CallToolResult, ParallelError> {
        validate_search(&args)?;
        self.0.call(Self::NAME, args).await
    }
}

#[derive(Debug, Deserialize, Serialize)]
pub struct FetchArgs {
    pub urls: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub objective: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub search_queries: Option<Vec<String>>,
}

#[derive(Clone)]
pub struct WebFetch(pub ParallelConversation);

impl Tool for WebFetch {
    const NAME: &'static str = "web_fetch";
    type Error = ParallelError;
    type Args = FetchArgs;
    type Output = CallToolResult;

    async fn definition(&self, _prompt: String) -> ToolDefinition {
        ToolDefinition {
            name: Self::NAME.into(),
            description: "Fetch public HTTP(S) documentation with Parallel when search excerpts are insufficient or a specific page is requested. Returns relevant excerpts and source URLs. Never send private URLs or repository content.".into(),
            parameters: json!({"type":"object","properties":{
                "urls":{"type":"array","items":{"type":"string"},"minItems":1,"maxItems":20},
                "objective":{"type":"string","maxLength":200},
                "search_queries":{"type":"array","items":{"type":"string"}}
            },"required":["urls"]}),
        }
    }

    async fn call(&self, args: FetchArgs) -> Result<CallToolResult, ParallelError> {
        validate_fetch(&args)?;
        self.0.call(Self::NAME, args).await
    }
}

fn validate_search(args: &SearchArgs) -> Result<()> {
    ensure!(
        !args.objective.trim().is_empty(),
        "Search objective must not be empty"
    );
    ensure!(
        !args.search_queries.is_empty() && args.search_queries.iter().all(|q| !q.trim().is_empty()),
        "At least one nonempty search query is required"
    );
    Ok(())
}

fn validate_fetch(args: &FetchArgs) -> Result<()> {
    ensure!(
        !args.urls.is_empty() && args.urls.len() <= 20,
        "Fetch requires 1–20 URLs"
    );
    ensure!(
        args.objective
            .as_ref()
            .is_none_or(|s| s.chars().count() <= 200),
        "Fetch objective must be at most 200 characters"
    );
    for url in &args.urls {
        let url = mcp_http::Url::parse(url).context("Invalid fetch URL")?;
        ensure!(
            matches!(url.scheme(), "http" | "https") && url.host_str().is_some(),
            "Fetch URLs must use HTTP or HTTPS"
        );
        ensure!(
            url.username().is_empty() && url.password().is_none(),
            "Fetch URLs must not contain credentials"
        );
    }

    Ok(())
}
