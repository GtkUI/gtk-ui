use super::lexer::{
    Token,
    TokenValue,
    DefinitionType as TokenDefinitionType,
    DirectiveType as TokenDirectiveType,
    IdentifierType as TokenIdentifierType,
    TypeIdentifierType as TokenTypeIdentifierType
};
use std::path::Path;
use std::ops::Range;

// Statement

#[derive(Debug, Clone)]
pub struct Property {
    pub internal_type: TokenTypeIdentifierType,
    pub name: String,
    pub definition_type: TokenDefinitionType
}

#[derive(Debug, Clone)]
pub enum DefinitionType {
    Raw,
    Collective,
    Root(String)
}

#[derive(Debug, Clone)]
pub struct Definition {
    pub name: String,
    pub children: Vec<Statement>,
    pub inherits: Vec<String>,
    pub definition_type: DefinitionType
}

#[derive(Debug, Clone)]
pub struct Setter {
    pub name: String,
    pub value: Token,
    pub range: Range<usize>
}

#[derive(Debug, Clone)]
pub struct Object {
    pub name: String,
    pub children: Vec<Statement>,
    pub arguments: Vec<Token>,
    pub setters: Vec<Setter>
}

#[derive(Debug, Clone)]
pub enum StatementValue {
    Property(Property),
    Definition(Definition),
    Object(Object),
    Header(String),
    Include(String)
}

#[derive(Debug, Clone)]
pub struct Statement {
    pub value: StatementValue,
    pub range: Range<usize>
}

impl Statement {
    pub fn to_string(&self) -> &str {
        match &self.value {
            StatementValue::Property(_) => "Property",
            StatementValue::Definition(_) => "Definition",
            StatementValue::Object(_) => "Object",
            StatementValue::Header(_) => "Header",
            StatementValue::Include(_) => "Include"
        }
    }
}

// Parser

pub struct Parser {
    pub statements: Vec<Statement>,
    index: usize,
    tokens: Vec<Token>,
    filename: String
}

impl Parser {
    // Parsing Functions

    fn block(&mut self) -> Result<(Vec<Statement>, Range<usize>), (String, Range<usize>)> {
        let token = &self.tokens[self.index].clone();
        if let TokenValue::StartBlock = token.value {
            self.index += 1;
            let mut statements = Vec::new();
            loop {
                let token = &self.tokens[self.index].clone();
                if let TokenValue::EndBlock = token.value {
                    self.index += 1;
                    break;
                }
                match self.parse_statement() {
                    Some(result) => {
                        match result {
                            Ok(statement) => {
                                match &statement.value {
                                    StatementValue::Property(_) | StatementValue::Object(_) => statements.push(statement),
                                    _ => return Err((format!("found {} inside block. Only properties and objects are allowed here.", statement.to_string()), statement.range)),
                                }
                            },
                            Err(err) => return Err(err)
                        }
                    }
                    None => continue
                }
            }
            Ok(( statements, token.range.clone() ))
        } else {
            Err((format!("expected the start of a block, got {}", token.to_string()), token.range.clone()))
        }
    }

    fn arglist(&mut self) -> Result<(Vec<Token>, Range<usize>), (String, Range<usize>)> {
        let token = &self.tokens[self.index];
        if let TokenValue::StartArgList = token.value {
            let mut args: Vec<Token> = Vec::new();
            loop {
                self.index += 1;
                let token = &self.tokens[self.index];
                match &token.value {
                    TokenValue::Number(_) | TokenValue::String(_) | TokenValue::Bool(_) => args.push(token.clone()),
                    TokenValue::Identifier(_identifier) => {
                        // if let TokenIdentifierType::Type(_) = identifier {
                        //     args.push(token.clone())
                        // } else {
                        //     return Err((format!("found generic identifier, expected Number, String, Bool, or type identifier"), token.position));
                        // }
                        args.push(token.clone())
                    },
                    _ => return Err((format!("found {}, expected Number, String, Bool, or type identifier", token.to_string()), token.range.clone()))
                }

                self.index += 1;
                let token = &self.tokens[self.index];
                match token.value {
                    TokenValue::ArgListDeliminator => continue,
                    TokenValue::EndArgList => break,
                    _ => return Err((format!("found '{}', expected ','", token.to_string()), token.range.clone()))
                }
            }
            self.index += 1;
            Ok(( args, token.range.clone() ))
        } else {
            Err((format!("expected start of argument list, found {}", token.to_string()), token.range.clone()))
        }
    }

