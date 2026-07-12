/// NFA (Non-deterministic Finite Automaton) construction
///
/// Implements Thompson's construction algorithm.
use std::collections::{HashSet, VecDeque};

/// NFA state ID
pub type StateId = usize;

/// Transition on the NFA
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Transition {
    /// Epsilon (empty) transition
    Epsilon,
    /// Match a specific character
    Char(char),
    /// Match any character
    Dot,
    /// Match a character in a set
    CharClass { negated: bool, chars: Vec<char> },
    /// Start anchor
    StartAnchor,
    /// End anchor
    EndAnchor,
    /// Word boundary
    WordBoundary,
}

/// An NFA state
#[derive(Debug, Clone)]
pub struct NfaState {
    pub id: StateId,
    pub transitions: Vec<(Transition, StateId)>,
    pub is_accept: bool,
}

/// NFA structure
#[derive(Debug, Clone)]
pub struct Nfa {
    pub states: Vec<NfaState>,
    pub start: StateId,
    pub accept: StateId,
}

impl Nfa {
    /// Create a new NFA with just start and accept states
    pub fn new() -> Self {
        let start = NfaState {
            id: 0,
            transitions: Vec::new(),
            is_accept: false,
        };
        let accept = NfaState {
            id: 1,
            transitions: Vec::new(),
            is_accept: true,
        };
        Self {
            states: vec![start, accept],
            start: 0,
            accept: 1,
        }
    }

    /// Add a new state and return its ID
    pub fn add_state(&mut self) -> StateId {
        let id = self.states.len();
        self.states.push(NfaState {
            id,
            transitions: Vec::new(),
            is_accept: false,
        });
        id
    }

    /// Mark a state as accepting
    pub fn set_accept(&mut self, id: StateId) {
        self.states[id].is_accept = true;
    }

    /// Add a transition
    pub fn add_transition(&mut self, from: StateId, trans: Transition, to: StateId) {
        self.states[from].transitions.push((trans, to));
    }

    /// Build NFA from a regex pattern string
    pub fn from_pattern(pattern: &str) -> crate::Result<Self> {
        let ast = crate::parser::RegexParser::parse(pattern)?;
        Ok(Self::from_ast(&ast))
    }

    /// Build NFA from an AST node using Thompson's construction
    pub fn from_ast(ast: &crate::ast::RegexAst) -> Self {
        let mut nfa = Self::new();
        let (start, accept) = nfa.build_node(&ast.root);
        nfa.start = start;
        nfa.accept = accept;
        nfa.set_accept(accept);
        nfa
    }

