/// Regex matching engine
///
/// Provides backtracking-based matching with support for all regex features.
use crate::ast::{AstNode, RegexAst};

/// A match result
#[derive(Debug, Clone)]
pub struct MatchResult {
    /// Whether the pattern matched
    pub matched: bool,
    /// The matched substring
    pub text: Option<String>,
    /// Start index of match
    pub start: Option<usize>,
    /// End index of match
    pub end: Option<usize>,
    /// Captured groups
    pub groups: Vec<Option<String>>,
}

/// Regex engine using backtracking
pub struct RegexEngine {
    ast: RegexAst,
}

struct MatchContext<'a> {
    input: &'a [char],
    groups: Vec<Option<String>>,
}

impl RegexEngine {
    /// Create a new engine from a pattern
    pub fn new(pattern: &str) -> crate::Result<Self> {
        let ast = crate::parser::RegexParser::parse(pattern)?;
        Ok(Self { ast })
    }

    /// Create from an already-parsed AST
    pub fn from_ast(ast: RegexAst) -> Self {
        Self { ast }
    }

    /// Check if the pattern matches the entire input
    pub fn is_match(&self, input: &str) -> bool {
        self.find_match(input, 0).is_some()
    }

    /// Find the first match in the input
    pub fn find_match(&self, input: &str, start: usize) -> Option<MatchResult> {
        let chars: Vec<char> = input.chars().collect();

        // Try matching at each position
        for i in start..=chars.len() {
            let mut ctx = MatchContext {
                input: &chars,
                groups: vec![None; self.ast.capturing_group_count() + 1],
            };

            if let Some(end) = self.match_node(&self.ast.root, &mut ctx, i) {
                let matched_text: String = chars[i..end].iter().collect();
                return Some(MatchResult {
                    matched: true,
                    text: Some(matched_text),
                    start: Some(i),
                    end: Some(end),
                    groups: ctx.groups,
                });
            }
        }

        None
    }

    /// Find all matches in the input
    pub fn find_all(&self, input: &str) -> Vec<MatchResult> {
        let mut matches = Vec::new();
        let mut pos = 0;
        while let Some(m) = self.find_match(input, pos) {
            let end = m.end.unwrap_or(pos);
            matches.push(m);
            if end == pos {
                pos += 1;
            } else {
                pos = end;
            }
        }
        matches
    }

