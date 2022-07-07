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
    
    // let tokens = lex(&file_content);
    let mut lexer = Lexer::new(&file_content);
    let tokens = lexer.lex();
    
    /* Legacy code
    let mut unparsed_chars = 0;
    for token in &tokens {
        if let Token::Number(n) = token {
            if *n == -1 {
                unparsed_chars += 1;
                continue
            }
        }
        println!("{:?}", token);
    }
    println!("Unlexed Chars: {}/{} ({:.3}%)", unparsed_chars, file_content.len(), (unparsed_chars as f32)/(file_content.len() as f32)*100.0);
    */
    println!("Parsing...");
    let mut parser = Parser::new(tokens);
    let statements = parser.parse();

    println!("Statement length: {}", statements.len());

    for statement in statements {
        println!("{:?}", statement);
    }
}