    fn build_node(&mut self, node: &crate::ast::AstNode) -> (StateId, StateId) {
        match node {
            crate::ast::AstNode::Literal(ch) => {
                let s = self.add_state();
                let e = self.add_state();
                self.add_transition(s, Transition::Char(*ch), e);
                (s, e)
            }
            crate::ast::AstNode::Dot => {
                let s = self.add_state();
                let e = self.add_state();
                self.add_transition(s, Transition::Dot, e);
                (s, e)
            }
            crate::ast::AstNode::CharacterClass { negated, ranges } => {
                let s = self.add_state();
                let e = self.add_state();
                let chars: Vec<char> = ranges
                    .iter()
                    .flat_map(|r| {
                        if r.start == r.end {
                            vec![r.start]
                        } else {
                            (r.start..=r.end).collect::<Vec<_>>()
                        }
                    })
                    .collect();
                self.add_transition(
                    s,
                    Transition::CharClass {
                        negated: *negated,
                        chars,
                    },
                    e,
                );
                (s, e)
            }
            crate::ast::AstNode::StartAnchor => {
                let s = self.add_state();
                let e = self.add_state();
                self.add_transition(s, Transition::StartAnchor, e);
                (s, e)
            }
            crate::ast::AstNode::EndAnchor => {
                let s = self.add_state();
                let e = self.add_state();
                self.add_transition(s, Transition::EndAnchor, e);
                (s, e)
            }
            crate::ast::AstNode::WordBoundary => {
                let s = self.add_state();
                let e = self.add_state();
                self.add_transition(s, Transition::WordBoundary, e);
                (s, e)
            }
            crate::ast::AstNode::Concatenation(exprs) => {
                if exprs.is_empty() {
                    let s = self.add_state();
                    let e = self.add_state();
                    self.add_transition(s, Transition::Epsilon, e);
                    return (s, e);
                }
                let (first_start, first_accept) = self.build_node(&exprs[0].root);
                let mut current_accept = first_accept;
                for expr in &exprs[1..] {
                    let (next_start, next_accept) = self.build_node(&expr.root);
                    self.add_transition(current_accept, Transition::Epsilon, next_start);
                    current_accept = next_accept;
                }
                (first_start, current_accept)
            }
            crate::ast::AstNode::Alternation(exprs) => {
                let s = self.add_state();
                let e = self.add_state();
                for expr in exprs {
                    let (start, accept) = self.build_node(&expr.root);
                    self.add_transition(s, Transition::Epsilon, start);
                    self.add_transition(accept, Transition::Epsilon, e);
                }
                (s, e)
            }
            crate::ast::AstNode::Star(expr) => {
                let s = self.add_state();
                let e = self.add_state();
                let (inner_start, inner_accept) = self.build_node(&expr.root);
                self.add_transition(s, Transition::Epsilon, inner_start);
                self.add_transition(s, Transition::Epsilon, e);
                self.add_transition(inner_accept, Transition::Epsilon, inner_start);
                self.add_transition(inner_accept, Transition::Epsilon, e);
                (s, e)
            }
            crate::ast::AstNode::Plus(expr) => {
                let s = self.add_state();
                let e = self.add_state();
                let (inner_start, inner_accept) = self.build_node(&expr.root);
                self.add_transition(s, Transition::Epsilon, inner_start);
                self.add_transition(inner_accept, Transition::Epsilon, inner_start);
                self.add_transition(inner_accept, Transition::Epsilon, e);
                (s, e)
            }
            crate::ast::AstNode::Optional(expr) => {
                let s = self.add_state();
                let e = self.add_state();
                let (inner_start, inner_accept) = self.build_node(&expr.root);
                self.add_transition(s, Transition::Epsilon, inner_start);
                self.add_transition(s, Transition::Epsilon, e);
                self.add_transition(inner_accept, Transition::Epsilon, e);
                (s, e)
            }
            crate::ast::AstNode::Group { expr, .. } => self.build_node(&expr.root),
            crate::ast::AstNode::Repetition {
                min,
                max,
                greedy: _,
                expr,
            } => {
                let s = self.add_state();
                let e = self.add_state();
                let mut last_accept = s;

                // Repeat min times
                for _ in 0..*min {
                    let (start, accept) = self.build_node(&expr.root);
                    self.add_transition(last_accept, Transition::Epsilon, start);
                    last_accept = accept;
                }

                // Handle max
                match max {
                    Some(m) => {
                        for _ in *min..*m {
                            let (start, accept) = self.build_node(&expr.root);
                            self.add_transition(last_accept, Transition::Epsilon, start);
                            self.add_transition(last_accept, Transition::Epsilon, e);
                            last_accept = accept;
                        }
                        self.add_transition(last_accept, Transition::Epsilon, e);
                    }
                    None => {
                        let (start, accept) = self.build_node(&expr.root);
                        self.add_transition(last_accept, Transition::Epsilon, start);
                        self.add_transition(last_accept, Transition::Epsilon, e);
                        self.add_transition(accept, Transition::Epsilon, start);
                        self.add_transition(accept, Transition::Epsilon, e);
                    }
                }

                (s, e)
            }
            _ => {
                let s = self.add_state();
                let e = self.add_state();
                self.add_transition(s, Transition::Epsilon, e);
                (s, e)
            }
        }
    }

    /// Get epsilon closure of a set of states at a given position
    pub fn epsilon_closure(&self, states: &HashSet<StateId>) -> HashSet<StateId> {
        let mut closure = states.clone();
        let mut queue: VecDeque<StateId> = states.iter().cloned().collect();

        while let Some(state) = queue.pop_front() {
            for (trans, target) in &self.states[state].transitions {
                if *trans == Transition::Epsilon && !closure.contains(target) {
                    closure.insert(*target);
                    queue.push_back(*target);
                }
            }
        }

        closure
    }

