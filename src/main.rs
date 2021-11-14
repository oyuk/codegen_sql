mod table;
mod lexer;

use std::env;
use std::fs::File;
use std::io::{BufReader, Read};
use codegen::Scope;
use crate::lexer::Lexer;

fn main() {
    let filename = env::args().nth(1).expect("1 argument FILENAME required");
    let mut file = File::open(filename).expect("file open filed");
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("something went wrong reading the file");

    let lexer = Lexer::new();
    lexer.run(&contents);

    // println!("With text:\n{}", contents);

    // let mut scope = Scope::new();
    //
    // scope.new_struct("Foo")
    //     .derive("Debug")
    //     .field("one", "usize")
    //     .field("two", "String");
    //
    // println!("{}", scope.to_string());
}
