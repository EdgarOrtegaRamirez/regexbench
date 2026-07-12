/// Regex parser - converts regex string to AST
///
/// Supports: literals, dot, character classes, groups, alternation,
/// quantifiers (*, +, ?, {n,m}), anchors, backreferences.
use crate::ast::{AstNode, CharRange, RegexAst};
use anyhow::{bail, Result};

pub struct RegexParser<'a> {
    input: &'a str,
    pos: usize,
}

impl<'a> RegexParser<'a> {
    pub fn new(input: &'a str) -> Self {
        Self { input, pos: 0 }
    }

    pub fn parse(input: &str) -> Result<RegexAst> {
        let mut parser = RegexParser::new(input);
        let ast = parser.parse_alternation()?;
        if parser.pos < parser.input.len() {
            bail!(
                "Unexpected character '{}' at position {}",
                parser.input.chars().nth(parser.pos).unwrap_or('?'),
                parser.pos
            );
        }
        Ok(ast)
    }

    fn peek(&self) -> Option<char> {
        self.input[self.pos..].chars().next()
    }

    fn advance(&mut self) -> Option<char> {
        let ch = self.peek()?;
        self.pos += ch.len_utf8();
        Some(ch)
    }

    fn expect(&mut self, expected: char) -> Result<()> {
        match self.advance() {
            Some(ch) if ch == expected => Ok(()),
            Some(ch) => bail!("Expected '{}', found '{}'", expected, ch),
            None => bail!("Expected '{}', found end of input", expected),
        }
    }

    fn parse_alternation(&mut self) -> Result<RegexAst> {
        let mut branches = vec![self.parse_concatenation()?];

        while self.peek() == Some('|') {
            self.advance();
            branches.push(self.parse_concatenation()?);
        }

        if branches.len() == 1 {
            Ok(branches.remove(0))
        } else {
            Ok(RegexAst::new(AstNode::Alternation(branches)))
        }
    }

    fn parse_concatenation(&mut self) -> Result<RegexAst> {
        let mut nodes = Vec::new();

        while let Some(ch) = self.peek() {
            match ch {
                '|' | ')' => break,
                _ => nodes.push(self.parse_quantified()?),
            }
        }

        if nodes.len() == 1 {
            Ok(nodes.remove(0))
        } else if nodes.is_empty() {
            Ok(RegexAst::new(AstNode::Concatenation(vec![])))
        } else {
            Ok(RegexAst::new(AstNode::Concatenation(nodes)))
        }
    }

    fn parse_quantified(&mut self) -> Result<RegexAst> {
        let base = self.parse_atom()?;

        if let Some(ch) = self.peek() {
            match ch {
                '*' => {
                    self.advance();
                    let _greedy = !self.eat_if('?');
                    return Ok(RegexAst::new(AstNode::Star(Box::new(base))));
                }
                '+' => {
                    self.advance();
                    let _greedy = !self.eat_if('?');
                    return Ok(RegexAst::new(AstNode::Plus(Box::new(base))));
                }
                '?' => {
                    self.advance();
                    let _greedy = !self.eat_if('?');
                    return Ok(RegexAst::new(AstNode::Optional(Box::new(base))));
                }
                '{' => {
                    return self.parse_repetition(base);
                }
                _ => {}
            }
        }

        Ok(base)
    }

    fn parse_repetition(&mut self, base: RegexAst) -> Result<RegexAst> {
        self.expect('{')?;

        let min = self.parse_number()?;

        let max = if self.peek() == Some(',') {
            self.advance();
            if self.peek() == Some('}') {
                None
            } else {
                Some(self.parse_number()?)
            }
        } else {
            Some(min)
        };

        self.expect('}')?;

        let greedy = !self.eat_if('?');

        Ok(RegexAst::new(AstNode::Repetition {
            min,
            max,
            greedy,
            expr: Box::new(base),
        }))
    }

