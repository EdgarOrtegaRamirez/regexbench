/// Benchmarking module for regex patterns
///
/// Measures performance of regex operations.
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Configuration for a benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkConfig {
    /// Number of iterations
    pub iterations: usize,
    /// Input strings to test
    pub inputs: Vec<String>,
    /// Whether to warm up (run some iterations first)
    pub warmup: bool,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            iterations: 1000,
            inputs: vec!["test string".to_string()],
            warmup: true,
        }
    }
}

/// Result of a benchmark run
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BenchmarkResult {
    /// The pattern that was benchmarked
    pub pattern: String,
    /// Total iterations
    pub iterations: usize,
    /// Total time in microseconds
    pub total_time_us: u64,
    /// Average time per iteration in nanoseconds
    pub avg_time_ns: f64,
    /// Minimum time per iteration in nanoseconds
    pub min_time_ns: f64,
    /// Maximum time per iteration in nanoseconds
    pub max_time_ns: f64,
    /// Operations per second
    pub ops_per_sec: f64,
    /// Whether all inputs matched
    pub all_matched: bool,
}

/// Benchmark runner
pub struct BenchmarkRunner;

impl BenchmarkRunner {
    /// Run a benchmark with the given pattern and config
    pub fn run(pattern: &str, config: &BenchmarkConfig) -> crate::Result<BenchmarkResult> {
        let engine = crate::engine::RegexEngine::new(pattern)?;

        // Warmup
        if config.warmup {
            for _ in 0..100 {
                for input in &config.inputs {
                    engine.find_match(input, 0);
                }
            }
        }

        // Benchmark
        let mut durations = Vec::new();
        let mut all_matched = true;

        for _ in 0..config.iterations {
            let start = Instant::now();
            for input in &config.inputs {
                let result = engine.find_match(input, 0);
                if result.is_none() {
                    all_matched = false;
                }
            }
            durations.push(start.elapsed().as_nanos() as u64);
        }

        let total_time: u64 = durations.iter().sum();
        let avg = total_time as f64 / config.iterations as f64;
        let min = *durations.iter().min().unwrap_or(&0) as f64;
        let max = *durations.iter().max().unwrap_or(&0) as f64;
        let ops_per_sec = if total_time > 0 {
            config.iterations as f64 / (total_time as f64 / 1_000_000_000.0)
        } else {
            0.0
        };

        Ok(BenchmarkResult {
            pattern: pattern.to_string(),
            iterations: config.iterations,
            total_time_us: total_time / 1000,
            avg_time_ns: avg,
            min_time_ns: min,
            max_time_ns: max,
            ops_per_sec,
            all_matched,
        })
    }

    /// Compare multiple patterns
    pub fn compare(
        patterns: &[&str],
        config: &BenchmarkConfig,
    ) -> crate::Result<Vec<BenchmarkResult>> {
        patterns.iter().map(|p| Self::run(p, config)).collect()
    }
}

impl BenchmarkResult {
    /// Format as human-readable string
    pub fn format(&self) -> String {
        format!(
            "Pattern: {}\n\
             Iterations: {}\n\
             Total: {:.2?}\n\
             Avg: {:.2?}\n\
             Min: {:.2?}\n\
             Max: {:.2?}\n\
             Ops/sec: {:.0}\n\
             All matched: {}",
            self.pattern,
            self.iterations,
            std::time::Duration::from_micros(self.total_time_us),
            std::time::Duration::from_nanos(self.avg_time_ns as u64),
            std::time::Duration::from_nanos(self.min_time_ns as u64),
            std::time::Duration::from_nanos(self.max_time_ns as u64),
            self.ops_per_sec,
            self.all_matched,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_benchmark() {
        let config = BenchmarkConfig {
            iterations: 100,
            inputs: vec!["test".to_string()],
            warmup: false,
        };

        let result = BenchmarkRunner::run(r"test", &config).unwrap();
        assert_eq!(result.iterations, 100);
        assert!(result.all_matched);
        assert!(result.avg_time_ns > 0.0);
    }

    #[test]
    fn test_compare_patterns() {
        let config = BenchmarkConfig {
            iterations: 50,
            inputs: vec!["hello world".to_string()],
            warmup: false,
        };

        let results = BenchmarkRunner::compare(&[r"hello", r"world", r"h.*o"], &config).unwrap();
        assert_eq!(results.len(), 3);
    }
}
