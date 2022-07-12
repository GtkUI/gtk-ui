use std::fs;
use std::env;
use std::process;

mod lexer;
mod parser;
mod preprocessor;

use lexer::Lexer;
use parser::Parser;
use preprocessor::Preprocessor;


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

    println!("Preprocessing...");
    let mut preprocessor = Preprocessor::new();
    preprocessor.preprocess(parser.statements, vec![filename.clone()]);
    
    for statement in &preprocessor.statements {
        println!("{:?}", statement);
    }
}
