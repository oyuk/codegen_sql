use crate::lexer::LexicalToken;
use crate::lexer::LexicalToken::{RParen, Semicolon};
use crate::parser::Ast::{Expr, Expr1, Expr2};
use crate::parser::ParseError::{Eof, UnexpectedToken};
use std::error::Error;
use std::fmt;
use std::fmt::Display;
use std::iter::Peekable;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ColumnType {
    Int,
    Json,
    Varchar,
    Date,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Ast {
    Expr2 {
        name: String,
        column_type: ColumnType,
        null: bool,
    },
    Expr1 {
        expr2: Box<Ast>,
        expr1: Option<Box<Ast>>,
    },
    Expr {
        table_name: String,
        expr1: Box<Ast>,
    },
}

pub struct Parser {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError {
    UnexpectedToken(LexicalToken),
    Eof,
}

impl Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use self::ParseError::*;
        match self {
            UnexpectedToken(token) => write!(f, "UnexpectedToken: {:?}", token),
            Eof => write!(f, "Eof"),
        }
    }
}

impl Error for ParseError {}

impl Parser {
    pub fn new() -> Self {
        Parser {}
    }

    fn parse_expr<LexicalTokens>(
        &self,
        tokens: &mut Peekable<LexicalTokens>,
    ) -> Result<Ast, Box<dyn Error>>
    where
        LexicalTokens: Iterator<Item = LexicalToken>,
    {
        self.parse_token(tokens, LexicalToken::CreateTable)?;
        let table_name = self.parse_text(tokens.next())?;
        self.parse_token(tokens, LexicalToken::LParen)?;
        let expr1 = self.parse_expr1(tokens)?;
        self.parse_token(tokens, RParen)?;
        self.parse_token(tokens, Semicolon)?;
        Ok(Expr {
            table_name,
            expr1: Box::new(expr1),
        })
    }

    fn parse_expr1<LexicalTokens>(
        &self,
        tokens: &mut Peekable<LexicalTokens>,
    ) -> Result<Ast, ParseError>
    where
        LexicalTokens: Iterator<Item = LexicalToken>,
    {
        let expr2 = self.parse_expr2(tokens)?;
        if matches!(tokens.peek(), Some(&LexicalToken::Text(_))) {
            let expr1 = self.parse_expr1(tokens)?;
            return Ok(Expr1 {
                expr2: Box::new(expr2),
                expr1: Some(Box::new(expr1)),
            });
        };
        Ok(Expr1 {
            expr2: Box::new(expr2),
            expr1: None,
        })
    }

    fn parse_expr2<LexicalTokens>(
        &self,
        tokens: &mut Peekable<LexicalTokens>,
    ) -> Result<Ast, ParseError>
    where
        LexicalTokens: Iterator<Item = LexicalToken>,
    {
        let name = self.parse_text(tokens.next())?;
        let column_type = self.parse_column_type(tokens.next())?;
        let next_token = tokens.peek();
        let null = match next_token {
            Some(LexicalToken::NotNull) => {
                tokens.next();
                false
            }
            _ => true,
        };
        let comma = tokens.next();
        if comma != Some(LexicalToken::Comma) {
            return Err(UnexpectedToken(comma.unwrap()));
        }
        Ok(Expr2 {
            name,
            column_type,
            null,
        })
    }

    fn parse_token<LexicalTokens>(
        &self,
        tokens: &mut Peekable<LexicalTokens>,
        target: LexicalToken,
    ) -> Result<(), ParseError>
    where
        LexicalTokens: Iterator<Item = LexicalToken>,
    {
        let token = tokens.next().ok_or(Eof)?;
        if token == target {
            return Ok(());
        }
        Err(UnexpectedToken(token))
    }

    fn parse_text(&self, token: Option<LexicalToken>) -> Result<String, ParseError> {
        match token {
            Some(LexicalToken::Text(text)) => Ok(text),
            Some(token) => Err(UnexpectedToken(token)),
            _ => Err(Eof),
        }
    }

    fn parse_column_type(&self, token: Option<LexicalToken>) -> Result<ColumnType, ParseError> {
        match token {
            Some(LexicalToken::Int) => Ok(ColumnType::Int),
            Some(LexicalToken::Json) => Ok(ColumnType::Json),
            Some(LexicalToken::Varchar) => Ok(ColumnType::Varchar),
            Some(LexicalToken::Date) => Ok(ColumnType::Date),
            Some(token) => Err(UnexpectedToken(token)),
            _ => Err(Eof),
        }
    }

    pub fn run(&self, tokens: Vec<LexicalToken>) -> Result<Ast, Box<dyn Error>> {
        let mut tokens = tokens.into_iter().peekable();
        self.parse_expr(&mut tokens)
    }
}

#[cfg(test)]
mod tests {
    use crate::lexer::LexicalToken::{
        Comma, CreateTable, Date, Int, LParen, NotNull, RParen, Semicolon, Text,
    };
    use crate::parser::Ast::{Expr, Expr1, Expr2};
    use crate::parser::{ColumnType, Parser};
    

    #[test]
    fn test_run() {
        let parser = Parser::new();
        let tokens = vec![
            CreateTable,
            Text("table_name".into()),
            LParen,
            Text("column_name1".into()),
            Int,
            NotNull,
            Comma,
            Text("column_name2".into()),
            Date,
            Comma,
            RParen,
            Semicolon,
        ];
        let result = parser.run(tokens);
        assert_eq!(
            result.ok(),
            Some(Expr {
                table_name: "table_name".into(),
                expr1: Box::from(Expr1 {
                    expr2: Box::from(Expr2 {
                        name: "column_name1".into(),
                        column_type: ColumnType::Int,
                        null: false
                    }),
                    expr1: Some(Box::from(Expr1 {
                        expr2: Box::from(Expr2 {
                            name: "column_name2".into(),
                            column_type: ColumnType::Date,
                            null: true
                        }),
                        expr1: None
                    }))
                })
            })
        );
    }
}
