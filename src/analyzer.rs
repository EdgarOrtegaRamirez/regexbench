/// Pattern analyzer for detecting potential issues
///
/// Analyzes regex patterns for complexity, backtracking risk, and optimization opportunities.
use serde::{Deserialize, Serialize};

/// Risk level for catastrophic backtracking
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BacktrackRisk {
    /// Safe - no nested quantifiers
    Safe,
    /// Low - some repetition but manageable
    Low,
    /// Medium - nested quantifiers present
    Medium,
    /// High - likely to cause catastrophic backtracking
    High,
}

/// Analysis result for a regex pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternAnalysis {
    /// The original pattern
    pub pattern: String,
    /// AST complexity score
    pub complexity_score: usize,
    /// Number of states in NFA
    pub nfa_states: usize,
    /// Number of states in DFA
    pub dfa_states: usize,
    /// Number of capturing groups
    pub capturing_groups: usize,
    /// Backtracking risk assessment
    pub backtrack_risk: BacktrackRisk,
    /// Issues found
    pub issues: Vec<PatternIssue>,
    /// Optimization suggestions
    pub suggestions: Vec<String>,
}

/// A specific issue found in the pattern
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternIssue {
    /// Issue severity
    pub severity: IssueSeverity,
    /// Description of the issue
    pub message: String,
    /// Position in the pattern (if applicable)
    pub position: Option<usize>,
}

/// Issue severity levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssueSeverity {
    /// Informational
    Info,
    /// Warning - potential issue
    Warning,
    /// Error - definite issue
    Error,
}

/// Analyzer for regex patterns
pub struct PatternAnalyzer;

impl PatternAnalyzer {
    /// Analyze a regex pattern
    pub fn analyze(pattern: &str) -> crate::Result<PatternAnalysis> {
        let ast = crate::parser::RegexParser::parse(pattern)?;
        let nfa = crate::nfa::Nfa::from_ast(&ast);
        let dfa = crate::dfa::Dfa::from_nfa(&nfa);
        let min_dfa = dfa.minimize();

        let mut issues = Vec::new();
        let mut suggestions = Vec::new();

        // Check for backtracking risk
        let backtrack_risk = Self::assess_backtrack_risk(&ast);

        // Check for common issues
        Self::check_literal_dot_literal(&ast, &mut issues, &mut suggestions);
        Self::check_unnecessary_groups(&ast, &mut issues, &mut suggestions);
        Self::check_char_class_single_char(&ast, &mut issues, &mut suggestions);
        Self::check_redundant_anchors(&ast, &mut issues);

        // Check complexity
        let complexity = ast.complexity_score();
        if complexity > 20 {
            suggestions.push("Pattern is complex. Consider simplifying if possible.".to_string());
        }

        Ok(PatternAnalysis {
            pattern: pattern.to_string(),
            complexity_score: complexity,
            nfa_states: nfa.state_count(),
            dfa_states: min_dfa.state_count(),
            capturing_groups: ast.capturing_group_count(),
            backtrack_risk,
            issues,
            suggestions,
        })
    }

    /// Assess backtracking risk by looking for nested quantifiers
    fn assess_backtrack_risk(ast: &crate::ast::RegexAst) -> BacktrackRisk {
        let risk = Self::check_nested_quantifiers(&ast.root);
        match risk {
            0 => BacktrackRisk::Safe,
            1 => BacktrackRisk::Low,
            2 => BacktrackRisk::Medium,
            _ => BacktrackRisk::High,
        }
    }

