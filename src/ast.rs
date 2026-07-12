//! AST (Abstract Syntax Tree) for regular expressions
//!
//! Represents the parsed structure of a regex pattern.

/// A node in the regex AST
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum AstNode {
    /// Literal character
    Literal(char),
    /// Dot (any character)
    Dot,
    /// Character class [...]
    CharacterClass {
        negated: bool,
        ranges: Vec<CharRange>,
    },
    /// Alternation (a|b)
    Alternation(Vec<RegexAst>),
    /// Concatenation
    Concatenation(Vec<RegexAst>),
    /// Zero or more (*)
    Star(Box<RegexAst>),
    /// One or more (+)
    Plus(Box<RegexAst>),
    /// Zero or one (?)
    Optional(Box<RegexAst>),
    /// Group (...)
    Group {
        capturing: bool,
        name: Option<String>,
        expr: Box<RegexAst>,
    },
    /// Repetition {n}, {n,}, {n,m}
    Repetition {
        min: u32,
        max: Option<u32>,
        greedy: bool,
        expr: Box<RegexAst>,
    },
    /// Start anchor (^)
    StartAnchor,
    /// End anchor ($)
    EndAnchor,
    /// Word boundary (\b)
    WordBoundary,
    /// Non-word boundary (\B)
    NonWordBoundary,
    /// Backreference (\1, \2, etc.)
    Backreference(u8),
    /// Named backreference (\k<name>)
    NamedBackreference(String),
}

/// A character range within a character class
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CharRange {
    pub start: char,
    pub end: char,
}

/// The root regex AST
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct RegexAst {
    pub root: AstNode,
}

impl RegexAst {
    /// Create a new AST from a root node
    pub fn new(root: AstNode) -> Self {
        Self { root }
    }

    /// Check if the pattern uses capturing groups
    pub fn has_capturing_groups(&self) -> bool {
        Self::has_capturing_groups_node(&self.root)
    }

    fn has_capturing_groups_node(node: &AstNode) -> bool {
        match node {
            AstNode::Group { capturing, .. } => *capturing,
            AstNode::Alternation(exprs) | AstNode::Concatenation(exprs) => exprs
                .iter()
                .any(|e| Self::has_capturing_groups_node(&e.root)),
            AstNode::Star(e) | AstNode::Plus(e) | AstNode::Optional(e) => {
                Self::has_capturing_groups_node(&e.root)
            }
            AstNode::Repetition { expr, .. } => Self::has_capturing_groups_node(&expr.root),
            _ => false,
        }
    }

    /// Count the number of capturing groups
    pub fn capturing_group_count(&self) -> usize {
        Self::count_capturing_groups_node(&self.root)
    }

    fn count_capturing_groups_node(node: &AstNode) -> usize {
        match node {
            AstNode::Group { capturing, .. } => {
                if *capturing {
                    1
                } else {
                    0
                }
            }
            AstNode::Alternation(exprs) | AstNode::Concatenation(exprs) => exprs
                .iter()
                .map(|e| Self::count_capturing_groups_node(&e.root))
                .sum(),
            AstNode::Star(e) | AstNode::Plus(e) | AstNode::Optional(e) => {
                Self::count_capturing_groups_node(&e.root)
            }
            AstNode::Repetition { expr, .. } => Self::count_capturing_groups_node(&expr.root),
            _ => 0,
        }
    }

    /// Calculate the pattern complexity (rough estimate of NFA states)
    pub fn complexity_score(&self) -> usize {
        Self::complexity_node(&self.root)
    }

    fn complexity_node(node: &AstNode) -> usize {
        match node {
            AstNode::Literal(_)
            | AstNode::Dot
            | AstNode::StartAnchor
            | AstNode::EndAnchor
            | AstNode::WordBoundary
            | AstNode::NonWordBoundary
            | AstNode::Backreference(_)
            | AstNode::NamedBackreference(_) => 1,
            AstNode::CharacterClass { ranges, .. } => 1 + ranges.len(),
            AstNode::Alternation(exprs) => {
                1 + exprs
                    .iter()
                    .map(|e| Self::complexity_node(&e.root))
                    .sum::<usize>()
            }
            AstNode::Concatenation(exprs) => {
                exprs.iter().map(|e| Self::complexity_node(&e.root)).sum()
            }
            AstNode::Star(e) | AstNode::Plus(e) | AstNode::Optional(e) => {
                1 + Self::complexity_node(&e.root)
            }
            AstNode::Repetition { expr, .. } => 2 + Self::complexity_node(&expr.root),
            AstNode::Group { expr, .. } => 1 + Self::complexity_node(&expr.root),
        }
    }
}

impl std::fmt::Display for RegexAst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fmt_node(&self.root, f)
    }
}

