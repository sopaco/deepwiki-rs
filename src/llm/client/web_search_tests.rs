//! Scripted model tests exercise the production resolver, tool loop and MCP transport.

use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::time::Duration;

use clap::Parser;
use serde_json::{Value, json};
use wiremock::{
    Mock, MockServer, Request, Respond, ResponseTemplate,
    matchers::{method, path},
};

use super::LLMClient;
use crate::config::Config;

const SOURCE: &str = "https://docs.rs/rig-core/latest/rig/";

struct Mcp(&'static str);

impl Respond for Mcp {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        let result = match body["method"].as_str().unwrap() {
            "initialize" => json!({
                "protocolVersion":body["params"]["protocolVersion"],
                "capabilities":{"tools":{}},
                "serverInfo":{"name":"test-search","version":"1"}
            }),
            "notifications/initialized" | "notifications/cancelled" => {
                return ResponseTemplate::new(202);
            }
            "tools/list" if self.0 == "missing_tools" => json!({"tools":[]}),
            "tools/list" => json!({"tools":[
                {"name":"web_search","inputSchema":{"type":"object"}},
                {"name":"web_fetch","inputSchema":{"type":"object"}}
            ]}),
            "tools/call" => {
                if self.0 == "http_error" {
                    return ResponseTemplate::new(503);
                }
                if self.0 == "malformed" {
                    return ResponseTemplate::new(200)
                        .set_body_raw("invalid JSON", "application/json");
                }
                if self.0 == "rpc_error" {
                    return ResponseTemplate::new(200).set_body_json(json!({"jsonrpc":"2.0","id":body["id"],"error":{"code":-32602,"message":"Invalid search"}}));
                }
                json!({"isError": self.0 == "tool_error", "content":[{"type":"text","text":
                    if self.0 == "tool_error" { "Quota exceeded: JsonError ApiResponse".to_owned() }
                    else { json!({"results":[{"url":SOURCE,"excerpts":["Rig agents execute registered tools."]}]}).to_string() }
                }]})
            }
            other => panic!("Unexpected MCP method: {other}"),
        };
        let response = ResponseTemplate::new(200)
            .set_body_json(json!({"jsonrpc":"2.0","id":body["id"],"result":result}));
        if self.0 == "delay" && body["method"] == "tools/call" {
            response.set_delay(Duration::from_secs(2))
        } else if self.0 == "sse" {
            response.set_body_raw(
                format!(
                    "event: message\ndata: {}\n\n",
                    json!({"jsonrpc":"2.0","id":body["id"],"result":result})
                ),
                "text/event-stream",
            )
        } else {
            response
        }
    }
}

struct Model {
    retry_once: Arc<AtomicBool>,
    live: bool,
}

impl Respond for Model {
    fn respond(&self, request: &Request) -> ResponseTemplate {
        let body: Value = serde_json::from_slice(&request.body).unwrap();
        assert_eq!(body["model"], "scripted-model");
        let messages = body["messages"].as_array().unwrap();
        let user = messages.iter().find(|m| m["role"] == "user").unwrap()["content"].to_string();
        let results: Vec<_> = messages.iter().filter(|m| m["role"] == "tool").collect();
        let tools = body["tools"].as_array();
        let has_search =
            tools.is_some_and(|t| t.iter().any(|t| t["function"]["name"] == "web_search"));
        // Rig forwards tool errors into the conversation so the model can explain them.
        let tool_failed = results.last().is_some_and(|result| {
            result["content"].as_str().is_some_and(|text| {
                serde_json::from_str::<rmcp::model::CallToolResult>(text).is_err()
            }) || (!self.live && !result.to_string().contains(SOURCE))
        });
        let message = if !has_search {
            json!({"role":"assistant","content":"No web tools available"})
        } else if tool_failed {
            json!({"role":"assistant","content":format!("Web tool failed: {}", results.last().unwrap())})
        } else if results.len() == 3 {
            if self.live {
                json!({"role":"assistant","content":results.iter().map(|r| r.to_string()).collect::<Vec<_>>().join("\n")})
            } else {
                for result in &results {
                    assert!(
                        result.to_string().contains(SOURCE),
                        "Tool result must reach the next model call"
                    );
                }
                json!({"role":"assistant","content":format!("Source: {SOURCE}. Rig agents execute registered tools.")})
            }
        } else {
            if user.contains("retry")
                && results.len() == 1
                && self.retry_once.swap(false, Ordering::SeqCst)
            {
                return ResponseTemplate::new(500).set_body_json(
                    json!({"error":{"message":"Retry this model request","type":"server_error"}}),
                );
            }
            let (name, args) = if user.contains("invalid-url") {
                ("web_fetch", json!({"urls":["file:///private/repository"]}))
            } else if user.contains("empty-query") {
                (
                    "web_search",
                    json!({"objective":"Read public docs","search_queries":[]}),
                )
            } else if results.len() == 1 {
                (
                    "web_fetch",
                    json!({"urls":[SOURCE],"objective":"Read the Rig tool documentation"}),
                )
            } else {
                (
                    "web_search",
                    json!({"objective":user,"search_queries":["rig agent tool documentation"]}),
                )
            };
            json!({"role":"assistant","content":null,"tool_calls":[{
                "id":format!("call-{}",results.len()),"type":"function",
                "function":{"name":name,"arguments":args.to_string()}
            }]})
        };
        ResponseTemplate::new(200).set_body_json(json!({
            "id":"scripted-completion","object":"chat.completion","created":0,"model":"scripted-model",
            "choices":[{"index":0,"message":message,"finish_reason":if has_search && !tool_failed && results.len()<3 {"tool_calls"} else {"stop"}}],
            "usage":{"prompt_tokens":10,"completion_tokens":10,"total_tokens":20}
        }))
    }
}

