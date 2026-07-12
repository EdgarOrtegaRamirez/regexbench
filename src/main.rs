/// RegexBench - Regex Testing, Debugging & Benchmarking CLI
///
/// A comprehensive tool for testing, analyzing, benchmarking, and exporting
/// regular expressions.
use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::Colorize;

#[derive(Parser)]
#[command(
    name = "regexbench",
    about = "A regex testing, debugging, and benchmarking CLI tool",
    version,
    long_about = "RegexBench provides comprehensive tools for working with regular expressions.\n\
                  Test patterns, analyze for issues, benchmark performance, and export to other languages."
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Test a regex pattern against inputs
    Test {
        /// The regex pattern
        pattern: String,

        /// Input strings to test
        #[arg(short, long, num_args = 1..)]
        input: Vec<String>,

        /// Input file (one test per line, prefix with ! to expect no match)
        #[arg(short, long)]
        file: Option<String>,

        /// Show match positions
        #[arg(long)]
        positions: bool,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Analyze a regex pattern for issues
    Analyze {
        /// The regex pattern
        pattern: String,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Benchmark regex performance
    Benchmark {
        /// The regex pattern
        pattern: String,

        /// Input strings to benchmark against
        #[arg(short, long, num_args = 1..)]
        input: Vec<String>,

        /// Number of iterations
        #[arg(short, long, default_value = "1000")]
        iterations: usize,

        /// Skip warmup
        #[arg(long)]
        no_warmup: bool,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Compare performance of multiple patterns
    Compare {
        /// Patterns to compare
        #[arg(num_args = 2..)]
        patterns: Vec<String>,

        /// Input string to test against
        #[arg(short, long)]
        input: String,

        /// Number of iterations
        #[arg(short, long, default_value = "1000")]
        iterations: usize,

        /// Output format (text, json)
        #[arg(long, default_value = "text")]
        format: String,
    },

    /// Export regex pattern to other languages
    Export {
        /// The regex pattern
        pattern: String,

        /// Target language
        #[arg(short, long)]
        language: String,
    },

    /// Visualize NFA/DFA for a pattern
    Visualize {
        /// The regex pattern
        pattern: String,

        /// Show DFA instead of NFA
        #[arg(long)]
        dfa: bool,

        /// Minimize the DFA
        #[arg(long)]
        minimize: bool,

        /// Show match timeline for input
        #[arg(short, long)]
        input: Option<String>,
    },

    /// Interactive REPL for testing patterns
    Repl,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Test {
            pattern,
            input,
            file,
            positions,
            format,
        } => cmd_test(&pattern, &input, file.as_deref(), positions, &format),

        Commands::Analyze { pattern, format } => cmd_analyze(&pattern, &format),

        Commands::Benchmark {
            pattern,
            input,
            iterations,
            no_warmup,
            format,
        } => cmd_benchmark(&pattern, &input, iterations, no_warmup, &format),

        Commands::Compare {
            patterns,
            input,
            iterations,
            format,
        } => cmd_compare(&patterns, &input, iterations, &format),

        Commands::Export { pattern, language } => cmd_export(&pattern, &language),

        Commands::Visualize {
            pattern,
            dfa,
            minimize,
            input,
        } => cmd_visualize(&pattern, dfa, minimize, input.as_deref()),

        Commands::Repl => cmd_repl(),
    }
}

fn cmd_test(
    pattern: &str,
    inputs: &[String],
    file: Option<&str>,
    positions: bool,
    format: &str,
) -> Result<()> {
    let mut test_cases: Vec<(String, bool)> = inputs.iter().map(|i| (i.clone(), true)).collect();

    if let Some(path) = file {
        let content = std::fs::read_to_string(path)?;
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(input) = line.strip_prefix('!') {
                test_cases.push((input.to_string(), false));
            } else {
                test_cases.push((line.to_string(), true));
            }
        }
    }

    let engine = regexbench::RegexEngine::new(pattern)?;

    if format == "json" {
        let results: Vec<_> = test_cases
            .iter()
            .map(|(input, should_match)| {
                let result = engine.find_match(input, 0);
                let matched = result.is_some();
                serde_json::json!({
                    "input": input,
                    "should_match": should_match,
                    "matched": matched,
                    "passed": matched == *should_match,
                    "start": result.as_ref().and_then(|r| r.start),
                    "end": result.as_ref().and_then(|r| r.end),
                    "text": result.and_then(|r| r.text),
                })
            })
            .collect();

        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!("Pattern: {}\n", pattern.bright_cyan());
        println!(
            "{:<40} {:<10} {:<10} {:<10}",
            "Input", "Expected", "Got", "Result"
        );
        println!("{}", "-".repeat(70));

        for (input, should_match) in &test_cases {
            let result = engine.find_match(input, 0);
            let matched = result.is_some();
            let passed = matched == *should_match;

            let status = if passed {
                "✓ PASS".green()
            } else {
                "✗ FAIL".red()
            };

            let expected = if *should_match {
                "match".green()
            } else {
                "no match".yellow()
            };

            let got = if matched {
                "match".green()
            } else {
                "no match".yellow()
            };

            let display_input = if input.len() > 38 {
                format!("{}...", &input[..35])
            } else {
                input.clone()
            };

            println!(
                "{:<40} {:<10} {:<10} {}",
                display_input, expected, got, status
            );

            if positions {
                if let Some(m) = result {
                    println!("  {} Match at {:?}: {:?}", "→".dimmed(), m.start, m.text);
                }
            }
        }
    }

    Ok(())
}

