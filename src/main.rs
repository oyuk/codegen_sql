mod interpreter;
mod lexer;
mod parser;

use crate::interpreter::Interpreter;
use crate::lexer::Lexer;
use crate::parser::Parser;

use std::env;
use std::fs::File;
use std::io::{Read};

/*

EXPR = CREATE TABLE Text ( EXPR1 ) ;
EXPR1 = EXPR2 {EXPR1}
EXPR2 = Text (Int|Json|Varchar|Date) [Not Null] ,

*/

fn main() {
    let filename = env::args().nth(1).expect("1 argument FILENAME required");
    let mut file = File::open(filename).expect("file open filed");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let lexer = Lexer::new();
    let tokens = match lexer.run(&contents) {
        Ok(tokens) => tokens,
        Err(error) => {
            panic!("Lexical error: {:?}", error)
        }
    };

    let parser = Parser::new();
    let ast = match parser.run(tokens) {
        Ok(ast) => ast,
        Err(error) => {
            panic!("Parse error: {:?}", error)
        }
    };

    let interpreter = Interpreter {};
    let result = interpreter.run(ast);

    println!("result:\n{:?}", result);
}