    fn definition(&mut self, definition_type: TokenDefinitionType, range: Range<usize>) -> Result<Statement, (String, Range<usize>)> {
        self.index += 1;
        if let TokenDefinitionType::Object(name) = definition_type {
            let token = &self.tokens[self.index];
            let mut inherits: Vec<String> = Vec::new();
            match &token.value {
                TokenValue::StartBlock => (),
                TokenValue::Inherits => {
                    self.index += 1;
                    let token = &self.tokens[self.index];
                    
                    match &token.value {
                        TokenValue::StartArgList => {
                            match self.arglist() {
                                Ok(arglist) => {
                                    for token in arglist.0 {
                                        if let TokenValue::Identifier(TokenIdentifierType::Generic(parent)) = &token.value {
                                            inherits.push(parent.clone());
                                        } else {
                                            return Err((String::from("argument list of parents must only contain definitions"), arglist.1.clone()));
                                        }
                                    }
                                },
                                Err(err) => {
                                    return Err(err);
                                }
                            }
                        },
                        TokenValue::Identifier(TokenIdentifierType::Generic(parent)) => {
                            inherits.push(parent.clone());
                            self.index += 1;
                        },
                        _ => return Err((format!("expected an argument list or definition identifier, found {}", token.to_string()), token.range.clone()))
                    }
                },
                _ => return Err((format!("expected a '->' or '{{', found '{}'", token.to_string()), token.range.clone()))
            }
            match self.block() {
                Ok(block) => {
                    let definition_type = {
                        if block.0.iter().all(|x| matches!(&x.value, StatementValue::Property(_))) {
                            DefinitionType::Raw
                        } else if block.0.iter().all(|x| matches!(&x.value, StatementValue::Object(_))) {
                            if name == "root" {
                                let path = Path::new(&self.filename);
                                DefinitionType::Root(path.file_stem().expect("invalid file path").to_str().expect("failed to unwrap file path string").to_string())
                            } else {
                                DefinitionType::Collective
                            }
                        } else {
                            return Err((String::from("a definition can only have all property definitions or all objects"), block.1));
                        }
                    };

                    let definition = Definition {
                        name: name.to_string(),
                        children: block.0,
                        definition_type,
                        inherits
                    };

                    Ok(Statement {
                        value: StatementValue::Definition(definition),
                        range: range.clone()
                    })
                },
                Err(err) => return Err(err)
            }
        } else {
            match self.arglist() {
                Ok(arglist) => {
                    if arglist.0.len() != 2 {
                        return Err((format!("expected only 2 arguments, found {} args", arglist.0.len()), arglist.1));
                    }
                    
                    let name = &arglist.0[0];
                    if let TokenValue::String(name) = &name.value {
                        let internal_type = &arglist.0[1];
                        if let TokenValue::Identifier(TokenIdentifierType::Type(internal_type)) = &internal_type.value {
                            let property = Property {
                                name: name.clone(),
                                internal_type: internal_type.clone(),
                                definition_type: definition_type.clone()
                            };
                            Ok(Statement {
                                value: StatementValue::Property(property),
                                range: range.clone()
                            })
                        } else {
                            return Err((format!("expected type identifier, found {}", internal_type.to_string()), internal_type.range.clone()));
                        }
                    } else {
                        return Err((format!("expected String, found {}", name.to_string()), name.range.clone()));
                    }
                },
                Err(err) => return Err(err)
            }
        }
    }

    fn directive(&mut self, directive_type: TokenDirectiveType, range: Range<usize>) -> Result<Statement, (String, Range<usize>)> {
        self.index += 1;
        let directive_argument_token = &self.tokens[self.index];
        if let TokenValue::String(arg) = &directive_argument_token.value {
            self.index += 1;
            let value = match directive_type {
                TokenDirectiveType::Header => {
                    StatementValue::Header(arg.clone())
                },
                TokenDirectiveType::Include => {
                    StatementValue::Include(arg.clone())
                }
            };
            Ok(Statement {
                value,
                range: range.clone()
            })
        } else {
            Err((format!("expected string, found {}", directive_argument_token.to_string()), directive_argument_token.range.clone()))
        }
    }

