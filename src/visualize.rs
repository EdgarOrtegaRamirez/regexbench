/// Visualization module for regex patterns
///
/// Provides ASCII art visualization of NFA/DFA automata and match results.
use crate::dfa::Dfa;
use crate::nfa::{Nfa, Transition};

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
            let mut by_target: std::collections::HashMap<usize, Vec<char>> =
                std::collections::HashMap::new();
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
}