    fn parse_number(&mut self) -> Result<u32> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            bail!("Expected number at position {}", self.pos);
        }
        self.input[start..self.pos]
            .parse::<u32>()
            .map_err(|e| anyhow::anyhow!("{}", e))
    }

    fn parse_atom(&mut self) -> Result<RegexAst> {
        let ch = self
            .peek()
            .ok_or_else(|| anyhow::anyhow!("Unexpected end of pattern"))?;

        match ch {
            '.' => {
                self.advance();
                Ok(RegexAst::new(AstNode::Dot))
            }
            '(' => {
                self.advance();
                self.parse_group()
            }
            '[' => {
                self.advance();
                self.parse_character_class()
            }
            '^' => {
                self.advance();
                Ok(RegexAst::new(AstNode::StartAnchor))
            }
            '$' => {
                self.advance();
                Ok(RegexAst::new(AstNode::EndAnchor))
            }
            '\\' => {
                self.advance();
                self.parse_escape()
            }
            _ => {
                self.advance();
                Ok(RegexAst::new(AstNode::Literal(ch)))
            }
        }
    }

    fn parse_group(&mut self) -> Result<RegexAst> {
        // Check for special group syntax
        if self.peek() == Some('?') {
            self.advance();
            match self.peek() {
                Some(':') => {
                    self.advance();
                    let expr = self.parse_alternation()?;
                    self.expect(')')?;
                    return Ok(RegexAst::new(AstNode::Group {
                        capturing: false,
                        name: None,
                        expr: Box::new(expr),
                    }));
                }
                Some('P') => {
                    self.advance();
                    self.expect('<')?;
                    let name = self.parse_name()?;
                    self.expect('>')?;
                    let expr = self.parse_alternation()?;
                    self.expect(')')?;
                    return Ok(RegexAst::new(AstNode::Group {
                        capturing: true,
                        name: Some(name),
                        expr: Box::new(expr),
                    }));
                }
                _ => {
                    bail!("Unsupported group syntax at position {}", self.pos - 1);
                }
            }
        }

        let expr = self.parse_alternation()?;
        self.expect(')')?;
        Ok(RegexAst::new(AstNode::Group {
            capturing: true,
            name: None,
            expr: Box::new(expr),
        }))
    }

    fn parse_name(&mut self) -> Result<String> {
        let start = self.pos;
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                self.advance();
            } else {
                break;
            }
        }
        if self.pos == start {
            bail!("Expected name at position {}", self.pos);
        }
        Ok(self.input[start..self.pos].to_string())
    }

    fn parse_character_class(&mut self) -> Result<RegexAst> {
        let negated = self.eat_if('^');
        let mut ranges = Vec::new();

        // Handle ] as first character in class
        if self.peek() == Some(']') {
            self.advance();
            ranges.push(CharRange {
                start: ']',
                end: ']',
            });
        }

        while let Some(ch) = self.peek() {
            if ch == ']' {
                self.advance();
                return Ok(RegexAst::new(AstNode::CharacterClass { negated, ranges }));
            }

            let start = if ch == '\\' {
                self.advance();
                self.parse_escape_char()?
            } else {
                self.advance();
                ch
            };

            if self.peek() == Some('-') {
                self.advance();
                if self.peek() == Some(']') {
                    // '-' at end of class: push previous char as literal, then '-' as literal
                    ranges.push(CharRange { start, end: start });
                    ranges.push(CharRange {
                        start: '-',
                        end: '-',
                    });
                } else {
                    let end = if self.peek() == Some('\\') {
                        self.advance();
                        self.parse_escape_char()?
                    } else {
                        self.advance().unwrap_or(']')
                    };
                    ranges.push(CharRange { start, end });
                }
            } else {
                ranges.push(CharRange { start, end: start });
            }
        }

        bail!("Unclosed character class");
    }

    fn parse_escape(&mut self) -> Result<RegexAst> {
        match self.peek() {
            Some('d') => {
                self.advance();
                // \d = [0-9]
                Ok(RegexAst::new(AstNode::CharacterClass {
                    negated: false,
                    ranges: vec![CharRange {
                        start: '0',
                        end: '9',
                    }],
                }))
            }
            Some('D') => {
                self.advance();
                // \D = [^0-9]
                Ok(RegexAst::new(AstNode::CharacterClass {
                    negated: true,
                    ranges: vec![CharRange {
                        start: '0',
                        end: '9',
                    }],
                }))
            }
            Some('s') => {
                self.advance();
                // \s = [ \t\n\r\f\v]
                Ok(RegexAst::new(AstNode::CharacterClass {
                    negated: false,
                    ranges: vec![
                        CharRange {
                            start: ' ',
                            end: ' ',
                        },
                        CharRange {
                            start: '\t',
                            end: '\t',
                        },
                        CharRange {
                            start: '\n',
                            end: '\n',
                        },
                        CharRange {
                            start: '\r',
                            end: '\r',
                        },
                        CharRange {
                            start: '\x0C',
                            end: '\x0C',
                        },
                        CharRange {
                            start: '\x0B',
                            end: '\x0B',
                        },
                    ],
                }))
            }
            Some('S') => {
                self.advance();
                // \S = [^ \t\n\r\f\v]
                Ok(RegexAst::new(AstNode::CharacterClass {
                    negated: true,
                    ranges: vec![
                        CharRange {
                            start: ' ',
                            end: ' ',
                        },
                        CharRange {
                            start: '\t',
                            end: '\t',
                        },
                        CharRange {
                            start: '\n',
                            end: '\n',
                        },
                        CharRange {
                            start: '\r',
                            end: '\r',
                        },
                        CharRange {
                            start: '\x0C',
                            end: '\x0C',
                        },
                        CharRange {
                            start: '\x0B',
                            end: '\x0B',
                        },
                    ],
                }))
            }
            Some('w') => {
                self.advance();
                // \w = [a-zA-Z0-9_]
                Ok(RegexAst::new(AstNode::CharacterClass {
                    negated: false,
                    ranges: vec![
                        CharRange {
                            start: 'a',
                            end: 'z',
                        },
                        CharRange {
                            start: 'A',
                            end: 'Z',
                        },
                        CharRange {
                            start: '0',
                            end: '9',
                        },
                        CharRange {
                            start: '_',
                            end: '_',
                        },
                    ],
                }))
            }
            Some('W') => {
                self.advance();
                // \W = [^a-zA-Z0-9_]
                Ok(RegexAst::new(AstNode::CharacterClass {
                    negated: true,
                    ranges: vec![
                        CharRange {
                            start: 'a',
                            end: 'z',
                        },
                        CharRange {
                            start: 'A',
                            end: 'Z',
                        },
                        CharRange {
                            start: '0',
                            end: '9',
                        },
                        CharRange {
                            start: '_',
                            end: '_',
                        },
                    ],
                }))
            }
            Some('b') => {
                self.advance();
                Ok(RegexAst::new(AstNode::WordBoundary))
            }
            Some('B') => {
                self.advance();
                Ok(RegexAst::new(AstNode::NonWordBoundary))
            }
            Some(ch) if ch.is_ascii_digit() => {
                self.advance();
                let n = ch.to_digit(10).unwrap() as u8;
                Ok(RegexAst::new(AstNode::Backreference(n)))
            }
            Some('k') => {
                self.advance();
                self.expect('<')?;
                let name = self.parse_name()?;
                self.expect('>')?;
                Ok(RegexAst::new(AstNode::NamedBackreference(name)))
            }
            Some(ch) => {
                self.advance();
                Ok(RegexAst::new(AstNode::Literal(Self::escape_char(ch))))
            }
            None => bail!("Unexpected end after backslash"),
        }
    }

    fn parse_escape_char(&mut self) -> Result<char> {
        match self.advance() {
            Some(ch) => Ok(Self::escape_char(ch)),
            None => bail!("Unexpected end after backslash"),
        }
    }

    fn escape_char(ch: char) -> char {
        match ch {
            'n' => '\n',
            'r' => '\r',
            't' => '\t',
            '0' => '\0',
            _ => ch,
        }
    }

    fn eat_if(&mut self, expected: char) -> bool {
        if self.peek() == Some(expected) {
            self.advance();
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_literal() {
        let ast = RegexParser::parse("abc").unwrap();
        assert_eq!(ast.to_string(), "abc");
    }

    #[test]
    fn test_dot() {
        let ast = RegexParser::parse("a.b").unwrap();
        assert_eq!(ast.to_string(), "a.b");
    }

    #[test]
    fn test_quantifiers() {
        let ast = RegexParser::parse("a*b+c?").unwrap();
        assert_eq!(ast.to_string(), "a*b+c?");
    }

    #[test]
    fn test_group() {
        let ast = RegexParser::parse("(abc)").unwrap();
        assert!(ast.has_capturing_groups());
        assert_eq!(ast.capturing_group_count(), 1);
    }

    #[test]
    fn test_non_capturing_group() {
        let ast = RegexParser::parse("(?:abc)").unwrap();
        assert!(!ast.has_capturing_groups());
    }

    #[test]
    fn test_named_group() {
        let ast = RegexParser::parse("(?P<name>abc)").unwrap();
        assert!(ast.has_capturing_groups());
    }

    #[test]
    fn test_alternation() {
        let ast = RegexParser::parse("a|b|c").unwrap();
        assert_eq!(ast.to_string(), "a|b|c");
    }

    #[test]
    fn test_character_class() {
        let ast = RegexParser::parse("[a-z]").unwrap();
        assert_eq!(ast.to_string(), "[a-z]");
    }

    #[test]
    fn test_negated_class() {
        let ast = RegexParser::parse("[^abc]").unwrap();
        assert_eq!(ast.to_string(), "[^abc]");
    }

    #[test]
    fn test_anchors() {
        let ast = RegexParser::parse("^abc$").unwrap();
        assert_eq!(ast.to_string(), "^abc$");
    }

    #[test]
    fn test_repetition() {
        let ast = RegexParser::parse("a{3}").unwrap();
        assert_eq!(ast.to_string(), "a{3}");

        let ast = RegexParser::parse("a{2,}").unwrap();
        assert_eq!(ast.to_string(), "a{2,}");

        let ast = RegexParser::parse("a{2,5}").unwrap();
        assert_eq!(ast.to_string(), "a{2,5}");
    }

    #[test]
    fn test_backreference() {
        let ast = RegexParser::parse("(a)\\1").unwrap();
        assert_eq!(ast.to_string(), "(a)\\1");
    }

    #[test]
    fn test_complex_pattern() {
        let ast = RegexParser::parse(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
        assert!(ast.to_string().contains("@"));
    }

    #[test]
    fn test_escape_sequences() {
        let ast = RegexParser::parse(r"\d+\s+\w+").unwrap();
        assert_eq!(ast.to_string(), r"\d+\s+\w+");
    }

    #[test]
    fn test_word_boundary() {
        let ast = RegexParser::parse(r"\bword\b").unwrap();
        assert_eq!(ast.to_string(), r"\bword\b");
    }
}
