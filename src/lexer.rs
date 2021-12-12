use crate::lexer::LexError::InvalidError;
use crate::lexer::LexicalToken::{
    Comma, CreateTable, Date, Int, Json, LParen, NotNull, RParen, Semicolon, Text, Varchar,
};
use regex::Regex;
use std::str::from_utf8;

#[derive(PartialEq, Eq, Debug, Clone)]
pub enum LexicalToken {
    CreateTable,
    Int,
    Varchar,
    Json,
    Date,
    NotNull,
    LParen,
    RParen,
    Comma,
    Semicolon,
    Text(String),
}

pub struct MatchResult {
    pub token: LexicalToken,
    pub start: usize,
    pub end: usize,
}

trait Matcher {
    fn exec(&self, input: &[u8], position: usize) -> Option<MatchResult>;
}

struct RegexAndToken {
    regex: Regex,
    token: LexicalToken,
}

struct RegexMatcher {
    regex_and_tokens: Vec<RegexAndToken>,
}

impl RegexMatcher {
    fn new(word_and_tokens: Vec<(&str, LexicalToken)>) -> Self {
        let regex_and_tokens = word_and_tokens
            .iter()
            .map(|w| RegexAndToken {
                regex: Regex::new(w.0).unwrap(),
                token: w.1.clone(),
            })
            .collect();
        Self { regex_and_tokens }
    }
}

impl Matcher for RegexMatcher {
    fn exec(&self, input: &[u8], position: usize) -> Option<MatchResult> {
        let target = from_utf8(&input[position..]).ok()?;
        for regex_and_token in self.regex_and_tokens.iter() {
            if let Some(m) = regex_and_token.regex.find(target) {
                return Some(MatchResult {
                    token: regex_and_token.token.clone(),
                    start: position,
                    end: position + m.end() - 1,
                });
            }
        }
        None
    }
}

struct SymbolAndToken {
    symbol: u8,
    token: LexicalToken,
}

struct SymbolMatcher {
    symbols: Vec<SymbolAndToken>,
}

impl Matcher for SymbolMatcher {
    fn exec(&self, input: &[u8], position: usize) -> Option<MatchResult> {
        for symbol_and_token in self.symbols.iter() {
            if input[position] == symbol_and_token.symbol {
                return Some(MatchResult {
                    token: symbol_and_token.token.clone(),
                    start: position,
                    end: position,
                });
            }
        }
        None
    }
}

struct TextMatcher {}

impl TextMatcher {
    fn text_is_valid(&self, text: u8) -> bool {
        text != b',' && (text.is_ascii_alphanumeric() || text == b'_')
    }
}

impl Matcher for TextMatcher {
    fn exec(&self, input: &[u8], position: usize) -> Option<MatchResult> {
        let mut new_position = position;
        while new_position < input.len() && self.text_is_valid(input[new_position]) {
            new_position += 1;
        }
        if position != new_position {
            return Some(MatchResult {
                token: Text(from_utf8(&input[position..new_position]).ok()?.into()),
                start: position,
                end: new_position,
            });
        }
        None
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexError {
    InvalidError(String),
}

pub struct Lexer {
    matcher: Vec<Box<dyn Matcher>>,
}

impl Lexer {
    pub fn new() -> Self {
        Lexer {
            matcher: vec![
                Box::new(RegexMatcher::new(vec![
                    (r"^(?i)CREATE TABLE", CreateTable),
                    (r"^(?i)int(eger)?", Int),
                    (r"^(?i)json", Json),
                    (r"^(?i)varchar", Varchar),
                    (r"^(?i)date", Date),
                    (r"^(?i)Not Null", NotNull),
                ])),
                Box::new(SymbolMatcher {
                    symbols: vec![
                        SymbolAndToken {
                            symbol: b'(',
                            token: LParen,
                        },
                        SymbolAndToken {
                            symbol: b')',
                            token: RParen,
                        },
                        SymbolAndToken {
                            symbol: b';',
                            token: Semicolon,
                        },
                    ],
                }),
                Box::new(TextMatcher {}),
            ],
        }
    }

    fn check(&self, input: &[u8], position: usize) -> Result<MatchResult, LexError> {
        for matcher in &self.matcher {
            if let Some(result) = matcher.exec(input, position) {
                return Ok(result);
            }
        }
        Err(InvalidError(format!("Invalid input {}", input[position])))
    }

    fn skip_space(&self, input: &[u8], position: usize) -> usize {
        let mut new_position = position;
        while new_position < input.len() && b" \n\t".contains(&input[new_position]) {
            new_position += 1;
        }
        new_position
    }

    pub fn run(&self, input: &str) -> Result<Vec<LexicalToken>, LexError> {
        let input = input.as_bytes();
        let mut pos = 0;
        let mut tokens: Vec<LexicalToken> = Vec::new();
        while pos < input.len() {
            match input[pos] {
                b' ' | b'\n' | b'\t' => {
                    pos = self.skip_space(input, pos);
                }
                b',' => {
                    tokens.push(Comma);
                    pos += 1;
                }
                _ => {
                    let _k = input[pos];
                    let result = self.check(input, pos)?;
                    tokens.push(result.token);
                    pos = result.end + 1;
                }
            }
        }
        let _a = input[pos - 2];
        Ok(tokens)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::LexicalToken::{
        Comma, CreateTable, Date, Int, Json, LParen, NotNull, RParen, Semicolon, Text, Varchar,
    };
    use crate::Lexer;

    #[test]
    fn test_run() {
        let lexer = Lexer::new();
        let result = lexer
            .run("create table not NULL int integer json varchar date ( ) , ; \n \t test_test").unwrap_or_default();
        assert_eq!(
            result,
            vec![
                CreateTable,
                NotNull,
                Int,
                Int,
                Json,
                Varchar,
                Date,
                LParen,
                RParen,
                Comma,
                Semicolon,
                Text("test_test".into())
            ]
        );
    }
}
