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

    fn block(&mut self) -> Vec<Statement> {
        let token = &self.tokens[self.index];
        if let TokenValue::StartBlock = token.value {
            self.index += 1;
            let mut statements = Vec::new();
            loop {
                let token = &self.tokens[self.index];
                if let TokenValue::EndBlock = token.value {
                    self.index += 1;
                    break;
                }
                let statement = self.parse_statement();
                match &statement.value {
                    StatementValue::Property(_) | StatementValue::Object(_) => statements.push(statement),
                    _ => panic!("found {} inside block. Only properties and objects are allowed here.", statement.to_string()),
                }
            }
            statements
        } else {
            panic!("expected the start of a block, got {}", token.to_string());
        }
    }

    fn arglist(&mut self) -> Vec<Token> {
        let token = &self.tokens[self.index];
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
                            panic!("found generic identifier, expected Number, String, Bool, or type identifier");
                        }
                    },
                    _ => panic!("found {}, expected Number, String, Bool, or type identifier", token.to_string())
                }

                self.index += 1;
                let token = &self.tokens[self.index];
                match token.value {
                    TokenValue::ArgListDeliminator => continue,
                    TokenValue::EndArgList => break,
                    _ => panic!("found '{}', expected ','", token.to_string())
                }
            }
            self.index += 1;
            args
        } else {
            panic!("expected start of argument list, found {}", token.to_string());
        }
    }

    fn definition(&mut self, definition_type: TokenDefinitionType, position: Position) -> Statement {
        self.index += 1;
        if let TokenDefinitionType::Object(name) = definition_type {
            let block = self.block();
            let definition_type = {
                if block.iter().all(|x| matches!(&x.value, StatementValue::Property(_))) {
                    DefinitionType::Raw
                } else if block.iter().all(|x| matches!(&x.value, StatementValue::Object(_))) {
                    if name == "root" {
                        let path = Path::new(&self.filename);
                        DefinitionType::Root(path.file_stem().expect("invalid file path").to_str().expect("failed to unwrap file path string").to_string())
                    } else {
                        DefinitionType::Collective
                    }
                } else {
                    panic!("a definition can only have all property definitions or all objects");
                }
            };

            let definition = Definition {
                name: name.to_string(),
                children: block,
                definition_type
            };

            Statement {
                value: StatementValue::Definition(definition),
                position: position.clone()
            }
        } else {
            let arglist = self.arglist();
            
            if arglist.len() != 2 {
                panic!("expected only 2 arguments, found {} args", arglist.len());
            }
            
            let name = &arglist[0];
            if let TokenValue::String(name) = &name.value {
                let internal_type = &arglist[1];
                if let TokenValue::Identifier(TokenIdentifierType::Type(internal_type)) = &internal_type.value {
                    let property = Property {
                        name: name.clone(),
                        internal_type: internal_type.clone(),
                        definition_type: definition_type.clone()
                    };
                    Statement {
                        value: StatementValue::Property(property),
                        position: position.clone()
                    }
                } else {
                    panic!("expected type identifier, found {}", internal_type.to_string());
                }
            } else {
                panic!("expected String, found {}", name.to_string());
            }
        }
    }

    fn directive(&mut self, directive_type: TokenDirectiveType, position: Position) -> Statement {
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
            Statement {
                value,
                position: position.clone()
            }
        } else {
            panic!("expected string, found {}", directive_argument_token.to_string());
        }
    }

    fn object(&mut self, identifier_type: TokenIdentifierType, position: Position) -> Statement {
        if let TokenIdentifierType::Generic(name) = identifier_type {
            self.index += 1;
            let token = &self.tokens[self.index];
            let mut arguments = Vec::new();
            let mut children = Vec::new();
            let mut setters = Vec::new();
            
            match token.value {
                TokenValue::StartArgList => {
                    arguments = self.arglist();
                    
                    let token = &self.tokens[self.index];
                    if let TokenValue::StartBlock = token.value {
                        children = self.block();
                    }
                },
                TokenValue::StartBlock => {
                    children = self.block();
                },
                _ => panic!("expected the start of an argument list or block, found '{}'", token.to_string())
            }

            loop {
                let token = &self.tokens[self.index];
                
                match &token.value {
                    TokenValue::Identifier(_) | TokenValue::EndBlock => break,
                    TokenValue::Setter(name) => {
                        self.index += 1;

                        let name = name.clone();
                        let args = self.arglist();
                        if args.len() != 1 {
                            panic!("expected 1 argument, got {}", args.len());
                        }

                        let value = &args[0];

                        match value.value {
                            TokenValue::Number(_) | TokenValue::String(_) | TokenValue::Bool(_) => {
                                setters.push(Setter {
                                    name: name,
                                    value: value.clone()
                                })
                            },
                            _ => panic!("expected Number, String, or Bool, found {}", value.to_string())
                        }
                    },
                    _ => {
                        panic!("expected setter, found {}", token.to_string());
                    }
                }
            }
            
            let object = Object {
                arguments,
                name: name.clone(),
                children,
                setters
            };

            Statement {
                value: StatementValue::Object(object),
                position: position.clone()
            }
        } else {
            panic!("expected generic identifier, found type identifier");
        }
    }

    fn parse_statement(&mut self) -> Statement {
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
            _ => panic!("unexpected {}", token.to_string())
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

    pub fn parse(&mut self) {
        loop {
            if self.index >= self.tokens.len() {
                break
            }
            let statement = self.parse_statement();
            match &statement.value {
                StatementValue::Definition(_) | StatementValue::Header(_) | StatementValue::Include(_) => self.statements.push(statement),
                _ => panic!("found {} on top level. Only object definitions and directives are allowed here.", statement.to_string()),
            }
        }
    }
}
