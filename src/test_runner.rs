/// Test runner for regex patterns
///
/// Runs test cases against regex patterns and reports results.
use serde::{Deserialize, Serialize};

/// A single test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Input string to test
    pub input: String,
    /// Whether the pattern should match
    pub should_match: bool,
    /// Optional expected match text
    pub expected_match: Option<String>,
    /// Optional description
    pub description: Option<String>,
}

/// Result of running a test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestResult {
    /// The test case
    pub test_case: TestCase,
    /// Whether the test passed
    pub passed: bool,
    /// Actual match result
    pub actual_match: bool,
    /// Actual matched text (if any)
    pub actual_text: Option<String>,
    /// Error message (if any)
    pub error: Option<String>,
    /// Execution time in microseconds
    pub duration_us: u64,
}

/// A test suite containing multiple test cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSuite {
    /// Pattern to test
    pub pattern: String,
    /// Test cases
    pub cases: Vec<TestCase>,
}

impl TestSuite {
    /// Create a new test suite
    pub fn new(pattern: &str) -> Self {
        Self {
            pattern: pattern.to_string(),
            cases: Vec::new(),
        }
    }

    /// Add a test case that should match
    pub fn add_match(mut self, input: &str) -> Self {
        self.cases.push(TestCase {
            input: input.to_string(),
            should_match: true,
            expected_match: None,
            description: None,
        });
        self
    }

    /// Add a test case that should not match
    pub fn add_no_match(mut self, input: &str) -> Self {
        self.cases.push(TestCase {
            input: input.to_string(),
            should_match: false,
            expected_match: None,
            description: None,
        });
        self
    }

    /// Add a test case with description
    pub fn add_case(mut self, input: &str, should_match: bool, description: &str) -> Self {
        self.cases.push(TestCase {
            input: input.to_string(),
            should_match,
            expected_match: None,
            description: Some(description.to_string()),
        });
        self
    }

    /// Run all test cases
    pub fn run(&self) -> Vec<TestResult> {
        let engine = match crate::engine::RegexEngine::new(&self.pattern) {
            Ok(e) => e,
            Err(e) => {
                return self
                    .cases
                    .iter()
                    .map(|tc| TestResult {
                        test_case: tc.clone(),
                        passed: false,
                        actual_match: false,
                        actual_text: None,
                        error: Some(e.to_string()),
                        duration_us: 0,
                    })
                    .collect();
            }
        };

        self.cases
            .iter()
            .map(|tc| {
                let start = std::time::Instant::now();
                let result = engine.find_match(&tc.input, 0);
                let duration = start.elapsed().as_micros() as u64;

                let actual_match = result.is_some();
                let actual_text = result.and_then(|r| r.text);

                let passed = if tc.should_match {
                    actual_match
                } else {
                    !actual_match
                };

                TestResult {
                    test_case: tc.clone(),
                    passed,
                    actual_match,
                    actual_text,
                    error: None,
                    duration_us: duration,
                }
            })
            .collect()
    }

    /// Run tests and return summary
    pub fn run_summary(&self) -> TestSummary {
        let results = self.run();
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let total_duration: u64 = results.iter().map(|r| r.duration_us).sum();

        TestSummary {
            pattern: self.pattern.clone(),
            total,
            passed,
            failed,
            total_duration_us: total_duration,
            results,
        }
    }
}

/// Summary of test execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSummary {
    pub pattern: String,
    pub total: usize,
    pub passed: usize,
    pub failed: usize,
    pub total_duration_us: u64,
    pub results: Vec<TestResult>,
}

impl TestSummary {
    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Get failed test cases
    pub fn failures(&self) -> Vec<&TestResult> {
        self.results.iter().filter(|r| !r.passed).collect()
    }

    /// Format as human-readable string
    pub fn format(&self) -> String {
        let mut output = String::new();

        output.push_str(&format!("Pattern: {}\n", self.pattern));
        output.push_str(&format!(
            "Results: {}/{} passed ({:.1?})\n",
            self.passed,
            self.total,
            std::time::Duration::from_micros(self.total_duration_us)
        ));
        output.push('\n');

        for result in &self.results {
            let status = if result.passed {
                "✓".green()
            } else {
                "✗".red()
            };

            let desc = result
                .test_case
                .description
                .as_deref()
                .unwrap_or(&result.test_case.input);

            output.push_str(&format!("  {} {}", status, desc));

            if !result.passed {
                if let Some(err) = &result.error {
                    output.push_str(&format!(" (error: {})", err));
                } else if result.test_case.should_match {
                    output.push_str(" (expected match)");
                } else {
                    output.push_str(" (unexpected match)");
                }
            }

            output.push('\n');
        }

        output
    }
}

use colored::Colorize;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_suite() {
        let suite = TestSuite::new(r"\d+")
            .add_match("123")
            .add_match("abc 456 def")
            .add_no_match("abc");

        let summary = suite.run_summary();
        assert_eq!(summary.total, 3);
        assert!(summary.all_passed());
    }

    #[test]
    fn test_email_suite() {
        let suite = TestSuite::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .add_match("user@example.com")
            .add_match("test.name+tag@domain.co.uk")
            .add_no_match("invalid")
            .add_no_match("@no-user.com")
            .add_no_match("user@.com");

        let summary = suite.run_summary();
        assert!(summary.all_passed());
    }

    #[test]
    fn test_suite_with_descriptions() {
        let suite = TestSuite::new(r"^[A-Z]")
            .add_case("Hello", true, "Starts with capital")
            .add_case("hello", false, "Starts with lowercase");

        let summary = suite.run_summary();
        assert!(summary.all_passed());
    }
}

#[cfg(test)]
mod debug_email {
    use super::*;

    #[test]
    fn debug_email_cases() {
        let suite = TestSuite::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$")
            .add_match("user@example.com")
            .add_match("test.name+tag@domain.co.uk")
            .add_no_match("invalid")
            .add_no_match("@no-user.com")
            .add_no_match("user@.com");

        let results = suite.run();
        for r in &results {
            eprintln!(
                "Input: {:30} should_match: {:5} passed: {:5} error: {:?}",
                r.test_case.input, r.test_case.should_match, r.passed, r.error
            );
        }
    }
}