    /// Get all possible end positions for a node at a given position.
    /// This enables proper backtracking by letting the caller try alternatives.
    fn match_node_all(&self, node: &AstNode, ctx: &mut MatchContext, pos: usize) -> Vec<usize> {
        match node {
            AstNode::Literal(ch) => {
                if pos < ctx.input.len() && ctx.input[pos] == *ch {
                    vec![pos + 1]
                } else {
                    vec![]
                }
            }
            AstNode::Dot => {
                if pos < ctx.input.len() && ctx.input[pos] != '\n' {
                    vec![pos + 1]
                } else {
                    vec![]
                }
            }
            AstNode::CharacterClass { negated, ranges } => {
                if pos >= ctx.input.len() {
                    return vec![];
                }
                let ch = ctx.input[pos];
                let in_class = ranges.iter().any(|r| ch >= r.start && ch <= r.end);
                if (*negated && !in_class) || (!*negated && in_class) {
                    vec![pos + 1]
                } else {
                    vec![]
                }
            }
            AstNode::Concatenation(exprs) => self.match_concat_all(exprs, ctx, pos),
            AstNode::Alternation(exprs) => {
                let mut results = Vec::new();
                for expr in exprs {
                    let ends = self.match_node_all(&expr.root, ctx, pos);
                    results.extend(ends);
                    if !results.is_empty() {
                        // For alternation, return first successful branch's results
                        return results;
                    }
                }
                results
            }
            AstNode::Star(expr) => {
                // Match zero or more, return all valid end positions
                let mut results = vec![pos]; // zero matches
                let mut current = pos;
                loop {
                    let next_ends = self.match_node_all(&expr.root, ctx, current);
                    // Take only the first (shortest) match to avoid exponential blowup
                    if let Some(&end) = next_ends.first() {
                        if end > current {
                            results.push(end);
                            current = end;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                results
            }
            AstNode::Plus(expr) => {
                // Match one or more, return all valid end positions
                let mut results = Vec::new();
                let mut current = pos;
                loop {
                    let next_ends = self.match_node_all(&expr.root, ctx, current);
                    if let Some(&end) = next_ends.first() {
                        if end > current {
                            results.push(end);
                            current = end;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }
                results
            }
            AstNode::Optional(expr) => {
                let mut results = vec![pos]; // zero matches
                let optionals = self.match_node_all(&expr.root, ctx, pos);
                results.extend(optionals);
                results
            }
            AstNode::Repetition {
                min,
                max,
                greedy: _,
                expr,
            } => {
                // Collect all possible positions at each repetition count
                let _all_positions: Vec<Vec<usize>> = vec![vec![pos]];
                let mut current = pos;
                let mut count = 0;

                loop {
                    let next_ends = self.match_node_all(&expr.root, ctx, current);
                    if let Some(&end) = next_ends.first() {
                        if end > current {
                            count += 1;
                            current = end;
                            if let Some(m) = max {
                                if count >= *m {
                                    break;
                                }
                            }
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Now compute valid end positions based on min
                // We matched count times, positions are in a chain
                // Collect all positions along the chain
                let mut positions = vec![pos];
                let mut cur = pos;
                for _ in 0..count {
                    let next_ends = self.match_node_all(&expr.root, ctx, cur);
                    if let Some(&end) = next_ends.first() {
                        if end > cur {
                            positions.push(end);
                            cur = end;
                        } else {
                            break;
                        }
                    } else {
                        break;
                    }
                }

                // Return all positions that satisfy min constraint
                positions.into_iter().skip(*min as usize).collect()
            }
            AstNode::Group {
                capturing,
                name: _,
                expr,
            } => {
                let group_idx = if *capturing {
                    Some(self.get_group_index(node))
                } else {
                    None
                };

                let ends = self.match_node_all(&expr.root, ctx, pos);

                // For capturing groups, set the captured text for the first match
                if let (Some(idx), Some(&end)) = (group_idx, ends.first()) {
                    let text: String = ctx.input[pos..end].iter().collect();
                    if idx < ctx.groups.len() {
                        ctx.groups[idx] = Some(text);
                    }
                }

                ends
            }
            AstNode::StartAnchor => {
                if pos == 0 {
                    vec![pos]
                } else {
                    vec![]
                }
            }
            AstNode::EndAnchor => {
                if pos >= ctx.input.len() {
                    vec![pos]
                } else {
                    vec![]
                }
            }
            AstNode::WordBoundary => {
                let prev_is_word = if pos > 0 {
                    ctx.input[pos - 1].is_alphanumeric() || ctx.input[pos - 1] == '_'
                } else {
                    false
                };
                let next_is_word = if pos < ctx.input.len() {
                    ctx.input[pos].is_alphanumeric() || ctx.input[pos] == '_'
                } else {
                    false
                };
                if prev_is_word != next_is_word {
                    vec![pos]
                } else {
                    vec![]
                }
            }
            AstNode::NonWordBoundary => {
                let prev_is_word = if pos > 0 {
                    ctx.input[pos - 1].is_alphanumeric() || ctx.input[pos - 1] == '_'
                } else {
                    false
                };
                let next_is_word = if pos < ctx.input.len() {
                    ctx.input[pos].is_alphanumeric() || ctx.input[pos] == '_'
                } else {
                    false
                };
                if prev_is_word == next_is_word {
                    vec![pos]
                } else {
                    vec![]
                }
            }
            _ => vec![pos],
        }
    }

    /// Match a concatenation of expressions, trying all possible end positions
    fn match_concat_all(
        &self,
        exprs: &[RegexAst],
        ctx: &mut MatchContext,
        pos: usize,
    ) -> Vec<usize> {
        if exprs.is_empty() {
            return vec![pos];
        }
        let first = &exprs[0];
        let rest = &exprs[1..];

        let first_ends = self.match_node_all(&first.root, ctx, pos);
        let mut results = Vec::new();

        for &end in &first_ends {
            let rest_ends = self.match_concat_all(rest, ctx, end);
            results.extend(rest_ends);
        }

        // Return the longest (greedy) match
        if let Some(&last) = results.last() {
            vec![last]
        } else {
            results
        }
    }

    /// Match a node at a given position (backward compatible)
    fn match_node(&self, node: &AstNode, ctx: &mut MatchContext, pos: usize) -> Option<usize> {
        let ends = self.match_node_all(node, ctx, pos);
        // Return the longest (greedy) match
        ends.into_iter().next_back()
    }

    fn get_group_index(&self, target: &AstNode) -> usize {
        let mut count = 1;
        self.count_groups_before(&self.ast.root, target, &mut count);
        count
    }

    fn count_groups_before(&self, current: &AstNode, target: &AstNode, count: &mut usize) -> bool {
        if std::ptr::eq(current as *const AstNode, target as *const AstNode) {
            return true;
        }

        match current {
            AstNode::Group {
                capturing, expr, ..
            } => {
                if *capturing {
                    let result = self.count_groups_before(&expr.root, target, count);
                    if result {
                        return true;
                    }
                    *count += 1;
                } else {
                    return self.count_groups_before(&expr.root, target, count);
                }
            }
            AstNode::Concatenation(exprs) | AstNode::Alternation(exprs) => {
                for expr in exprs {
                    if self.count_groups_before(&expr.root, target, count) {
                        return true;
                    }
                }
            }
            AstNode::Star(e) | AstNode::Plus(e) | AstNode::Optional(e) => {
                return self.count_groups_before(&e.root, target, count);
            }
            AstNode::Repetition { expr, .. } => {
                return self.count_groups_before(&expr.root, target, count);
            }
            _ => {}
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_match() {
        let engine = RegexEngine::new("abc").unwrap();
        assert!(engine.is_match("abc"));
        assert!(!engine.is_match("abd"));
    }

    #[test]
    fn test_find_match() {
        let engine = RegexEngine::new(r"\d+").unwrap();
        let result = engine.find_match("abc 123 def", 0).unwrap();
        assert_eq!(result.text.as_deref(), Some("123"));
    }

    #[test]
    fn test_find_all() {
        let engine = RegexEngine::new(r"\d+").unwrap();
        let results = engine.find_all("12 abc 34 def 56");
        assert_eq!(results.len(), 3);
    }

    #[test]
    fn test_group_capture() {
        let engine = RegexEngine::new(r"(\d+)-(\d+)").unwrap();
        let result = engine.find_match("123-456", 0).unwrap();
        assert_eq!(result.groups.len(), 3); // full match + 2 groups
    }
}
