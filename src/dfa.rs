/// DFA (Deterministic Finite Automaton) construction
///
/// Implements subset construction (NFA→DFA) and Hopcroft's minimization.
use std::collections::{HashMap, HashSet};

use crate::nfa::{Nfa, StateId, Transition};

/// DFA state ID
pub type DfaStateId = usize;

/// A DFA state (set of NFA states)
#[derive(Debug, Clone)]
pub struct DfaState {
    pub id: DfaStateId,
    pub nfa_states: HashSet<StateId>,
    pub is_accept: bool,
}

/// DFA transition table
pub type DfaTransitions = HashMap<(DfaStateId, char), DfaStateId>;

/// DFA structure
#[derive(Debug, Clone)]
pub struct Dfa {
    pub states: Vec<DfaState>,
    pub transitions: DfaTransitions,
    pub start: DfaStateId,
    pub alphabet: Vec<char>,
}

impl Dfa {
    /// Convert NFA to DFA using subset construction
    pub fn from_nfa(nfa: &Nfa) -> Self {
        let mut dfa = Self {
            states: Vec::new(),
            transitions: HashMap::new(),
            start: 0,
            alphabet: Self::extract_alphabet(nfa),
        };

        // Start state is epsilon closure of NFA start
        let mut start_nfa = HashSet::new();
        start_nfa.insert(nfa.start);
        let start_closure = nfa.epsilon_closure(&start_nfa);

        let start_id = dfa.add_dfa_state(&start_closure, nfa);
        dfa.start = start_id;

        let mut worklist = vec![start_id];
        let mut seen: HashMap<Vec<StateId>, DfaStateId> = HashMap::new();
        let mut start_vec: Vec<StateId> = start_closure.into_iter().collect();
        start_vec.sort();
        seen.insert(start_vec, start_id);

        while let Some(current) = worklist.pop() {
            let nfa_states = dfa.states[current].nfa_states.clone();

            for &ch in &dfa.alphabet.clone() {
                // Follow NFA transitions
                let mut moved = HashSet::new();
                for &ns in &nfa_states {
                    for (trans, target) in &nfa.states[ns].transitions {
                        match trans {
                            Transition::Char(c) if *c == ch => {
                                moved.insert(*target);
                            }
                            Transition::Dot if ch != '\n' => {
                                moved.insert(*target);
                            }
                            Transition::CharClass { negated, chars } => {
                                let contains = chars.contains(&ch);
                                if (*negated && !contains) || (!*negated && contains) {
                                    moved.insert(*target);
                                }
                            }
                            _ => {}
                        }
                    }
                }

                if moved.is_empty() {
                    continue;
                }

                let closure = nfa.epsilon_closure(&moved);
                let mut closure_vec: Vec<StateId> = closure.into_iter().collect();
                closure_vec.sort();

                if let Some(&existing) = seen.get(&closure_vec) {
                    dfa.transitions.insert((current, ch), existing);
                } else {
                    let new_id = dfa.add_dfa_state(&closure_vec.iter().cloned().collect(), nfa);
                    seen.insert(closure_vec, new_id);
                    dfa.transitions.insert((current, ch), new_id);
                    worklist.push(new_id);
                }
            }
        }

        dfa
    }

    fn add_dfa_state(&mut self, nfa_states: &HashSet<StateId>, nfa: &Nfa) -> DfaStateId {
        let id = self.states.len();
        let is_accept = nfa_states.contains(&nfa.accept);
        self.states.push(DfaState {
            id,
            nfa_states: nfa_states.clone(),
            is_accept,
        });
        id
    }