    fn object(&mut self, identifier_type: TokenIdentifierType, range: Range<usize>) -> Result<Statement, (String, Range<usize>)> {
        if let TokenIdentifierType::Generic(name) = identifier_type {
            self.index += 1;
            let token = &self.tokens[self.index];
            let mut arguments = Vec::new();
            let mut children = Vec::new();
            let mut setters = Vec::new();
            
            match token.value {
                TokenValue::StartArgList => {
                    match self.arglist() {
                        Ok(args) => {
                            arguments = args.0;
                            let token = &self.tokens[self.index];
                            if let TokenValue::StartBlock = token.value {
                                match self.block() {
                                    Ok(c) => children = c.0,
                                    Err(e) => return Err(e)
                                }
                            }
                        },
                        Err(err) => return Err(err)
                    }
                },
                TokenValue::StartBlock => {
                    match self.block() {
                        Ok(c) => children = c.0,
                        Err(e) => return Err(e)
                    }
                },
                _ => return Err((format!("expected the start of an argument list or block, found '{}'", token.to_string()), token.range.clone()))
            }

            loop {
                let token = self.tokens[self.index].clone();
                
                match &token.value {
                    TokenValue::Identifier(_) | TokenValue::EndBlock => break,
                    TokenValue::Setter(name) => {
                        self.index += 1;

                        let name = name.clone();
                        match self.arglist() {
                            Ok(args) => {
                                if args.0.len() != 1 {
                                    return Err((format!("expected 1 argument, got {}", args.0.len()), args.1));
                                }

                                let value = &args.0[0];

                                match value.value {
                                    TokenValue::Number(_) | TokenValue::String(_) | TokenValue::Bool(_) => {
                                        setters.push(Setter {
                                            name: name,
                                            value: value.clone(),
                                            range: token.range
                                        })
                                    },
                                    _ => return Err((format!("expected Number, String, or Bool, found {}", value.to_string()), value.range.clone()))
                                }
                            },
                            Err(err) => return Err(err)
                        }
                    },
                    _ => {
                        return Err((format!("expected setter, found {}", token.to_string()), token.range));
                    }
                }
            }
            
            let object = Object {
                arguments,
                name: name.clone(),
                children,
                setters
            };

            Ok(Statement {
                value: StatementValue::Object(object),
                range: range.clone()
            })
        } else {
            Err((format!("expected generic identifier, found type identifier"), range))
        }
    }

    fn parse_statement(&mut self) -> Option<Result<Statement, (String, Range<usize>)>> {
        let token = &self.tokens[self.index];
        match &token.value {
            TokenValue::Definition(definition) => {
                let definition = definition.clone();
                Some(self.definition(definition, token.range.clone()))
            },
            TokenValue::Directive(directive) => {
                let directive = directive.clone();
                Some(self.directive(directive, token.range.clone()))
            },
            TokenValue::Identifier(identifier) => {
                let identifier = identifier.clone();
                Some(self.object(identifier, token.range.clone()))
            },
            TokenValue::Comment => None,
            _ => Some(Err(( format!("unexpected {}", token.to_string()), token.range.clone() )))
        }
    }

    // Pubs
    pub fn new(tokens: Vec<Token>, filename: String) -> Parser {
        return Parser {
            statements: Vec::new(),
            index: 0,
            tokens,
            filename
        }
    }

    pub fn parse(&mut self) -> Result<(), (String, Range<usize>)> {
        loop {
            if self.index >= self.tokens.len() {
                break Ok(())
            }
            match self.parse_statement() {
                Some(result) => {
                    match result {
                        Ok(statement) => {
                            
                            match &statement.value {
                                StatementValue::Definition(_) | StatementValue::Header(_) | StatementValue::Include(_) => self.statements.push(statement),
                                _ => return Err(( format!("found {} on top level. Only object definitions and directives are allowed here.", statement.to_string()), statement.range )),
                            }
                        },
                        Err(err) => return Err(err)
                    }
                },
                None => continue
            }
        }
    }
}