fn cmd_analyze(pattern: &str, format: &str) -> Result<()> {
    let analysis = regexbench::analyzer::PatternAnalyzer::analyze(pattern)?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&analysis)?);
    } else {
        println!("Pattern Analysis: {}\n", pattern.bright_cyan());
        println!("Complexity Score: {}", analysis.complexity_score);
        println!("NFA States:       {}", analysis.nfa_states);
        println!("DFA States:       {}", analysis.dfa_states);
        println!("Capturing Groups: {}", analysis.capturing_groups);

        let risk_color = match analysis.backtrack_risk {
            regexbench::analyzer::BacktrackRisk::Safe => "Safe".green(),
            regexbench::analyzer::BacktrackRisk::Low => "Low".yellow(),
            regexbench::analyzer::BacktrackRisk::Medium => "Medium".yellow(),
            regexbench::analyzer::BacktrackRisk::High => "High".red(),
        };
        println!("Backtrack Risk:   {}", risk_color);

        if !analysis.issues.is_empty() {
            println!("\nIssues:");
            for issue in &analysis.issues {
                let severity = match issue.severity {
                    regexbench::analyzer::IssueSeverity::Info => "INFO".blue(),
                    regexbench::analyzer::IssueSeverity::Warning => "WARN".yellow(),
                    regexbench::analyzer::IssueSeverity::Error => "ERROR".red(),
                };
                println!("  [{}] {}", severity, issue.message);
            }
        }

        if !analysis.suggestions.is_empty() {
            println!("\nSuggestions:");
            for suggestion in &analysis.suggestions {
                println!("  💡 {}", suggestion);
            }
        }
    }

    Ok(())
}

fn cmd_benchmark(
    pattern: &str,
    inputs: &[String],
    iterations: usize,
    no_warmup: bool,
    format: &str,
) -> Result<()> {
    let config = regexbench::benchmark::BenchmarkConfig {
        iterations,
        inputs: inputs.to_vec(),
        warmup: !no_warmup,
    };

    let result = regexbench::benchmark::BenchmarkRunner::run(pattern, &config)?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&result)?);
    } else {
        println!("Benchmark: {}\n", pattern.bright_cyan());
        println!("Iterations:   {}", result.iterations);
        println!(
            "Total Time:   {:.2?}",
            std::time::Duration::from_micros(result.total_time_us)
        );
        println!(
            "Avg/Iter:     {:.2?}",
            std::time::Duration::from_nanos(result.avg_time_ns as u64)
        );
        println!(
            "Min/Iter:     {:.2?}",
            std::time::Duration::from_nanos(result.min_time_ns as u64)
        );
        println!(
            "Max/Iter:     {:.2?}",
            std::time::Duration::from_nanos(result.max_time_ns as u64)
        );
        println!("Ops/sec:      {:.0}", result.ops_per_sec);
        println!(
            "All Matched:  {}",
            if result.all_matched {
                "yes".green()
            } else {
                "no".red()
            }
        );
    }

    Ok(())
}

fn cmd_compare(patterns: &[String], input: &str, iterations: usize, format: &str) -> Result<()> {
    let config = regexbench::benchmark::BenchmarkConfig {
        iterations,
        inputs: vec![input.to_string()],
        warmup: true,
    };

    let pattern_refs: Vec<&str> = patterns.iter().map(|s| s.as_str()).collect();
    let results = regexbench::benchmark::BenchmarkRunner::compare(&pattern_refs, &config)?;

    if format == "json" {
        println!("{}", serde_json::to_string_pretty(&results)?);
    } else {
        println!("Pattern Comparison\n");
        println!(
            "{:<30} {:>12} {:>12} {:>12}",
            "Pattern", "Avg (ns)", "Ops/sec", "Matched"
        );
        println!("{}", "-".repeat(66));

        // Sort by avg time
        let mut sorted = results.clone();
        sorted.sort_by(|a, b| a.avg_time_ns.partial_cmp(&b.avg_time_ns).unwrap());

        for (i, result) in sorted.iter().enumerate() {
            let rank = format!("#{}", i + 1);
            let pattern_display = if result.pattern.len() > 28 {
                format!("{}...", &result.pattern[..25])
            } else {
                result.pattern.clone()
            };

            println!(
                "{} {:<27} {:>12.0} {:>12.0} {:>12}",
                rank.bright_cyan(),
                pattern_display,
                result.avg_time_ns,
                result.ops_per_sec,
                if result.all_matched {
                    "yes".green()
                } else {
                    "no".red()
                },
            );
        }
    }

    Ok(())
}

