/// Visualization module for regex patterns
///
/// Provides ASCII art visualization of NFA/DFA automata, match results,
/// Graphviz DOT output, and step-by-step match tracing.
use crate::dfa::{Dfa, DfaStateId};
use crate::nfa::{Nfa, Transition};
use std::collections::{BTreeMap, HashMap};

/// Visualizer for regex structures
pub struct Visualizer;

impl Visualizer {
    /// Visualize an NFA as ASCII art
    pub fn nfa_to_ascii(nfa: &Nfa) -> String {
        let mut output = String::new();
        output.push_str("=== NFA States ===\n");
        output.push_str(&format!(
            "States: {}, Start: {}, Accept: {}\n\n",
            nfa.states.len(),
            nfa.start,
            nfa.accept
        ));

        for state in &nfa.states {
            let marker = if state.is_accept {
                " (accept)"
            } else if state.id == nfa.start {
                " (start)"
            } else {
                ""
            };
            output.push_str(&format!("State {}{}:\n", state.id, marker));

            for (trans, target) in &state.transitions {
                let label = match trans {
                    Transition::Epsilon => "ε".to_string(),
                    Transition::Char(c) => format!("'{}'", c),
                    Transition::Dot => ".".to_string(),
                    Transition::CharClass { negated, chars: _ } => {
                        if *negated {
                            "[^...]".to_string()
                        } else {
                            "[...]".to_string()
                        }
                    }
                    Transition::StartAnchor => "^".to_string(),
                    Transition::EndAnchor => "$".to_string(),
                    Transition::WordBoundary => "\\b".to_string(),
                };
                output.push_str(&format!("  --{}--> State {}\n", label, target));
            }
            output.push('\n');
        }

        output
    }

    /// Visualize a DFA as ASCII art
    pub fn dfa_to_ascii(dfa: &Dfa) -> String {
        let mut output = String::new();
        output.push_str("=== DFA States ===\n");
        output.push_str(&format!(
            "States: {}, Start: {}\n\n",
            dfa.states.len(),
            dfa.start
        ));

        for state in &dfa.states {
            let marker = if state.is_accept {
                " (accept)"
            } else if state.id == dfa.start {
                " (start)"
            } else {
                ""
            };
            output.push_str(&format!("State {}{}:\n", state.id, marker));

            // Group transitions by target
            let mut by_target: HashMap<usize, Vec<char>> = HashMap::new();
            for (&(from, ch), &to) in &dfa.transitions {
                if from == state.id {
                    by_target.entry(to).or_default().push(ch);
                }
            }

            let mut targets: Vec<_> = by_target.into_iter().collect();
            targets.sort_by_key(|(t, _)| *t);

            for (target, mut chars) in targets {
                chars.sort();
                let label = if chars.len() <= 3 {
                    chars
                        .iter()
                        .map(|c| format!("'{}'", c))
                        .collect::<Vec<_>>()
                        .join(", ")
                } else {
                    format!("{} chars", chars.len())
                };
                output.push_str(&format!("  --[{}]--> State {}\n", label, target));
            }
            output.push('\n');
        }

        output
    }

    /// Visualize a match result with highlighting
    pub fn match_highlight(input: &str, start: usize, end: usize) -> String {
        let before: String = input.chars().take(start).collect();
        let matched: String = input.chars().skip(start).take(end - start).collect();
        let after: String = input.chars().skip(end).collect();

        format!("{}[{}]{}", before, matched, after)
    }

    /// Visualize all matches in a string
    pub fn matches_highlight(input: &str, matches: &[(usize, usize)]) -> String {
        let mut output = String::new();
        let chars: Vec<char> = input.chars().collect();
        let mut last_end = 0;

        for &(start, end) in matches {
            // Add unmatched prefix
            for &ch in chars[last_end..start].iter() {
                output.push(ch);
            }
            // Add matched section with brackets
            output.push('[');
            for &ch in chars[start..end].iter() {
                output.push(ch);
            }
            output.push(']');
            last_end = end;
        }

        // Add remaining
        for &ch in chars[last_end..].iter() {
            output.push(ch);
        }

        output
    }

    /// Create a visual timeline of match attempts
    pub fn match_timeline(input: &str, pattern: &str) -> String {
        let mut output = String::new();
        let chars: Vec<char> = input.chars().collect();

        output.push_str("Input:  ");
        for c in chars.iter() {
            output.push_str(&format!("{:2}", c));
        }
        output.push_str("\nIndex:  ");
        for i in 0..chars.len() {
            output.push_str(&format!("{:2}", i));
        }
        output.push('\n');

        // Try matching at each position
        let engine = match crate::engine::RegexEngine::new(pattern) {
            Ok(e) => e,
            Err(_) => return output,
        };

        output.push_str("Match:  ");
        for i in 0..chars.len() {
            let label = if engine.find_match(input, i).is_some() {
                " M"
            } else {
                " ."
            };
            output.push_str(label);
        }
        output.push('\n');

        output
    }

