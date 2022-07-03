use std::fs;
use std::env;
use std::process;

mod lexer;

use lexer::{lex, Token};

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
    
    let tokens = lex(&file_content);
    let mut unparsed_chars = 0;
    for token in tokens {
        if let Token::Number(n) = token {
            if n == -1 {
                unparsed_chars += 1;
                continue
            }
        }
        println!("{:?}", token);
    }
    println!("Unparsed Chars: {}/{} ({:.3}%)", unparsed_chars, file_content.len(), (unparsed_chars as f32)/(file_content.len() as f32)*100.0);
}
