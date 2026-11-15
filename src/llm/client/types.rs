use serde::{Deserialize, Serialize};

/// Token usage information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    /// Number of input tokens
    pub input_tokens: usize,
    /// Number of output tokens
    pub output_tokens: usize,
    /// Total number of tokens
    pub total_tokens: usize,
}

impl TokenUsage {
    pub fn new(input_tokens: usize, output_tokens: usize) -> Self {
        Self {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens + output_tokens,
        }
    }

    /// Estimate cost in USD using the model's own price.
    ///
    /// Prices come from the `llm-budget` table (OpenAI, Anthropic, Gemini,
    /// Mistral, DeepSeek, xAI). Models not in the table, such as self-hosted
    /// ones, fall back to a flat default rate.
    pub fn estimate_cost(&self, model_name: &str) -> f64 {
        if let Some(price) = llm_budget::PriceTable::builtin().get(model_name) {
            return price.cost(self.input_tokens as u64, self.output_tokens as u64);
        }

        let (input_cost_per_1k, output_cost_per_1k) = (0.00025, 0.002);

        (self.input_tokens as f64 / 1000.0) * input_cost_per_1k
            + (self.output_tokens as f64 / 1000.0) * output_cost_per_1k
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn estimate_cost_uses_model_price() {
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        // gpt-4o: $2.50 in / $10.00 out per million tokens
        assert!((usage.estimate_cost("gpt-4o") - 12.5).abs() < 1e-9);
        // OpenRouter-style provider prefix is accepted
        assert!((usage.estimate_cost("openai/gpt-4o") - 12.5).abs() < 1e-9);
        // gpt-4o-mini: $0.15 / $0.60
        assert!((usage.estimate_cost("gpt-4o-mini") - 0.75).abs() < 1e-9);
    }

    #[test]
    fn estimate_cost_falls_back_for_unknown_model() {
        let usage = TokenUsage::new(1_000_000, 1_000_000);
        let expected = 0.25 + 2.0;
        assert!((usage.estimate_cost("Qwen/Qwen3-Next-80B-A3B-Instruct") - expected).abs() < 1e-9);
        assert!((usage.estimate_cost("") - expected).abs() < 1e-9);
    }
}