async fn setup(mode: &'static str) -> (Config, MockServer, MockServer) {
    let mcp = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/mcp"))
        .respond_with(Mcp(mode))
        .mount(&mcp)
        .await;
    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(405))
        .mount(&mcp)
        .await;
    let model = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(Model {
            retry_once: Arc::new(AtomicBool::new(true)),
            live: false,
        })
        .mount(&model)
        .await;
    let mut config = Config::default();
    config.llm.api_key = "scripted-model-key".into();
    config.llm.api_base_url = format!("{}/v1", model.uri());
    config.llm.model_efficient = "scripted-model".into();
    config.llm.model_powerful = "scripted-model".into();
    config.llm.retry_attempts = 1;
    config.llm.retry_delay_ms = 0;
    config.llm.timeout_seconds = 1;
    config.llm.web_search_mcp_url = format!("{}/mcp", mcp.uri());
    (config, mcp, model)
}

async fn calls(mcp: &MockServer) -> Vec<Value> {
    let requests = mcp.received_requests().await.unwrap();
    for request in &requests {
        assert_eq!(request.url.path(), "/mcp");
        assert_eq!(
            request.headers["user-agent"],
            concat!("deepwiki-rs/", env!("CARGO_PKG_VERSION"))
        );
        assert!(!request.headers.contains_key("authorization"));
    }
    requests
        .iter()
        .filter_map(|r| serde_json::from_slice::<Value>(&r.body).ok())
        .filter(|b| b["method"] == "tools/call")
        .collect()
}

#[tokio::test]
async fn web_search_agent_loop_isolates_concurrent_and_new_runs() {
    let (config, mcp, _model) = setup("normal").await;
    let client = LLMClient::new(config).unwrap();
    let (a, b) = tokio::join!(
        client.prompt("Use public documentation", "run-a"),
        client.prompt("Use public documentation", "run-b")
    );
    assert!(a.unwrap().contains(SOURCE));
    assert!(b.unwrap().contains(SOURCE));
    assert!(
        client
            .prompt("Use public documentation", "run-c")
            .await
            .unwrap()
            .contains(SOURCE)
    );
    let calls = calls(&mcp).await;
    assert_eq!(calls.len(), 9);
    let mut sessions = std::collections::HashSet::new();
    for run in ["run-a", "run-b", "run-c"] {
        let searches: Vec<_> = calls
            .iter()
            .filter(|c| {
                c["params"]["arguments"]["objective"]
                    .as_str()
                    .is_some_and(|s| s.contains(run))
            })
            .collect();
        assert_eq!(searches.len(), 2);
        let id = searches[0]["params"]["arguments"]["session_id"]
            .as_str()
            .unwrap();
        uuid::Uuid::parse_str(id).unwrap();
        assert_eq!(searches[1]["params"]["arguments"]["session_id"], id);
        let related: Vec<_> = calls
            .iter()
            .filter(|c| c["params"]["arguments"]["session_id"] == id)
            .collect();
        assert_eq!(related.len(), 3);
        assert_eq!(related[1]["params"]["name"], "web_fetch");
        assert_eq!(related[1]["params"]["arguments"]["urls"][0], SOURCE);
        assert!(sessions.insert(id.to_owned()));
    }
    let requests = mcp.received_requests().await.unwrap();
    assert_eq!(
        requests
            .iter()
            .filter(|r| String::from_utf8_lossy(&r.body).contains("\"method\":\"initialize\""))
            .count(),
        1
    );
}

