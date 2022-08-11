use super::lexer::Lexer;
use super::parser::{
    Parser,
    Statement
};
use super::util::check_error;
use std::fs;

const LIB_PATH: &str =  "./libs";

fn path_exists(path: &String) -> bool {
    fs::metadata(path).is_ok()
}

// TODO: Warn about a file being included multiple times.
// At the moment, regardless of where the library is included, if it is included twice, everything defined in the library is redefined
// This is of course very easily avoidable, but for user friendliness, a notification about this would be nice

pub struct Preprocessor {
    pub statements: Vec<Statement>,
}

impl Preprocessor { 

    // Pubs
    pub fn preprocess(&mut self, input: Vec<Statement>, included_files: Vec<String>) {
        for statement in input {
            match statement {
                Statement::Include(path) => {
                    let file_content;
                    let file_path: String;

                    let lib_file_path = format!("{LIB_PATH}/{path}.gui");
                    if path_exists(&lib_file_path) {
                        file_content = fs::read_to_string(&lib_file_path).expect("could not read included file");
                        file_path = lib_file_path;
                    } else if path_exists(&path) {
                        file_content = fs::read_to_string(&path).expect("could not read included file");
                        file_path = path;
                    } else {
                        panic!("could not find file '{}' in lib directory or current working directory", path);
                    }

                    if included_files.iter().any(|n| n == &file_path) {
                        panic!("recursive include of '{}'", file_path);
                    }
                
                    let mut lexer = Lexer::new(file_content);
                    check_error(lexer.lex());
                    let mut parser = Parser::new(lexer.tokens, included_files.last().unwrap().clone());
                    parser.parse();

                    let mut included_files = included_files.clone();
                    included_files.push(file_path);

                    self.preprocess(parser.statements, included_files);
                },
                statement => {
                    self.statements.push(statement);
                }
            }
        }
    }

    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    } 
}
