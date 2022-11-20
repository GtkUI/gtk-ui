use super::lexer::{
    Lexer,
};
use super::parser::{
    Parser,
    Statement,
    StatementValue
};
use super::util::{check_error, get_include_path};
use std::ops::Range;
use std::fs;


// TODO: Warn about a file being included multiple times.
// At the moment, regardless of where the library is included, if it is included twice, everything defined in the library is redefined
// This is of course very easily avoidable, but for user friendliness, a notification about this would be nice

pub struct Preprocessor {
    pub statements: Vec<Statement>,
}

impl Preprocessor { 

    // Pubs
    pub fn preprocess(&mut self, input: Vec<Statement>, included_files: Vec<String>) -> Result<(), (String, Range<usize>)> {
        for statement in input {
            match statement.value {
                StatementValue::Include(path) => {
                    if let Some(path) = get_include_path(&path) {
                        match fs::read_to_string(&path) {
                            Ok(content) => {
                                if included_files.iter().any(|n| n == &path) {
                                    return Err((format!("recursive include of '{}'", path), (1..0)));
                                }
                            
                                let mut lexer = Lexer::new(content.clone());
                                check_error(lexer.lex(false), &path, &content);
                                let mut parser = Parser::new(lexer.tokens, included_files.last().unwrap().clone());
                                check_error(parser.parse(), &path, &content);

                                let mut included_files = included_files.clone();
                                included_files.push(path);

                                if let Err(err) = self.preprocess(parser.statements, included_files) {
                                    return Err(err);
                                }
                            },
                            Err(err) => return Err((String::from(err.to_string()), 1..0))
                        }
                    } else {
                        return Err((format!("could not find file '{}' in lib directory or current working directory", path), (1..0)));
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