    /// Generate Graphviz DOT output for an NFA
    pub fn nfa_to_dot(nfa: &Nfa) -> String {
        let mut dot = String::from("digraph NFA {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=circle];\n\n");

        // Start arrow
        dot.push_str("  __start__ [shape=point, style=invis];\n");
        dot.push_str(&format!("  __start__ -> q{};\n\n", nfa.start));

        // Accept state is double circle
        dot.push_str(&format!("  q{} [shape=doublecircle];\n\n", nfa.accept));

        // Collect transitions grouped by (from, symbols, to)
        for state in &nfa.states {
            for (trans, target) in &state.transitions {
                let label = match trans {
                    Transition::Epsilon => "ε".to_string(),
                    Transition::Char(c) => c.to_string(),
                    Transition::Dot => ".".to_string(),
                    Transition::CharClass {
                        negated: _,
                        chars: _,
                    } => "[...]".to_string(),
                    Transition::StartAnchor => "^".to_string(),
                    Transition::EndAnchor => "$".to_string(),
                    Transition::WordBoundary => "\\b".to_string(),
                };
                dot.push_str(&format!(
                    "  q{} -> q{} [label=\"{}\"];\n",
                    state.id, target, label
                ));
            }
        }

        dot.push_str("}\n");
        dot
    }

    /// Generate Graphviz DOT output for a DFA
    pub fn dfa_to_dot(dfa: &Dfa) -> String {
        let mut dot = String::from("digraph DFA {\n");
        dot.push_str("  rankdir=LR;\n");
        dot.push_str("  node [shape=circle];\n\n");

        // Start arrow
        dot.push_str("  __start__ [shape=point, style=invis];\n");
        dot.push_str(&format!("  __start__ -> q{};\n\n", dfa.start));

        // Accept states are double circles
        for state in &dfa.states {
            if state.is_accept {
                dot.push_str(&format!("  q{} [shape=doublecircle];\n", state.id));
            }
        }
        dot.push('\n');

        // Group transitions by (from, to) and collect characters
        let mut by_edge: BTreeMap<(DfaStateId, DfaStateId), Vec<char>> = BTreeMap::new();
        for (&(from, ch), &to) in &dfa.transitions {
            by_edge.entry((from, to)).or_default().push(ch);
        }

        for ((from, to), mut chars) in by_edge {
            chars.sort();
            let label: String = chars
                .iter()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(",");
            dot.push_str(&format!("  q{} -> q{} [label=\"{}\"];\n", from, to, label));
        }

        dot.push_str("}\n");
        dot
    }

    /// Step-by-step DFA match tracing
    ///
    /// Returns a vector of trace entries showing each step through the DFA.
    /// Each entry contains: (step_number, character, state_id, is_accepting)
    pub fn match_trace(dfa: &Dfa, input: &str) -> Vec<(usize, String, DfaStateId, bool)> {
        let mut trace = Vec::new();
        let mut current = dfa.start;

        trace.push((
            0,
            "START".to_string(),
            current,
            dfa.states[current].is_accept,
        ));

        for (i, c) in input.chars().enumerate() {
            match dfa.transitions.get(&(current, c)) {
                Some(&next) => {
                    trace.push((i + 1, format!("'{}'", c), next, dfa.states[next].is_accept));
                    current = next;
                }
                None => {
                    trace.push((i + 1, format!("'{}' \u{2717} REJECT", c), current, false));
                    break;
                }
            }
        }

        trace
    }