/// Detect if a character class matches a known shorthand notation
fn detect_shorthand(ranges: &[CharRange], negated: bool) -> Option<&'static str> {
    match (ranges, negated) {
        // \d = [0-9]
        (
            [CharRange {
                start: '0',
                end: '9',
            }],
            false,
        ) => Some("\\d"),
        // \D = [^0-9]
        (
            [CharRange {
                start: '0',
                end: '9',
            }],
            true,
        ) => Some("\\D"),
        // \w = [a-zA-Z0-9_]
        (
            [CharRange {
                start: 'a',
                end: 'z',
            }, CharRange {
                start: 'A',
                end: 'Z',
            }, CharRange {
                start: '0',
                end: '9',
            }, CharRange {
                start: '_',
                end: '_',
            }],
            false,
        ) => Some("\\w"),
        // \W = [^a-zA-Z0-9_]
        (
            [CharRange {
                start: 'a',
                end: 'z',
            }, CharRange {
                start: 'A',
                end: 'Z',
            }, CharRange {
                start: '0',
                end: '9',
            }, CharRange {
                start: '_',
                end: '_',
            }],
            true,
        ) => Some("\\W"),
        // \s = [ \t\n\r\f\v]
        (
            [CharRange {
                start: ' ',
                end: ' ',
            }, CharRange {
                start: '\t',
                end: '\t',
            }, CharRange {
                start: '\n',
                end: '\n',
            }, CharRange {
                start: '\r',
                end: '\r',
            }, CharRange {
                start: '\x0C',
                end: '\x0C',
            }, CharRange {
                start: '\x0B',
                end: '\x0B',
            }],
            false,
        ) => Some("\\s"),
        // \S = [^ \t\n\r\f\v]
        (
            [CharRange {
                start: ' ',
                end: ' ',
            }, CharRange {
                start: '\t',
                end: '\t',
            }, CharRange {
                start: '\n',
                end: '\n',
            }, CharRange {
                start: '\r',
                end: '\r',
            }, CharRange {
                start: '\x0C',
                end: '\x0C',
            }, CharRange {
                start: '\x0B',
                end: '\x0B',
            }],
            true,
        ) => Some("\\S"),
        _ => None,
    }
}

fn fmt_node(node: &AstNode, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match node {
        AstNode::Literal(c) => write!(f, "{}", c),
        AstNode::Dot => write!(f, "."),
        AstNode::CharacterClass { negated, ranges } => {
            // Check for known shorthand character classes
            if let Some(shorthand) = detect_shorthand(ranges, *negated) {
                return write!(f, "{}", shorthand);
            }
            // Fallback to bracket notation
            if *negated {
                write!(f, "[^")?;
            } else {
                write!(f, "[")?;
            }
            for r in ranges {
                if r.start == r.end {
                    write!(f, "{}", r.start)?;
                } else {
                    write!(f, "{}-{}", r.start, r.end)?;
                }
            }
            write!(f, "]")
        }
        AstNode::Alternation(exprs) => {
            for (i, e) in exprs.iter().enumerate() {
                if i > 0 {
                    write!(f, "|")?;
                }
                fmt_node(&e.root, f)?;
            }
            Ok(())
        }
        AstNode::Concatenation(exprs) => {
            for e in exprs {
                fmt_node(&e.root, f)?;
            }
            Ok(())
        }
        AstNode::Star(e) => {
            fmt_node(&e.root, f)?;
            write!(f, "*")
        }
        AstNode::Plus(e) => {
            fmt_node(&e.root, f)?;
            write!(f, "+")
        }
        AstNode::Optional(e) => {
            fmt_node(&e.root, f)?;
            write!(f, "?")
        }
        AstNode::Group {
            capturing,
            name,
            expr,
        } => {
            if let Some(n) = name {
                write!(f, "(?P<{}>", n)?;
            } else if *capturing {
                write!(f, "(")?;
            } else {
                write!(f, "(?:")?;
            }
            fmt_node(&expr.root, f)?;
            write!(f, ")")
        }
        AstNode::Repetition {
            min,
            max,
            greedy,
            expr,
        } => {
            fmt_node(&expr.root, f)?;
            match (min, max) {
                (n, None) if *n > 0 => write!(f, "{{{},}}", n)?,
                (n, Some(m)) if *n == *m => write!(f, "{{{}}}", n)?,
                (n, Some(m)) => write!(f, "{{{},{}}}", n, m)?,
                _ => {}
            }
            if !greedy {
                write!(f, "?")?;
            }
            Ok(())
        }
        AstNode::StartAnchor => write!(f, "^"),
        AstNode::EndAnchor => write!(f, "$"),
        AstNode::WordBoundary => write!(f, "\\b"),
        AstNode::NonWordBoundary => write!(f, "\\B"),
        AstNode::Backreference(n) => write!(f, "\\{}", n),
        AstNode::NamedBackreference(name) => write!(f, "\\k<{}>", name),
    }
}