    fn check_nested_quantifiers(node: &crate::ast::AstNode) -> usize {
        match node {
            crate::ast::AstNode::Star(expr)
            | crate::ast::AstNode::Plus(expr)
            | crate::ast::AstNode::Optional(expr) => {
                // Check if the inner expression contains quantifiers
                if Self::has_quantifier(&expr.root) {
                    2 + Self::check_nested_quantifiers(&expr.root)
                } else {
                    Self::check_nested_quantifiers(&expr.root)
                }
            }
            crate::ast::AstNode::Repetition { expr, .. } => {
                if Self::has_quantifier(&expr.root) {
                    2 + Self::check_nested_quantifiers(&expr.root)
                } else {
                    Self::check_nested_quantifiers(&expr.root)
                }
            }
            crate::ast::AstNode::Concatenation(exprs) | crate::ast::AstNode::Alternation(exprs) => {
                exprs
                    .iter()
                    .map(|e| Self::check_nested_quantifiers(&e.root))
                    .max()
                    .unwrap_or(0)
            }
            crate::ast::AstNode::Group { expr, .. } => {
                if Self::has_quantifier(&expr.root) {
                    1 + Self::check_nested_quantifiers(&expr.root)
                } else {
                    Self::check_nested_quantifiers(&expr.root)
                }
            }
            _ => 0,
        }
    }

    fn has_quantifier(node: &crate::ast::AstNode) -> bool {
        match node {
            crate::ast::AstNode::Star(_)
            | crate::ast::AstNode::Plus(_)
            | crate::ast::AstNode::Optional(_)
            | crate::ast::AstNode::Repetition { .. } => true,
            crate::ast::AstNode::Concatenation(exprs) | crate::ast::AstNode::Alternation(exprs) => {
                exprs.iter().any(|e| Self::has_quantifier(&e.root))
            }
            crate::ast::AstNode::Group { expr, .. } => Self::has_quantifier(&expr.root),
            _ => false,
        }
    }

    /// Check for literal followed by dot followed by literal (common mistake)
    fn check_literal_dot_literal(
        ast: &crate::ast::RegexAst,
        issues: &mut Vec<PatternIssue>,
        suggestions: &mut Vec<String>,
    ) {
        Self::check_literal_dot_literal_node(&ast.root, issues, suggestions);
    }

    fn check_literal_dot_literal_node(
        node: &crate::ast::AstNode,
        issues: &mut Vec<PatternIssue>,
        suggestions: &mut Vec<String>,
    ) {
        if let crate::ast::AstNode::Concatenation(exprs) = node {
            for window in exprs.windows(3) {
                if let (
                    crate::ast::AstNode::Literal(_),
                    crate::ast::AstNode::Dot,
                    crate::ast::AstNode::Literal(_),
                ) = (&window[0].root, &window[1].root, &window[2].root)
                {
                    issues.push(PatternIssue {
                        severity: IssueSeverity::Info,
                        message:
                            "Literal-dot-literal pattern detected. Did you mean to escape the dot?"
                                .to_string(),
                        position: None,
                    });
                    suggestions
                        .push("If you want to match a literal dot, escape it: \\.".to_string());
                }
            }
        }
    }

    /// Check for unnecessary non-capturing groups
    fn check_unnecessary_groups(
        ast: &crate::ast::RegexAst,
        issues: &mut Vec<PatternIssue>,
        suggestions: &mut Vec<String>,
    ) {
        Self::check_unnecessary_groups_node(&ast.root, issues, suggestions);
    }

    fn check_unnecessary_groups_node(
        node: &crate::ast::AstNode,
        issues: &mut Vec<PatternIssue>,
        suggestions: &mut Vec<String>,
    ) {
        match node {
            crate::ast::AstNode::Group {
                capturing, expr, ..
            } => {
                if !capturing {
                    // Check if the group contains only a single literal
                    if let crate::ast::AstNode::Concatenation(inner) = &expr.root {
                        if inner.len() == 1 {
                            if let crate::ast::AstNode::Literal(_) = &inner[0].root {
                                issues.push(PatternIssue {
                                    severity: IssueSeverity::Info,
                                    message:
                                        "Unnecessary non-capturing group around single literal"
                                            .to_string(),
                                    position: None,
                                });
                                suggestions.push("Remove the group: (?:a) → a".to_string());
                            }
                        }
                    }
                }
                Self::check_unnecessary_groups_node(&expr.root, issues, suggestions);
            }
            crate::ast::AstNode::Concatenation(exprs) | crate::ast::AstNode::Alternation(exprs) => {
                for expr in exprs {
                    Self::check_unnecessary_groups_node(&expr.root, issues, suggestions);
                }
            }
            crate::ast::AstNode::Star(e)
            | crate::ast::AstNode::Plus(e)
            | crate::ast::AstNode::Optional(e) => {
                Self::check_unnecessary_groups_node(&e.root, issues, suggestions);
            }
            crate::ast::AstNode::Repetition { expr, .. } => {
                Self::check_unnecessary_groups_node(&expr.root, issues, suggestions);
            }
            _ => {}
        }
    }