fn cmd_export(pattern: &str, language: &str) -> Result<()> {
    let lang = match language.to_lowercase().as_str() {
        "python" | "py" => regexbench::exporter::Language::Python,
        "javascript" | "js" => regexbench::exporter::Language::JavaScript,
        "go" | "golang" => regexbench::exporter::Language::Go,
        "rust" | "rs" => regexbench::exporter::Language::Rust,
        "java" => regexbench::exporter::Language::Java,
        "csharp" | "c#" | "cs" => regexbench::exporter::Language::CSharp,
        _ => {
            anyhow::bail!(
                "Unsupported language: {}. Supported: python, javascript, go, rust, java, csharp",
                language
            );
        }
    };

    let code = regexbench::exporter::export(pattern, lang)?;
    println!("{}", code);
    Ok(())
}

fn cmd_visualize(pattern: &str, show_dfa: bool, minimize: bool, input: Option<&str>) -> Result<()> {
    let ast = regexbench::parser::RegexParser::parse(pattern)?;

    if show_dfa {
        let nfa = regexbench::nfa::Nfa::from_ast(&ast);
        let mut dfa = regexbench::dfa::Dfa::from_nfa(&nfa);

        if minimize {
            dfa = dfa.minimize();
        }

        print!("{}", regexbench::visualize::Visualizer::dfa_to_ascii(&dfa));
    } else {
        let nfa = regexbench::nfa::Nfa::from_ast(&ast);
        print!("{}", regexbench::visualize::Visualizer::nfa_to_ascii(&nfa));
    }

    if let Some(input) = input {
        println!("\n=== Match Timeline ===\n");
        print!(
            "{}",
            regexbench::visualize::Visualizer::match_timeline(input, pattern)
        );
    }

    Ok(())
}

fn cmd_repl() -> Result<()> {
    println!("RegexBench REPL (type 'help' for commands, 'quit' to exit)\n");

    let mut current_pattern: Option<regexbench::RegexEngine> = None;

    loop {
        print!("regexbench> ");
        use std::io::Write;
        std::io::stdout().flush()?;

        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        match line {
            "quit" | "exit" | "q" => break,
            "help" | "h" => {
                println!("Commands:");
                println!("  /pattern <regex>  - Set the current pattern");
                println!("  /test <input>     - Test input against pattern");
                println!("  /analyze          - Analyze current pattern");
                println!("  /viz              - Visualize NFA/DFA");
                println!("  /help             - Show this help");
                println!("  quit              - Exit REPL");
                println!();
                println!("Or type any string to test it against the current pattern.");
            }
            p if p.starts_with("/pattern ") || p.starts_with("/p ") => {
                let pat = if let Some(stripped) = p.strip_prefix("/pattern ") {
                    stripped
                } else {
                    &p[3..]
                };
                match regexbench::RegexEngine::new(pat) {
                    Ok(engine) => {
                        current_pattern = Some(engine);
                        println!("Pattern set: {}", pat.bright_cyan());
                    }
                    Err(e) => println!("Error: {}", e),
                }
            }
            t if t.starts_with("/test ") || t.starts_with("/t ") => {
                let input = if let Some(stripped) = t.strip_prefix("/test ") {
                    stripped
                } else {
                    &t[3..]
                };
                if let Some(ref engine) = current_pattern {
                    let result = engine.find_match(input, 0);
                    if let Some(m) = result {
                        println!("Match: {:?} at {:?}-{:?}", m.text, m.start, m.end);
                    } else {
                        println!("No match");
                    }
                } else {
                    println!("No pattern set. Use /pattern <regex>");
                }
            }
            "/analyze" | "/a" => {
                if let Some(ref _engine) = current_pattern {
                    // We'd need to store the pattern string too
                    println!("Analysis not available in REPL (use analyze command)");
                } else {
                    println!("No pattern set. Use /pattern <regex>");
                }
            }
            "/viz" | "/v" => {
                if let Some(ref _engine) = current_pattern {
                    println!("Visualization not available in REPL (use visualize command)");
                } else {
                    println!("No pattern set. Use /pattern <regex>");
                }
            }
            input => {
                if let Some(ref engine) = current_pattern {
                    let result = engine.find_match(input, 0);
                    if let Some(m) = result {
                        println!("Match: {:?} at {:?}-{:?}", m.text, m.start, m.end);
                        if !m.groups.is_empty() {
                            for (i, g) in m.groups.iter().enumerate() {
                                if let Some(g) = g {
                                    println!("  Group {}: {:?}", i, g);
                                }
                            }
                        }
                    } else {
                        println!("No match");
                    }
                } else {
                    println!("No pattern set. Use /pattern <regex>");
                }
            }
        }
    }

    Ok(())
}