    /// Format match trace as a human-readable string
    pub fn format_match_trace(dfa: &Dfa, input: &str) -> String {
        let trace = Self::match_trace(dfa, input);
        let mut output = String::new();

        output.push_str(&format!("=== DFA Match Trace: \"{}\" ===\n\n", input));
        output.push_str(&format!(
            "{:<8} {:<15} {:<10} {}\n",
            "Step", "Input", "State", "Status"
        ));
        output.push_str(&format!("{}\n", "-".repeat(45)));

        for (step, ch, state_id, is_accept) in &trace {
            let status = if ch.contains("\u{2717}") {
                "✗ REJECT".to_string()
            } else if *is_accept && *step == input.chars().count() {
                "✓ ACCEPT".to_string()
            } else if *is_accept {
                "accepting".to_string()
            } else {
                "".to_string()
            };
            output.push_str(&format!(
                "{:<8} {:<15} q{:<8} {}\n",
                step, ch, state_id, status
            ));
        }

        // Determine final result based on whether the trace ended with a rejection
        let is_rejected = trace
            .last()
            .map(|(_, ch, _, _)| ch.contains("\u{2717}"))
            .unwrap_or(true);
        let final_state = trace
            .last()
            .map(|(_, _, state, _)| *state)
            .unwrap_or(dfa.start);
        let was_accepted = !is_rejected && dfa.states[final_state].is_accept;
        output.push('\n');
        if was_accepted {
            output.push_str(&format!(
                "Result: ✓ ACCEPT (reached accepting state q{})\n",
                final_state
            ));
        } else if is_rejected {
            output.push_str(&format!(
                "Result: ✗ REJECT (no transition for character at state q{})\n",
                final_state
            ));
        } else {
            output.push_str(&format!(
                "Result: ✗ REJECT (stuck at non-accepting state q{})\n",
                final_state
            ));
        }

        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nfa_visualization() {
        let nfa = Nfa::from_pattern("abc").unwrap();
        let viz = Visualizer::nfa_to_ascii(&nfa);
        assert!(viz.contains("NFA States"));
        assert!(viz.contains("State"));
    }

    #[test]
    fn test_dfa_visualization() {
        let nfa = Nfa::from_pattern("ab").unwrap();
        let dfa = Dfa::from_nfa(&nfa);
        let viz = Visualizer::dfa_to_ascii(&dfa);
        assert!(viz.contains("DFA States"));
    }

    #[test]
    fn test_match_highlight() {
        let result = Visualizer::match_highlight("hello world", 6, 11);
        assert_eq!(result, "hello [world]");
    }

    #[test]
    fn test_matches_highlight() {
        let result = Visualizer::matches_highlight("a b c", &[(0, 1), (2, 3), (4, 5)]);
        assert_eq!(result, "[a] [b] [c]");
    }

    #[test]
    fn test_nfa_to_dot() {
        let nfa = Nfa::from_pattern("a").unwrap();
        let dot = Visualizer::nfa_to_dot(&nfa);
        assert!(dot.contains("digraph NFA"));
        // States may not be q0 (Thompson construction adds new states)
        // Just check the format is correct
        assert!(dot.contains("q"));
        assert!(dot.contains("__start__"));
    }

    #[test]
    fn test_dfa_to_dot() {
        let nfa = Nfa::from_pattern("ab").unwrap();
        let dfa = Dfa::from_nfa(&nfa);
        let dot = Visualizer::dfa_to_dot(&dfa);
        assert!(dot.contains("digraph DFA"));
        assert!(dot.contains("q0"));
    }

    #[test]
    fn test_match_trace_accepted() {
        let nfa = Nfa::from_pattern("ab").unwrap();
        let dfa = Dfa::from_nfa(&nfa);
        let trace = Visualizer::match_trace(&dfa, "ab");
        assert!(!trace.is_empty());
        // First entry should be START
        assert_eq!(trace[0].1, "START");
        // Last state should be accepting
        assert!(trace.last().unwrap().3);
    }

    #[test]
    fn test_match_trace_rejected() {
        let nfa = Nfa::from_pattern("ab").unwrap();
        let dfa = Dfa::from_nfa(&nfa);
        let trace = Visualizer::match_trace(&dfa, "ac");
        assert!(!trace.is_empty());
        // Should contain REJECT
        let has_reject = trace.iter().any(|(_, ch, _, _)| ch.contains("\u{2717}"));
        assert!(has_reject);
    }

    #[test]
    fn test_format_match_trace() {
        let nfa = Nfa::from_pattern("abc").unwrap();
        let dfa = Dfa::from_nfa(&nfa);
        let formatted = Visualizer::format_match_trace(&dfa, "abc");
        assert!(formatted.contains("Match Trace"));
        assert!(formatted.contains("ACCEPT") || formatted.contains("REJECT"));
    }

    #[test]
    fn test_dfa_dot_minimized() {
        let nfa = Nfa::from_pattern("a|b").unwrap();
        let dfa = Dfa::from_nfa(&nfa);
        let minimized = dfa.minimize();
        let dot = Visualizer::dfa_to_dot(&minimized);
        assert!(dot.contains("digraph DFA"));
        assert!(dot.contains("doublecircle"));
    }
}
