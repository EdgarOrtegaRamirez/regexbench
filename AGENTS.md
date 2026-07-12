# AGENTS.md

## Project Overview
RegexBench is a regex testing, debugging, and benchmarking CLI tool built in Rust.

## Architecture
- `src/ast.rs` — Regex AST types (Literal, Dot, CharacterClass, Groups, Quantifiers, Anchors)
- `src/parser.rs` — Recursive descent parser converting regex strings to AST
- `src/nfa.rs` — Thompson's NFA construction from AST
- `src/dfa.rs` — Subset construction (NFA→DFA) and Hopcroft's minimization
- `src/engine.rs` — Backtracking-based matching engine
- `src/test_runner.rs` — Test case execution and reporting
- `src/benchmark.rs` — Performance measurement with timing
- `src/analyzer.rs` — Pattern analysis and issue detection
- `src/exporter.rs` — Code generation for Python, JS, Go, Rust, Java, C#
- `src/visualize.rs` — ASCII art for NFA/DFA and match visualization

## Key Algorithms
1. **Thompson's Construction** — NFA from regex AST
2. **Subset Construction** — NFA to DFA conversion
3. **Hopcroft's Minimization** — DFA state minimization
4. **Backtracking Engine** — Pattern matching with group capture

## Testing
- Unit tests in each module (`#[cfg(test)]`)
- Integration tests in `tests/integration.rs`
- Run all: `cargo test`

## Building
```bash
cargo build --release
cargo test
```

## CLI Commands
- `test` — Run pattern against inputs
- `analyze` — Detect issues and backtracking risk
- `benchmark` — Measure performance
- `compare` — Compare multiple patterns
- `export` — Generate code in other languages
- `visualize` — Show NFA/DFA automata
- `repl` — Interactive mode

## Dependencies
- `clap` — CLI argument parsing
- `colored` — Terminal color output
- `anyhow` — Error handling
- `serde` / `serde_json` — JSON serialization
