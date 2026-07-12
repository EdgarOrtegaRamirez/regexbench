# RegexBench

A regex testing, debugging, and benchmarking CLI tool.

## Features

- **Test**: Run regex patterns against multiple inputs with detailed results
- **Analyze**: Detect backtracking risks, complexity issues, and optimization opportunities
- **Benchmark**: Measure regex performance with precise timing
- **Compare**: Compare performance of multiple patterns side-by-side
- **Export**: Generate equivalent code for Python, JavaScript, Go, Rust, Java, C#
- **Visualize**: See NFA/DFA automata and match timelines
- **REPL**: Interactive mode for exploring patterns

## Installation

```bash
cargo install regexbench
```

## Quick Start

```bash
# Test a pattern
regexbench test "\d+" -i "hello 123 world"

# Analyze for issues
regexbench analyze "(a+)+b"

# Benchmark performance
regexbench benchmark "\d+" -i "test123" -n 10000

# Compare patterns
regexbench compare "a*b*c*" "a+b+c+" -i "aaaabbbccc"

# Export to Python
regexbench export "\d+" -l python

# Visualize automata
regexbench visualize "ab(c|d)*" --dfa --minimize
```

## Commands

### test
Run a regex pattern against inputs.

```bash
regexbench test "<pattern>" -i "<input1>" -i "<input2>"
regexbench test "<pattern>" -f tests.txt  # one test per line, !prefix = no match
```

### analyze
Analyze a pattern for issues and optimization opportunities.

```bash
regexbench analyze "<pattern>"
```

### benchmark
Measure pattern performance.

```bash
regexbench benchmark "<pattern>" -i "<input>" -n 10000
```

### compare
Compare multiple patterns.

```bash
regexbench compare "<pattern1>" "<pattern2>" "<pattern3>" -i "<input>"
```

### export
Generate code in other languages.

```bash
regexbench export "<pattern>" -l python|javascript|go|rust|java|csharp
```

### visualize
Show NFA/DFA automata.

```bash
regexbench visualize "<pattern>" --dfa --minimize
regexbench visualize "<pattern>" -i "<input>"  # show match timeline
```

### repl
Interactive REPL for testing patterns.

```bash
regexbench repl
```

## Architecture

- **AST**: Regex abstract syntax tree types
- **Parser**: Recursive descent parser for regex patterns
- **NFA**: Thompson's construction for NFA building
- **DFA**: Subset construction and Hopcroft's minimization
- **Engine**: Backtracking-based matching engine
- **Test Runner**: Test case execution and reporting
- **Benchmark**: Performance measurement
- **Analyzer**: Pattern analysis and issue detection
- **Exporter**: Code generation for multiple languages
- **Visualize**: ASCII art for automata

## License

MIT
