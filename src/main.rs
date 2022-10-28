use std::fs;
use std::env;
use std::process;

use gtk_ui::lexer::Lexer;
use gtk_ui::parser::Parser;
use gtk_ui::preprocessor::Preprocessor;
use gtk_ui::generator::Generator;
use gtk_ui::util::check_error;

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
    
    let mut lexer = Lexer::new(file_content.clone());
    check_error(lexer.lex(false), filename, &file_content);
    
    let mut parser = Parser::new(lexer.tokens, filename.clone());
    check_error(parser.parse(), filename, &file_content);

    let mut preprocessor = Preprocessor::new();
    check_error(preprocessor.preprocess(parser.statements, vec![filename.clone()]), filename, &file_content);
    
    let mut generator = Generator::new(preprocessor.statements);
    check_error(generator.generate(), filename, &file_content);
}
