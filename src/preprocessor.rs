use super::lexer::{
    Lexer,
    Position
};
use super::parser::{
    Parser,
    Statement,
    StatementValue
};
use super::util::check_error;
use std::fs;

const LIB_PATH: &str =  "/usr/share/gtkui/";

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
    pub fn preprocess(&mut self, input: Vec<Statement>, included_files: Vec<String>) -> Result<(), (String, Position)> {
        for statement in input {
            match statement.value {
                StatementValue::Include(path) => {
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
                        return Err((format!("could not find file '{}' in lib directory or current working directory", path), Position { character: -1, line: -1 }));
                    }

                    if included_files.iter().any(|n| n == &file_path) {
                        return Err((format!("recursive include of '{}'", file_path), Position { character: -1, line: -1 }));
                    }
                
                    let mut lexer = Lexer::new(file_content);
                    check_error(lexer.lex());
                    let mut parser = Parser::new(lexer.tokens, included_files.last().unwrap().clone());
                    check_error(parser.parse());

                    let mut included_files = included_files.clone();
                    included_files.push(file_path);

                    if let Err(err) = self.preprocess(parser.statements, included_files) {
                        return Err(err);
                    }
                },
                _ => {
                    self.statements.push(statement);
                }
            }
        }
        Ok(())
    }

    pub fn new() -> Self {
        Self {
            statements: Vec::new(),
        }
    } 
}