    fn extract_alphabet(nfa: &Nfa) -> Vec<char> {
        let mut chars: Vec<char> = Vec::new();
        for state in &nfa.states {
            for (trans, _) in &state.transitions {
                match trans {
                    Transition::Char(c) => {
                        if !chars.contains(c) {
                            chars.push(*c);
                        }
                    }
                    Transition::CharClass {
                        chars: class_chars, ..
                    } => {
                        for c in class_chars {
                            if !chars.contains(c) {
                                chars.push(*c);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        chars.sort();
        chars.dedup();
        chars
    }

    /// Minimize the DFA using Hopcroft's algorithm
    pub fn minimize(&self) -> Self {
        if self.states.is_empty() {
            return self.clone();
        }

        // Group states into accepting and non-accepting
        let mut partitions: Vec<HashSet<DfaStateId>> = Vec::new();
        let mut accept_set = HashSet::new();
        let mut non_accept_set = HashSet::new();

        for state in &self.states {
            if state.is_accept {
                accept_set.insert(state.id);
            } else {
                non_accept_set.insert(state.id);
            }
        }

        if !accept_set.is_empty() {
            partitions.push(accept_set);
        }
        if !non_accept_set.is_empty() {
            partitions.push(non_accept_set);
        }

        // Refine partitions
        let mut changed = true;
        while changed {
            changed = false;
            let mut new_partitions = Vec::new();

            for partition in &partitions {
                if partition.len() <= 1 {
                    new_partitions.push(partition.clone());
                    continue;
                }

                // Try to split this partition
                let mut groups: HashMap<Vec<DfaStateId>, HashSet<DfaStateId>> = HashMap::new();

                for &state_id in partition {
                    let mut signature = Vec::new();
                    for ch in &self.alphabet {
                        if let Some(&target) = self.transitions.get(&(state_id, *ch)) {
                            // Find which partition target belongs to
                            let target_partition = partitions
                                .iter()
                                .position(|p| p.contains(&target))
                                .unwrap_or(0);
                            signature.push(target_partition);
                        } else {
                            signature.push(usize::MAX);
                        }
                    }
                    groups.entry(signature).or_default().insert(state_id);
                }

                if groups.len() > 1 {
                    changed = true;
                }

                for (_, group) in groups {
                    new_partitions.push(group);
                }
            }

            partitions = new_partitions;
        }

        // Build minimized DFA
        let mut state_map: HashMap<DfaStateId, DfaStateId> = HashMap::new();
        let mut min_dfa = Self {
            states: Vec::new(),
            transitions: HashMap::new(),
            start: 0,
            alphabet: self.alphabet.clone(),
        };

        for partition in &partitions {
            let representative = *partition.iter().next().unwrap();
            let new_id = min_dfa.states.len();
            min_dfa.states.push(DfaState {
                id: new_id,
                nfa_states: HashSet::new(),
                is_accept: self.states[representative].is_accept,
            });

            for &state_id in partition {
                state_map.insert(state_id, new_id);
            }
        }

        // Map start state
        min_dfa.start = *state_map.get(&self.start).unwrap_or(&0);

        // Map transitions
        for (&(from, ch), &to) in &self.transitions {
            if let (Some(&new_from), Some(&new_to)) = (state_map.get(&from), state_map.get(&to)) {
                min_dfa.transitions.insert((new_from, ch), new_to);
            }
        }

        min_dfa
    }

    /// Check if a string is accepted by the DFA
    pub fn matches(&self, input: &str) -> bool {
        let mut current = self.start;
        for ch in input.chars() {
            match self.transitions.get(&(current, ch)) {
                Some(&next) => current = next,
                None => return false,
            }
        }
        self.states[current].is_accept
    }

    /// Get the number of states
    pub fn state_count(&self) -> usize {
        self.states.len()
    }

    /// Get the number of transitions
    pub fn transition_count(&self) -> usize {
        self.transitions.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::nfa::Nfa;

    #[test]
    fn test_nfa_to_dfa() {
        let nfa = Nfa::from_pattern("abc").unwrap();
        let dfa = Dfa::from_nfa(&nfa);
        assert!(dfa.matches("abc"));
        assert!(!dfa.matches("ab"));
        assert!(!dfa.matches("abcd"));
    }

    #[test]
    fn test_minimized_dfa() {
        let nfa = Nfa::from_pattern("a|b").unwrap();
        let dfa = Dfa::from_nfa(&nfa);
        let minimized = dfa.minimize();
        assert!(minimized.matches("a"));
        assert!(minimized.matches("b"));
        assert!(!minimized.matches("c"));
        // Minimized DFA should have fewer states
        assert!(minimized.state_count() <= dfa.state_count());
    }

    #[test]
    fn test_star_dfa() {
        let nfa = Nfa::from_pattern("ab*c").unwrap();
        let dfa = Dfa::from_nfa(&nfa);
        assert!(dfa.matches("ac"));
        assert!(dfa.matches("abc"));
        assert!(dfa.matches("abbbc"));
        assert!(!dfa.matches("ab"));
    }
}
