use super::lexer::{
    Token,
    TokenValue,
    Position,
    DefinitionType as TokenDefinitionType,
    DirectiveType as TokenDirectiveType,
    IdentifierType as TokenIdentifierType,
    TypeIdentifierType as TokenTypeIdentifierType
};
use std::path::Path;

// Statement

#[derive(Debug)]
pub struct Property {
    pub internal_type: TokenTypeIdentifierType,
    pub name: String,
    pub definition_type: TokenDefinitionType
}

#[derive(Debug)]
pub enum DefinitionType {
    Raw,
    Collective,
    Root(String)
}

#[derive(Debug)]
pub struct Definition {
    pub name: String,
    pub children: Vec<Statement>,
    pub definition_type: DefinitionType
}

#[derive(Debug)]
pub struct Setter {
    pub name: String,
    pub value: Token
}

#[derive(Debug)]
pub struct Object {
    pub name: String,
    pub children: Vec<Statement>,
    pub arguments: Vec<Token>,
    pub setters: Vec<Setter>
}

#[derive(Debug)]
pub enum StatementValue {
    Property(Property),
    Definition(Definition),
    Object(Object),
    Header(String),
    Include(String)
}

#[derive(Debug)]
pub struct Statement {
    pub value: StatementValue,
    pub position: Position
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

    fn block(&mut self) -> Result<(Vec<Statement>, Position), (String, Position)> {
        let token = &self.tokens[self.index];
        let position = token.position.clone();
        if let TokenValue::StartBlock = token.value {
            self.index += 1;
            let mut statements = Vec::new();
            loop {
                let token = &self.tokens[self.index];
                if let TokenValue::EndBlock = token.value {
                    self.index += 1;
                    break;
                }
                match self.parse_statement() {
                    Ok(statement) => {
                        match &statement.value {
                            StatementValue::Property(_) | StatementValue::Object(_) => statements.push(statement),
                            _ => return Err((format!("found {} inside block. Only properties and objects are allowed here.", statement.to_string()), statement.position)),
                        }
                    },
                    Err(err) => return Err(err)
                }
            }
            Ok(( statements, position ))
        } else {
            Err((format!("expected the start of a block, got {}", token.to_string()), token.position))
        }
    }

    fn arglist(&mut self) -> Result<(Vec<Token>, Position), (String, Position)> {
        let token = &self.tokens[self.index];
        let position = token.position.clone();
        if let TokenValue::StartArgList = token.value {
            let mut args: Vec<Token> = Vec::new();
            loop {
                self.index += 1;
                let token = &self.tokens[self.index];
                match &token.value {
                    TokenValue::Number(_) | TokenValue::String(_) | TokenValue::Bool(_) => args.push(token.clone()),
                    TokenValue::Identifier(identifier) => {
                        if let TokenIdentifierType::Type(_) = identifier {
                            args.push(token.clone())
                        } else {
                            return Err((format!("found generic identifier, expected Number, String, Bool, or type identifier"), token.position));
                        }
                    },
                    _ => return Err((format!("found {}, expected Number, String, Bool, or type identifier", token.to_string()), token.position))
                }

                self.index += 1;
                let token = &self.tokens[self.index];
                match token.value {
                    TokenValue::ArgListDeliminator => continue,
                    TokenValue::EndArgList => break,
                    _ => return Err((format!("found '{}', expected ','", token.to_string()), token.position))
                }
            }
            self.index += 1;
            Ok(( args, position ))
        } else {
            Err((format!("expected start of argument list, found {}", token.to_string()), token.position))
        }
    }

    fn definition(&mut self, definition_type: TokenDefinitionType, position: Position) -> Result<Statement, (String, Position)> {
        self.index += 1;
        if let TokenDefinitionType::Object(name) = definition_type {
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
                        definition_type
                    };

                    Ok(Statement {
                        value: StatementValue::Definition(definition),
                        position: position.clone()
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
                                position: position.clone()
                            })
                        } else {
                            return Err((format!("expected type identifier, found {}", internal_type.to_string()), internal_type.position));
                        }
                    } else {
                        return Err((format!("expected String, found {}", name.to_string()), name.position));
                    }
                },
                Err(err) => return Err(err)
            }
        }
    }

    fn directive(&mut self, directive_type: TokenDirectiveType, position: Position) -> Result<Statement, (String, Position)> {
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
                position: position.clone()
            })
        } else {
            Err((format!("expected string, found {}", directive_argument_token.to_string()), directive_argument_token.position))
        }
    }

    fn object(&mut self, identifier_type: TokenIdentifierType, position: Position) -> Result<Statement, (String, Position)> {
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
                _ => return Err((format!("expected the start of an argument list or block, found '{}'", token.to_string()), token.position))
            }

            loop {
                let token = &self.tokens[self.index];
                
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
                                            value: value.clone()
                                        })
                                    },
                                    _ => return Err((format!("expected Number, String, or Bool, found {}", value.to_string()), value.position))
                                }
                            },
                            Err(err) => return Err(err)
                        }
                    },
                    _ => {
                        return Err((format!("expected setter, found {}", token.to_string()), token.position));
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
                position: position.clone()
            })
        } else {
            Err((format!("expected generic identifier, found type identifier"), position))
        }
    }

    fn parse_statement(&mut self) -> Result<Statement, (String, Position)> {
        let token = &self.tokens[self.index];
        match &token.value {
            TokenValue::Definition(definition) => {
                let definition = definition.clone();
                self.definition(definition, token.position)
            },
            TokenValue::Directive(directive) => {
                let directive = directive.clone();
                self.directive(directive, token.position)
            },
            TokenValue::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.object(identifier, token.position)
            },
            _ => Err(( format!("unexpected {}", token.to_string()), token.position ))
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

    pub fn parse(&mut self) -> Result<(), (String, Position)> {
        loop {
            if self.index >= self.tokens.len() {
                break Ok(())
            }
            match self.parse_statement() {
                Ok(statement) => {
                    match &statement.value {
                        StatementValue::Definition(_) | StatementValue::Header(_) | StatementValue::Include(_) => self.statements.push(statement),
                        _ => return Err(( format!("found {} on top level. Only object definitions and directives are allowed here.", statement.to_string()), statement.position )),
                    }
                },
                Err(err) => return Err(err)
            }
        }
    }
}
