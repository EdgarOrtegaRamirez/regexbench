//! RegexBench - Regex Testing, Debugging & Benchmarking CLI
//!
//! This library provides regex pattern testing, performance benchmarking,
//! catastrophic backtracking detection, and cross-language export.

pub type Result<T> = anyhow::Result<T>;

pub mod analyzer;
pub mod ast;
pub mod benchmark;
pub mod dfa;
pub mod engine;
pub mod exporter;
pub mod nfa;
pub mod parser;
pub mod test_runner;
pub mod visualize;

pub use analyzer::{BacktrackRisk, PatternAnalysis};
pub use ast::{AstNode, RegexAst};
pub use benchmark::{BenchmarkConfig, BenchmarkResult};
pub use dfa::{Dfa, DfaState};
pub use engine::RegexEngine;
pub use exporter::Language;
pub use nfa::{Nfa, NfaState, Transition};
pub use parser::RegexParser;
pub use test_runner::{TestCase, TestResult, TestSuite};
pub use visualize::Visualizer;
