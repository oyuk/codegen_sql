use regex::Regex;
use crate::lexer::LexError::InvalidError;
use crate::lexer::LexicalToken::{Comma, CreateTable, LParen, NotNull, Null, RParen, Semicolon, Text};
use std::str::from_utf8;

#[derive(PartialEq,Eq, Debug, Clone)]
pub enum LexicalToken {
    CreateTable,
    Null,
    NotNull,
    LParen,
    RParen,
    Comma,
    Semicolon,
    Text(String)
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
    token: LexicalToken
}

struct RegexMatcher {
    regex_and_tokens: Vec<RegexAndToken>
}

impl RegexMatcher {

    fn new(word_and_tokens: Vec<(&str, LexicalToken)>) -> Self {
        let regex_and_tokens = word_and_tokens.iter().map(|w| {
           RegexAndToken {
               regex: Regex::new(w.0).unwrap(),
               token: w.1.clone()
           }
        }).collect();
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
                    end: position + m.end() - 1
                })
            }
        }
        None
    }
}

struct SymbolAndToken {
    symbol: u8,
    token: LexicalToken
}

struct SymbolMatcher {
    symbols: Vec<SymbolAndToken>
}

impl Matcher for SymbolMatcher {
    fn exec(&self, input: &[u8], position: usize) -> Option<MatchResult> {
        for symbol_and_token in self.symbols.iter() {
            if input[position] == symbol_and_token.symbol {
                return Some(MatchResult {
                    token: symbol_and_token.token.clone(),
                    start: position,
                    end: position + 1 });
            }
        }
        None
    }
}

struct TextMatcher {

}

impl Matcher for TextMatcher {
    fn exec(&self, input: &[u8], position: usize) -> Option<MatchResult> {
        let mut new_position = position;
        while new_position < input.len() && input[new_position].is_ascii_alphanumeric() {
            new_position += 1;
        }
        if position != new_position {
            return Some(MatchResult {
                token: Text(from_utf8(&input[position..new_position]).ok()?.into()),
                start: position,
                end: new_position
            });
        }
        None
    }
}

pub enum LexError {
    InvalidError(String)
}

pub struct Lexer {
    matcher: Vec<Box<dyn Matcher>>
}

impl Lexer {
    
    pub fn new() -> Self {

        Lexer {
            matcher: vec![
                Box::new(RegexMatcher::new(vec![
                    (r"^(?i)CREATE TABLE".into(), CreateTable),
                    (r"^(?i)NULL".into(), Null),
                    (r"^(?i)Not Null".into(), NotNull)])),
                Box::new(SymbolMatcher { symbols: vec![
                    SymbolAndToken{ symbol: b'(', token: LParen },
                    SymbolAndToken{ symbol: b')', token: RParen },
                    SymbolAndToken{ symbol: b',', token: Comma },
                    SymbolAndToken{ symbol: b';', token: Semicolon }]}),
                Box::new(TextMatcher{})
            ]
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
                b' '| b'\n' | b'\t' => {
                    pos = self.skip_space(input, pos);
                }
                _ => {
                    let result = self.check(input, pos)?;
                    tokens.push(result.token);
                    pos = result.end + 1;
                }
            }
        }
        Ok(tokens)
    }

}

#[cfg(test)]
mod tests {
    use crate::Lexer;
    use crate::lexer::LexicalToken::{Comma, CreateTable, LParen, NotNull, Null, RParen, Semicolon, Text};

    #[test]
    fn test_run() {
        let lexer = Lexer::new();
        let result = lexer.run("create table NULL not NULL ( ) , ; \n \t test").unwrap_or(vec![]);
        assert_eq!(result , vec![CreateTable, Null, NotNull, LParen, RParen, Comma, Semicolon, Text("test".into())]);
    }
}