    /// Check for character classes with single character
    fn check_char_class_single_char(
        ast: &crate::ast::RegexAst,
        issues: &mut Vec<PatternIssue>,
        suggestions: &mut Vec<String>,
    ) {
        Self::check_char_class_node(&ast.root, issues, suggestions);
    }

    fn check_char_class_node(
        node: &crate::ast::AstNode,
        issues: &mut Vec<PatternIssue>,
        suggestions: &mut Vec<String>,
    ) {
        match node {
            crate::ast::AstNode::CharacterClass { negated, ranges } => {
                if !negated && ranges.len() == 1 && ranges[0].start == ranges[0].end {
                    issues.push(PatternIssue {
                        severity: IssueSeverity::Info,
                        message: "Character class contains single character".to_string(),
                        position: None,
                    });
                    suggestions.push("Use the literal character instead: [a] → a".to_string());
                }
            }
            crate::ast::AstNode::Concatenation(exprs) | crate::ast::AstNode::Alternation(exprs) => {
                for expr in exprs {
                    Self::check_char_class_node(&expr.root, issues, suggestions);
                }
            }
            crate::ast::AstNode::Star(e)
            | crate::ast::AstNode::Plus(e)
            | crate::ast::AstNode::Optional(e) => {
                Self::check_char_class_node(&e.root, issues, suggestions);
            }
            crate::ast::AstNode::Repetition { expr, .. } => {
                Self::check_char_class_node(&expr.root, issues, suggestions);
            }
            crate::ast::AstNode::Group { expr, .. } => {
                Self::check_char_class_node(&expr.root, issues, suggestions);
            }
            _ => {}
        }
    }

    /// Check for redundant anchors
    fn check_redundant_anchors(ast: &crate::ast::RegexAst, issues: &mut Vec<PatternIssue>) {
        if let crate::ast::AstNode::Concatenation(exprs) = &ast.root {
            // Check for ^ at start followed by $ at end with nothing else
            if exprs.len() == 2 {
                if let (crate::ast::AstNode::StartAnchor, crate::ast::AstNode::EndAnchor) =
                    (&exprs[0].root, &exprs[1].root)
                {
                    issues.push(PatternIssue {
                        severity: IssueSeverity::Warning,
                        message: "Pattern matches empty string only".to_string(),
                        position: None,
                    });
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_safe_pattern() {
        let analysis = PatternAnalyzer::analyze(r"abc").unwrap();
        assert_eq!(analysis.backtrack_risk, BacktrackRisk::Safe);
        assert!(analysis.issues.is_empty());
    }

    #[test]
    fn test_nested_quantifiers() {
        let analysis = PatternAnalyzer::analyze(r"(a+)+").unwrap();
        assert_eq!(analysis.backtrack_risk, BacktrackRisk::High);
    }

    #[test]
    fn test_single_char_class() {
        let analysis = PatternAnalyzer::analyze(r"[a]").unwrap();
        assert!(!analysis.issues.is_empty());
        assert!(analysis.suggestions.iter().any(|s| s.contains("literal")));
    }

    #[test]
    fn test_complexity_score() {
        let simple = PatternAnalyzer::analyze(r"abc").unwrap();
        let complex = PatternAnalyzer::analyze(r"(?:[a-z]+@[a-z]+\.[a-z]{2,})").unwrap();
        assert!(complex.complexity_score > simple.complexity_score);
    }
}
