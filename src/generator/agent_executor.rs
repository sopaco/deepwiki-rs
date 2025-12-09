use anyhow::Result;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use toon_format::encode_default as toon_encode;

use crate::generator::context::GeneratorContext;
use crate::llm::client::utils::estimate_token_usage;

pub struct AgentExecuteParams {
    pub prompt_sys: String,
    pub prompt_user: String,
    pub cache_scope: String,
    pub log_tag: String,
}

pub async fn prompt(context: &GeneratorContext, params: AgentExecuteParams) -> Result<String> {
    let prompt_sys = &params.prompt_sys;
    let prompt_user = &params.prompt_user;
    let cache_scope = &params.cache_scope;
    let log_tag = &params.log_tag;

    let prompt_key = format!("{}|{}|reply-prompt", prompt_sys, prompt_user);
    // Try to get from cache - Use prompt directly as key, CacheManager will automatically calculate hash
    if let Some(cached_reply) = context
        .cache_manager
        .read()
        .await
        .get::<serde_json::Value>(cache_scope, &prompt_key)
        .await?
    {
        let msg = context.config.target_language.msg_cache_hit().replace("{}", log_tag);
        println!("{}", msg);
        return Ok(cached_reply.to_string());
    }

    let msg = context.config.target_language.msg_ai_analyzing().replace("{}", log_tag);
    println!("{}", msg);

    let reply = context
        .llm_client
        .prompt_without_react(prompt_sys, prompt_user)
        .await
        .map_err(|e| anyhow::anyhow!("AI analysis failed: {}", e))?;

    // Estimate token usage
    let input_text = format!("{} {}", prompt_sys, prompt_user);
    let token_usage = estimate_token_usage(&input_text, &reply);

    // Cache result - Use method with token information
    context
        .cache_manager
        .write()
        .await
        .set_with_tokens(cache_scope, &prompt_key, &reply, token_usage)
        .await?;

    Ok(reply)
}

pub async fn prompt_with_tools(
    context: &GeneratorContext,
    params: AgentExecuteParams,
) -> Result<String> {
    let prompt_sys = &params.prompt_sys;
    let prompt_user = &params.prompt_user;
    let cache_scope = &params.cache_scope;
    let log_tag = &params.log_tag;

    let prompt_key = format!("{}|{}|reply-prompt+tool", prompt_sys, prompt_user);
    // Try to get from cache - Use prompt directly as key, CacheManager will automatically calculate hash
    if let Some(cached_reply) = context
        .cache_manager
        .read()
        .await
        .get::<serde_json::Value>(cache_scope, &prompt_key)
        .await?
    {
        let msg = context.config.target_language.msg_cache_hit().replace("{}", log_tag);
        println!("{}", msg);
        return Ok(cached_reply.to_string());
    }

    let msg = context.config.target_language.msg_ai_analyzing().replace("{}", log_tag);
    println!("{}", msg);

    let reply = context
        .llm_client
        .prompt(prompt_sys, prompt_user)
        .await
        .map_err(|e| anyhow::anyhow!("AI analysis failed: {}", e))?;

    // Estimate token usage
    let input_text = format!("{} {}", prompt_sys, prompt_user);
    let output_text = serde_json::to_string(&reply).unwrap_or_default();
    let token_usage = estimate_token_usage(&input_text, &output_text);

    // Cache result - Use method with token information
    context
        .cache_manager
        .write()
        .await
        .set_with_tokens(cache_scope, &prompt_key, &reply, token_usage)
        .await?;

    Ok(reply)
}

pub async fn extract<T>(context: &GeneratorContext, params: AgentExecuteParams) -> Result<T>
where
    T: JsonSchema + for<'a> Deserialize<'a> + Serialize + Send + Sync + 'static,
{
    let prompt_sys = &params.prompt_sys;
    let prompt_user = &params.prompt_user;
    let cache_scope = &params.cache_scope;
    let log_tag = &params.log_tag;

    let prompt_key = format!("{}|{}", prompt_sys, prompt_user);
    // Try to get from cache - Use prompt directly as key, CacheManager will automatically calculate hash
    if let Some(cached_reply) = context
        .cache_manager
        .read()
        .await
        .get::<T>(cache_scope, &prompt_key)
        .await?
    {
        let msg = context.config.target_language.msg_cache_hit().replace("{}", log_tag);
        println!("{}", msg);
        return Ok(cached_reply);
    }

    let msg = context.config.target_language.msg_ai_analyzing().replace("{}", log_tag);
    println!("{}", msg);

    let reply = context
        .llm_client
        .extract::<T>(prompt_sys, prompt_user)
        .await
        .map_err(|e| anyhow::anyhow!("AI analysis failed: {}", e))?;

    // Estimate token usage
    let input_text = format!("{} {}", prompt_sys, prompt_user);
    let output_text = toon_encode(&reply).unwrap_or_default();
    let token_usage = estimate_token_usage(&input_text, &output_text);

    // Cache result - Use method with token information
    context
        .cache_manager
        .write()
        .await
        .set_with_tokens(cache_scope, &prompt_key, &reply, token_usage)
        .await?;

    Ok(reply)
}
