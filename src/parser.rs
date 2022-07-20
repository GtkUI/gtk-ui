use super::lexer::{
    Token,
    DefinitionType as TokenDefinitionType,
    DirectiveType as TokenDirectiveType,
    IdentifierType as TokenIdentifierType,
    TypeIdentifierType as TokenTypeIdentifierType
};

// Statement

#[derive(Debug)]
pub struct Property {
    internal_type: TokenTypeIdentifierType,
    name: String,
    definition_type: TokenDefinitionType
}

#[derive(Debug)]
pub enum DefinitionType {
    Raw,
    Collective,
    Root(String)
}

#[derive(Debug)]
pub struct Definition {
    name: String,
    children: Vec<Statement>,
    definition_type: DefinitionType
}

#[derive(Debug)]
pub struct Setter {
    name: String,
    value: Token
}

#[derive(Debug)]
pub struct Object {
    name: String,
    children: Vec<Statement>,
    arguments: Vec<Token>,
    setters: Vec<Setter>
}

#[derive(Debug)]
pub enum Statement {
    Property(Property),
    Definition(Definition),
    Object(Object),
    Header(String),
    Include(String)
}

impl Statement {
    pub fn to_string(&self) -> &str {
        match self {
            Statement::Property(_) => "Property",
            Statement::Definition(_) => "Definition",
            Statement::Object(_) => "Object",
            Statement::Header(_) => "Header",
            Statement::Include(_) => "Include"
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
        if let Token::StartBlock = token {
            self.index += 1;
            let mut statements = Vec::new();
            loop {
                let token = &self.tokens[self.index];
                if let Token::EndBlock = token {
                    self.index += 1;
                    break;
                }
                let statement = self.parse_statement();
                match statement {
                    Statement::Property(_) | Statement::Object(_) => statements.push(statement),
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
        if let Token::StartArgList = token {
            let mut args: Vec<Token> = Vec::new();
            loop {
                self.index += 1;
                let token = &self.tokens[self.index];
                match token {
                    Token::Number(_) | Token::String(_) | Token::Bool(_) => args.push(token.clone()),
                    Token::Identifier(identifier) => {
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
                match token {
                    Token::ArgListDeliminator => continue,
                    Token::EndArgList => break,
                    _ => panic!("found '{}', expected ','", token.to_string())
                }
            }
            self.index += 1;
            args
        } else {
            panic!("expected start of argument list, found {}", token.to_string());
        }
    }

    fn definition(&mut self, definition_type: TokenDefinitionType) -> Statement {
        self.index += 1;
        if let TokenDefinitionType::Object(name) = definition_type {
            let block = self.block();
            let definition_type = {
                if block.iter().all(|x| matches!(x, Statement::Property(_))) {
                    DefinitionType::Raw
                } else if block.iter().all(|x| matches!(x, Statement::Object(_))) {
                    if name == "root" {
                        DefinitionType::Root(self.filename.clone())
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

            Statement::Definition(definition)
        } else {
            let arglist = self.arglist();
            
            if arglist.len() != 2 {
                panic!("expected only 2 arguments, found {} args", arglist.len());
            }
            
            let name = &arglist[0];
            if let Token::String(name) = name {
                let internal_type = &arglist[1];
                if let Token::Identifier(TokenIdentifierType::Type(internal_type)) = internal_type {
                    let property = Property {
                        name: name.clone(),
                        internal_type: internal_type.clone(),
                        definition_type: definition_type.clone()
                    };
                    Statement::Property(property)
                } else {
                    panic!("expected type identifier, found {}", internal_type.to_string());
                }
            } else {
                panic!("expected String, found {}", name.to_string());
            }
        }
    }

    fn directive(&mut self, directive_type: TokenDirectiveType) -> Statement {
        self.index += 1;
        let directive_argument_token = &self.tokens[self.index];
        if let Token::String(arg) = directive_argument_token {
            self.index += 1;
            match directive_type {
                TokenDirectiveType::Header => {
                    Statement::Header(arg.clone())
                },
                TokenDirectiveType::Include => {
                    Statement::Include(arg.clone())
                }
            }
        } else {
            panic!("expected string, found {}", directive_argument_token.to_string());
        }
    }

    fn object(&mut self, identifier_type: TokenIdentifierType) -> Statement {
        if let TokenIdentifierType::Generic(name) = identifier_type {
            self.index += 1;
            let token = &self.tokens[self.index];
            let mut arguments = Vec::new();
            let mut children = Vec::new();
            let mut setters = Vec::new();
            
            match token {
                Token::StartArgList => {
                    arguments = self.arglist();
                    
                    let token = &self.tokens[self.index];
                    if let Token::StartBlock = token {
                        children = self.block();
                    }
                },
                Token::StartBlock => {
                    children = self.block();
                },
                _ => panic!("expected the start of an argument list or block, found '{}'", token.to_string())
            }

            loop {
                let token = &self.tokens[self.index];
                
                match token {
                    Token::Identifier(_) | Token::EndBlock => break,
                    Token::Setter(name) => {
                        self.index += 1;

                        let name = name.clone();
                        let args = self.arglist();
                        if args.len() != 1 {
                            panic!("expected 1 argument, got {}", args.len());
                        }

                        let value = &args[0];

                        match value {
                            Token::Number(_) | Token::String(_) | Token::Bool(_) => {
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

            Statement::Object(object)
        } else {
            panic!("expected generic identifier, found type identifier");
        }
    }

    fn parse_statement(&mut self) -> Statement {
        let token = &self.tokens[self.index];
        match token {
            Token::Definition(definition) => {
                let definition = definition.clone();
                self.definition(definition)
            },
            Token::Directive(directive) => {
                let directive = directive.clone();
                self.directive(directive)
            },
            Token::Identifier(identifier) => {
                let identifier = identifier.clone();
                self.object(identifier)
            },
            _ => panic!("unexpected {} {:?} {:?} {:?}", token.to_string(), self.tokens[self.index - 1], self.tokens[self.index - 2], self.tokens[self.index - 3])
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

    pub fn parse(&mut self) -> &Vec<Statement> {
        loop {
            if self.index >= self.tokens.len() {
                break
            }
            let statement = self.parse_statement();
            match statement {
                Statement::Definition(_) | Statement::Header(_) | Statement::Include(_) => self.statements.push(statement),
                _ => panic!("found {} on top level. Only object definitions and directives are allowed here.", statement.to_string()),
            }
        }
        &self.statements
    }
}