    /// Get epsilon closure at a specific position in the input
    fn epsilon_closure_at(
        &self,
        states: &HashSet<StateId>,
        pos: usize,
        input_len: usize,
    ) -> HashSet<StateId> {
        let mut closure = states.clone();
        let mut queue: VecDeque<StateId> = states.iter().cloned().collect();

        while let Some(state) = queue.pop_front() {
            for (trans, target) in &self.states[state].transitions {
                let is_enabled = match trans {
                    Transition::Epsilon => true,
                    Transition::StartAnchor => pos == 0,
                    Transition::EndAnchor => pos == input_len,
                    _ => false,
                };
                if is_enabled && !closure.contains(target) {
                    closure.insert(*target);
                    queue.push_back(*target);
                }
            }
        }

        closure
    }

    /// Follow transitions from a set of states on a character
    pub fn follow(&self, states: &HashSet<StateId>, ch: char) -> HashSet<StateId> {
        let mut result = HashSet::new();
        for &state in states {
            for (trans, target) in &self.states[state].transitions {
                match trans {
                    Transition::Char(c) if *c == ch => {
                        result.insert(*target);
                    }
                    Transition::Dot => {
                        if ch != '\n' {
                            result.insert(*target);
                        }
                    }
                    Transition::CharClass { negated, chars } => {
                        let contains = chars.contains(&ch);
                        if (*negated && !contains) || (!*negated && contains) {
                            result.insert(*target);
                        }
                    }
                    _ => {}
                }
            }
        }
        result
    }

    /// Check if a string is accepted by the NFA
    pub fn matches(&self, input: &str) -> bool {
        let mut current = HashSet::new();
        current.insert(self.start);
        current = self.epsilon_closure_at(&current, 0, input.len());

        for (pos, ch) in input.chars().enumerate() {
            current = self.follow(&current, ch);
            current = self.epsilon_closure_at(&current, pos + 1, input.len());
        }

        current.contains(&self.accept)
    }

    /// Get the number of states
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// Get the number of transitions
    pub fn transition_count(&self) -> usize {
        self.states.iter().map(|s| s.transitions.len()).sum()
    }
}

impl Default for Nfa {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_nfa() {
        let nfa = Nfa::from_pattern("abc").unwrap();
        assert!(nfa.matches("abc"));
        assert!(!nfa.matches("ab"));
        assert!(!nfa.matches("abcd"));
    }

    #[test]
    fn test_dot() {
        let nfa = Nfa::from_pattern("a.c").unwrap();
        assert!(nfa.matches("abc"));
        assert!(nfa.matches("aXc"));
        assert!(!nfa.matches("ac"));
    }

    #[test]
    fn test_star() {
        let nfa = Nfa::from_pattern("ab*c").unwrap();
        assert!(nfa.matches("ac"));
        assert!(nfa.matches("abc"));
        assert!(nfa.matches("abbbc"));
    }

    #[test]
    fn test_plus() {
        let nfa = Nfa::from_pattern("ab+c").unwrap();
        assert!(!nfa.matches("ac"));
        assert!(nfa.matches("abc"));
        assert!(nfa.matches("abbbc"));
    }

    #[test]
    fn test_optional() {
        let nfa = Nfa::from_pattern("ab?c").unwrap();
        assert!(nfa.matches("ac"));
        assert!(nfa.matches("abc"));
        assert!(!nfa.matches("abbc"));
    }

    #[test]
    fn test_alternation() {
        let nfa = Nfa::from_pattern("cat|dog").unwrap();
        assert!(nfa.matches("cat"));
        assert!(nfa.matches("dog"));
        assert!(!nfa.matches("bird"));
    }

    #[test]
    fn test_character_class() {
        let nfa = Nfa::from_pattern("[abc]+").unwrap();
        assert!(nfa.matches("a"));
        assert!(nfa.matches("abc"));
        assert!(!nfa.matches("def"));
    }

    #[test]
    fn test_negated_class() {
        let nfa = Nfa::from_pattern("[^abc]+").unwrap();
        assert!(!nfa.matches("a"));
        assert!(nfa.matches("def"));
    }

    #[test]
    fn test_complex_pattern() {
        let nfa = Nfa::from_pattern(r"^[a-z]+@[a-z]+\.[a-z]+$").unwrap();
        assert!(nfa.matches("user@example.com"));
        assert!(!nfa.matches("USER@EXAMPLE.COM"));
        assert!(!nfa.matches("invalid"));
    }
}
