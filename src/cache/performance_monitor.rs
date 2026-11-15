use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::Duration;

use crate::i18n::TargetLanguage;
use crate::llm::client::types::TokenUsage;

/// Cache performance monitor
#[derive(Clone)]
pub struct CachePerformanceMonitor {
    metrics: Arc<CacheMetrics>,
    target_language: TargetLanguage,
}

/// Cache metrics
#[derive(Default)]
pub struct CacheMetrics {
    /// Cache hit count
    pub cache_hits: AtomicUsize,
    /// Cache miss count
    pub cache_misses: AtomicUsize,
    /// Cache write count
    pub cache_writes: AtomicUsize,
    /// Cache error count
    pub cache_errors: AtomicUsize,
    /// Total inference time saved (seconds)
    pub total_inference_time_saved: AtomicU64,
    /// Total cost saved (estimated)
    pub total_cost_saved: AtomicUsize,
    /// Total input tokens saved
    pub total_input_tokens_saved: AtomicUsize,
    /// Total output tokens saved
    pub total_output_tokens_saved: AtomicUsize,
}

/// Cache performance report
#[derive(Debug, Serialize, Deserialize)]
pub struct CachePerformanceReport {
    /// Cache hit rate
    pub hit_rate: f64,
    /// Total cache operations
    pub total_operations: usize,
    /// Cache hit count
    pub cache_hits: usize,
    /// Cache miss count
    pub cache_misses: usize,
    /// Cache write count
    pub cache_writes: usize,
    /// Cache error count
    pub cache_errors: usize,
    /// Inference time saved (seconds)
    pub inference_time_saved: f64,
    /// Cost saved (USD, estimated)
    pub cost_saved: f64,
    /// Performance improvement percentage
    pub performance_improvement: f64,
    /// Input tokens saved
    pub input_tokens_saved: usize,
    /// Output tokens saved
    pub output_tokens_saved: usize,
    /// Category statistics
    pub category_stats: HashMap<String, CategoryPerformanceStats>,
}

/// Category performance statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoryPerformanceStats {
    pub hits: u64,
    pub misses: u64,
    pub hit_rate: f64,
    pub time_saved: f64,
    pub cost_saved: f64,
}

impl CachePerformanceMonitor {
    pub fn new(target_language: TargetLanguage) -> Self {
        Self {
            metrics: Arc::new(CacheMetrics::default()),
            target_language,
        }
    }

    /// Record cache hit
    pub fn record_cache_hit(
        &self,
        category: &str,
        inference_time_saved: Duration,
        token_usage: TokenUsage,
        model_name: &str,
    ) {
        self.metrics.cache_hits.fetch_add(1, Ordering::Relaxed);
        self.metrics
            .total_inference_time_saved
            .fetch_add(inference_time_saved.as_millis() as u64, Ordering::Relaxed);

        // Record saved token count
        self.metrics
            .total_input_tokens_saved
            .fetch_add(token_usage.input_tokens, Ordering::Relaxed);
        self.metrics
            .total_output_tokens_saved
            .fetch_add(token_usage.output_tokens, Ordering::Relaxed);

        // Calculate saved cost based on actual token usage
        let estimated_cost_saved = token_usage.estimate_cost(model_name);
        self.metrics.total_cost_saved.fetch_add(
            (estimated_cost_saved * 1_000_000.0).round() as usize, // Store as micro-dollars
            Ordering::Relaxed,
        );

        // Use localized message for cache hit with detailed statistics
        let msg = match &self.target_language {
            TargetLanguage::Chinese => format!(
                "   💰 缓存命中 [{}] - 节省推理时间: {:.2}秒, 节省tokens: {}输入+{}输出, 估算节省成本: ${:.4}",
                category, inference_time_saved.as_secs_f64(), token_usage.input_tokens, token_usage.output_tokens, estimated_cost_saved
            ),
            _ => format!(
                "   💰 Cache hit [{}] - Time saved: {:.2}s, Tokens saved: {} input + {} output, Cost saved: ${:.4}",
                category, inference_time_saved.as_secs_f64(), token_usage.input_tokens, token_usage.output_tokens, estimated_cost_saved
            ),
        };
        println!("{}", msg);
    }

    /// Record cache miss
    pub fn record_cache_miss(&self, category: &str) {
        self.metrics.cache_misses.fetch_add(1, Ordering::Relaxed);
        let msg = self.target_language.msg_cache_miss().replace("{}", category);
        println!("{}", msg);
    }

    /// Record cache write
    pub fn record_cache_write(&self, category: &str) {
        self.metrics.cache_writes.fetch_add(1, Ordering::Relaxed);
        let msg = self.target_language.msg_cache_write().replace("{}", category);
        println!("{}", msg);
    }

    /// Record cache error
    pub fn record_cache_error(&self, category: &str, error: &str) {
        self.metrics.cache_errors.fetch_add(1, Ordering::Relaxed);
        let msg = self.target_language.msg_cache_error()
            .replace("{}", category)
            .replacen("{}", error, 1);
        eprintln!("{}", msg);
    }

    /// Generate performance report
    pub fn generate_report(&self) -> CachePerformanceReport {
        let hits = self.metrics.cache_hits.load(Ordering::Relaxed);
        let misses = self.metrics.cache_misses.load(Ordering::Relaxed);
        let writes = self.metrics.cache_writes.load(Ordering::Relaxed);
        let errors = self.metrics.cache_errors.load(Ordering::Relaxed);
        let total_operations = hits + misses;

        let hit_rate = if total_operations > 0 {
            hits as f64 / total_operations as f64
        } else {
            0.0
        };

        let inference_time_saved = self
            .metrics
            .total_inference_time_saved
            .load(Ordering::Relaxed) as f64
            / 1000.0; // Convert to seconds
        let cost_saved = self.metrics.total_cost_saved.load(Ordering::Relaxed) as f64 / 1_000_000.0; // Convert to dollars

        let input_tokens_saved = self
            .metrics
            .total_input_tokens_saved
            .load(Ordering::Relaxed);
        let output_tokens_saved = self
            .metrics
            .total_output_tokens_saved
            .load(Ordering::Relaxed);

        let performance_improvement = if misses > 0 {
            (hits as f64 / (hits + misses) as f64) * 100.0
        } else {
            0.0
        };

        CachePerformanceReport {
            hit_rate,
            total_operations,
            cache_hits: hits,
            cache_misses: misses,
            cache_writes: writes,
            cache_errors: errors,
            inference_time_saved,
            cost_saved,
            performance_improvement,
            input_tokens_saved,
            output_tokens_saved,
            category_stats: HashMap::new(), // TODO: Implement category statistics
        }
    }
}

impl Default for CachePerformanceMonitor {
    fn default() -> Self {
        Self::new(TargetLanguage::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_saved_keeps_sub_cent_hits() {
        let monitor = CachePerformanceMonitor::default();
        // 2,000 in + 500 out on gpt-4o-mini is $0.0006, under one milli-dollar
        let usage = TokenUsage::new(2_000, 500);
        monitor.record_cache_hit("test", Duration::from_secs(1), usage.clone(), "gpt-4o-mini");
        monitor.record_cache_hit("test", Duration::from_secs(1), usage, "gpt-4o-mini");

        let report = monitor.generate_report();
        assert!((report.cost_saved - 0.0012).abs() < 1e-9);
    }
}
