use std::fs;
use std::env;
use std::process;

mod macros;
mod lexer;
mod parser;
mod preprocessor;
mod generator;
mod util;

use lexer::Lexer;
use parser::Parser;
use preprocessor::Preprocessor;
use generator::Generator;
use util::check_error;

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
    
    let mut lexer = Lexer::new(file_content);
    check_error(lexer.lex());
    
    let mut parser = Parser::new(lexer.tokens, filename.clone());
    check_error(parser.parse());

    let mut preprocessor = Preprocessor::new();
    check_error(preprocessor.preprocess(parser.statements, vec![filename.clone()]));
    
    let mut generator = Generator::new(preprocessor.statements);
    check_error(generator.generate());
}
