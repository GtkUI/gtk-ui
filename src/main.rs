use std::fs;
use std::env;
use std::process;

mod lexer;
mod parser;

use lexer::Lexer;
use parser::Parser;


fn print_help() {
    println!("Usage: gtk-ui [FILENAME]");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        print_help();
        process::exit(0);
    }

    let filename = &args[1];
    let file_content = fs::read_to_string(filename)
        .expect("Something went wrong while trying to read the file");
    
    println!("Lexing...");
    let mut lexer = Lexer::new(file_content);
    lexer.lex();
    
    println!("Parsing...");
    let mut parser = Parser::new(lexer.tokens);
    parser.parse();

    println!("Statement length: {}", parser.statements.len());

    for statement in &parser.statements {
        println!("{:?}", statement);
    }
}
