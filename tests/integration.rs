use regexbench::{analyzer, benchmark, exporter, test_runner, visualize, Dfa, Nfa, RegexEngine};

#[test]
fn test_end_to_end_match() {
    let engine = RegexEngine::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    assert!(engine.is_match("user@example.com"));
    assert!(!engine.is_match("invalid"));
}

#[test]
fn test_nfa_dfa_equivalence() {
    let pattern = r"ab(c|d)*e";
    let engine = RegexEngine::new(pattern).unwrap();
    let nfa = Nfa::from_pattern(pattern).unwrap();
    let dfa = Dfa::from_nfa(&nfa);

    let test_cases = vec![
        ("abe", true),
        ("abcde", true),
        ("abccdde", true),
        ("ab", false),
        ("e", false),
    ];

    for (input, expected) in test_cases {
        assert_eq!(
            engine.is_match(input),
            expected,
            "Engine failed on: {}",
            input
        );
        assert_eq!(nfa.matches(input), expected, "NFA failed on: {}", input);
        assert_eq!(dfa.matches(input), expected, "DFA failed on: {}", input);
    }
}

#[test]
fn test_analyzer_detects_backtracking() {
    let analysis = analyzer::PatternAnalyzer::analyze(r"(a+)+b").unwrap();
    assert_eq!(analysis.backtrack_risk, analyzer::BacktrackRisk::High);
}

#[test]
fn test_benchmark_runs() {
    let config = benchmark::BenchmarkConfig {
        iterations: 100,
        inputs: vec!["test123".to_string()],
        warmup: false,
    };
    let result = benchmark::BenchmarkRunner::run(r"\d+", &config).unwrap();
    assert!(result.all_matched);
    assert!(result.avg_time_ns > 0.0);
}

#[test]
fn test_export_python() {
    let code = exporter::export(r"\d+", exporter::Language::Python).unwrap();
    assert!(code.contains("import re"));
}

#[test]
fn test_visualization() {
    let nfa = Nfa::from_pattern("abc").unwrap();
    let viz = visualize::Visualizer::nfa_to_ascii(&nfa);
    assert!(viz.contains("State"));
}

#[test]
fn test_test_runner() {
    let suite = test_runner::TestSuite::new(r"\d+")
        .add_match("123")
        .add_no_match("abc");
    let summary = suite.run_summary();
    assert!(summary.all_passed());
}