#[tokio::test]
async fn web_search_retry_keeps_the_conversation_id() {
    let (mut config, mcp, _model) = setup("normal").await;
    config.llm.retry_attempts = 2;
    let client = LLMClient::new(config).unwrap();
    assert!(
        client
            .prompt("Use public documentation", "run-retry")
            .await
            .unwrap()
            .contains(SOURCE)
    );
    let calls = calls(&mcp).await;
    assert!(
        calls.len() >= 4,
        "Must execute a tool before the failed model turn, then retry"
    );
    let id = &calls[0]["params"]["arguments"]["session_id"];
    assert!(
        calls
            .iter()
            .all(|c| &c["params"]["arguments"]["session_id"] == id)
    );
}

#[tokio::test]
async fn web_search_permissions_and_saved_settings() {
    let (mut config, mcp, model) = setup("normal").await;
    let saved = toml::to_string(&config).unwrap();
    let legacy = saved
        .lines()
        .filter(|line| {
            !line.starts_with("disable_web_search") && !line.starts_with("web_search_mcp_url")
        })
        .collect::<Vec<_>>()
        .join("\n");
    let legacy: Config = toml::from_str(&legacy).unwrap();
    assert!(!legacy.llm.disable_web_search);
    assert_eq!(
        legacy.llm.web_search_mcp_url,
        crate::llm::tools::parallel::DEFAULT_MCP_URL
    );
    assert_eq!(legacy.llm.api_key, config.llm.api_key);
    assert_eq!(legacy.llm.api_base_url, config.llm.api_base_url);
    assert_eq!(legacy.llm.provider, config.llm.provider);

    config.llm.disable_web_search = true;
    config.llm.web_search_mcp_url = "invalid disabled endpoint".into();
    let disabled: Config = toml::from_str(&toml::to_string(&config).unwrap()).unwrap();
    assert!(disabled.llm.disable_web_search);
    let client = LLMClient::new(disabled).unwrap();
    assert_eq!(
        client
            .prompt("Analyze local files", "offline")
            .await
            .unwrap(),
        "No web tools available"
    );
    let requests = model.received_requests().await.unwrap();
    let body: Value = serde_json::from_slice(&requests[0].body).unwrap();
    assert!(
        body["tools"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["function"]["name"] == "file_reader")
    );

    config.llm.disable_web_search = false;
    config.llm.disable_preset_tools = true;
    assert_eq!(
        LLMClient::new(config)
            .unwrap()
            .prompt("Analyze local files", "offline")
            .await
            .unwrap(),
        "No web tools available"
    );
    let (config, _mcp2, _model2) = setup("normal").await;
    assert_eq!(
        LLMClient::new(config)
            .unwrap()
            .prompt_without_react("No tools", "offline")
            .await
            .unwrap(),
        "No web tools available"
    );
    assert!(mcp.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn web_search_cli_preserves_saved_permissions_and_endpoint() {
    let (mut config, mcp, _model) = setup("normal").await;
    let dir = std::env::temp_dir().join(format!("deepwiki-cli-{}", uuid::Uuid::new_v4()));
    std::fs::create_dir(&dir).unwrap();
    let path = dir.join("litho.toml");
    for (saved_disabled, web_disabled, flag) in [
        (true, false, false),
        (false, false, true),
        (false, true, false),
        (false, false, false),
    ] {
        config.llm.disable_preset_tools = saved_disabled;
        config.llm.disable_web_search = web_disabled;
        std::fs::write(&path, toml::to_string(&config).unwrap()).unwrap();
        let mut args = vec![
            "deepwiki-rs".to_owned(),
            "--config".into(),
            path.to_str().unwrap().to_owned(),
        ];
        if flag {
            args.push("--disable-preset-tools".into());
        }
        let loaded = crate::cli::Args::parse_from(args).to_config();
        assert_eq!(loaded.llm.disable_preset_tools, saved_disabled || flag);
        assert_eq!(loaded.llm.disable_web_search, web_disabled);
        assert_eq!(loaded.llm.web_search_mcp_url, config.llm.web_search_mcp_url);
        assert_eq!(loaded.llm.api_key, config.llm.api_key);
        assert_eq!(loaded.llm.api_base_url, config.llm.api_base_url);
        assert_eq!(loaded.llm.model_efficient, config.llm.model_efficient);
        let answer = LLMClient::new(loaded)
            .unwrap()
            .prompt("Use public documentation", "run-cli")
            .await
            .unwrap();
        if saved_disabled || web_disabled || flag {
            assert_eq!(answer, "No web tools available");
            assert!(mcp.received_requests().await.unwrap().is_empty());
        } else {
            assert!(answer.contains(SOURCE));
            assert_eq!(calls(&mcp).await.len(), 3);
        }
    }
    std::fs::remove_dir_all(dir).unwrap();
}

#[tokio::test]
async fn web_search_mcp_errors_and_timeout_reach_the_caller() {
    for mode in [
        "tool_error",
        "rpc_error",
        "http_error",
        "malformed",
        "delay",
    ] {
        let (config, mcp, _model) = setup(mode).await;
        let result = LLMClient::new(config)
            .unwrap()
            .prompt("Use public documentation", "run-error")
            .await;
        let answer = result.unwrap();
        assert!(answer.starts_with("Web tool failed:"), "{mode}: {answer}");
        if mode == "delay" {
            assert!(answer.contains("timed out"));
        }
        assert_eq!(calls(&mcp).await.len(), 1);
    }
}

#[tokio::test]
async fn web_search_invalid_arguments_never_reach_mcp() {
    let (config, mcp, _model) = setup("normal").await;
    let client = LLMClient::new(config).unwrap();
    for prompt in ["invalid-url", "empty-query"] {
        let answer = client
            .prompt("Use public documentation", prompt)
            .await
            .unwrap();
        assert!(answer.starts_with("Web tool failed:"));
    }
    assert!(mcp.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn web_search_discovery_requires_both_tools_at_the_configured_endpoint() {
    let (config, mcp, _model) = setup("missing_tools").await;
    assert!(
        LLMClient::new(config)
            .unwrap()
            .prompt("Use public documentation", "run-error")
            .await
            .unwrap()
            .contains("missing web_search")
    );
    assert!(calls(&mcp).await.is_empty());
    assert!(!mcp.received_requests().await.unwrap().is_empty());
}

#[tokio::test]
async fn web_search_cancelling_the_caller_stops_the_agent_loop() {
    let (config, mcp, model) = setup("delay").await;
    let client = LLMClient::new(config).unwrap();
    let concurrent_client = client.clone();
    let task = tokio::spawn(async move {
        concurrent_client
            .prompt("Use public documentation", "run-cancel")
            .await
    });
    tokio::time::timeout(Duration::from_secs(2), async {
        loop {
            if !calls(&mcp).await.is_empty() {
                break;
            }
            tokio::task::yield_now().await;
        }
    })
    .await
    .unwrap();
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(model.received_requests().await.unwrap().len(), 1);
    assert_eq!(
        client
            .prompt_without_react("No tools", "offline")
            .await
            .unwrap(),
        "No web tools available"
    );
}

#[tokio::test]
async fn web_search_streamable_http_sse_responses() {
    let (config, mcp, _model) = setup("sse").await;
    assert!(
        LLMClient::new(config)
            .unwrap()
            .prompt("Use public documentation", "run-sse")
            .await
            .unwrap()
            .contains(SOURCE)
    );
    assert_eq!(calls(&mcp).await.len(), 3);
}

#[tokio::test]
#[ignore = "Makes live anonymous Parallel MCP requests using a scripted model"]
async fn web_search_live_anonymous() {
    let (mut config, _mcp, model) = setup("normal").await;
    model.reset().await;
    Mock::given(method("POST"))
        .and(path("/v1/chat/completions"))
        .respond_with(Model {
            retry_once: Arc::new(AtomicBool::new(false)),
            live: true,
        })
        .mount(&model)
        .await;
    config.llm.web_search_mcp_url = crate::llm::tools::parallel::DEFAULT_MCP_URL.into();
    config.llm.timeout_seconds = 60;
    let result = LLMClient::new(config)
        .unwrap()
        .prompt(
            "Use public documentation",
            "Find public Rig framework tool documentation",
        )
        .await;
    println!("Live anonymous MCP through scripted model: {result:?}");
    let answer = result.unwrap();
    assert!(!answer.starts_with("Web tool failed:"), "{answer}");
    assert!(answer.contains("results"));
    assert!(answer.contains("https://"));
}
