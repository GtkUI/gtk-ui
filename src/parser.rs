use super::lexer::{
    Token,
    DefinitionType,
    DirectiveType,
    IdentifierType,
    TypeIdentifierType as TokenTypeIdentifierType
};

// Statement

#[derive(Debug)]
pub struct Property<'a> {
    internal_type: &'a TokenTypeIdentifierType,
    name: &'a String,
    definition_type: &'a DefinitionType
}

#[derive(Debug)]
pub struct Definition<'a> {
    name: &'a String,
    children: Vec<Statement<'a>>,
}

#[derive(Debug)]
pub struct Setter<'a> {
    name: &'a String,
    value: &'a Token
}

#[derive(Debug)]
pub struct Object<'a> {
    name: &'a String,
    children: Vec<Statement<'a>>,
    arguments: Vec<&'a Token>,
    setters: Vec<Setter<'a>>
}

#[derive(Debug)]
pub enum Statement<'a> {
    Property(Property<'a>),
    Definition(Definition<'a>),
    Object(Object<'a>),
    Header(&'a String),
    Include(&'a String)
}

impl<'a> Statement<'a> {
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

pub struct Parser<'a> {
    statements: Vec<Statement<'a>>,
    index: usize,
    tokens: &'a Vec<Token>
}

impl<'a> Parser<'a> {
    // Parsing Functions

    fn block(&mut self) -> Vec<Statement<'a>> {
        let token = &self.tokens[self.index];
        if let Token::StartBlock = token {
            let mut statements = Vec::new();
            loop {
                self.index += 1;
                let token = &self.tokens[self.index];
                if let Token::EndBlock = token {
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

    fn arglist(&mut self) -> Vec<&'a Token> {
        let token = &self.tokens[self.index];
        if let Token::StartArgList = token {
            let mut args = Vec::new();
            loop {
                self.index += 1;
                let token = &self.tokens[self.index];
                match token {
                    Token::Number(_) | Token::String(_) | Token::Bool(_) => args.push(token),
                    Token::Identifier(identifier) => {
                        if let IdentifierType::Type(_) = identifier {
                            args.push(token)
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
            args
        } else {
            panic!("expected start of argument list, found {}", token.to_string());
        }
    }

    fn definition(&mut self, definition_type: &'a DefinitionType) -> Statement<'a> {
        if let DefinitionType::Object(name) = definition_type {
            self.index += 1;
            let definition = Definition {
                name: &name,
                children: self.block()
            };
            self.index += 1;

            Statement::Definition(definition)
        } else {
            self.index += 1;
            let arglist = self.arglist();
            
            if arglist.len() != 2 {
                panic!("expected only 2 arguments, found {} args", arglist.len());
            }
            
            let name = &arglist[0];
            if let Token::String(name) = name {
                let internal_type = &arglist[1];
                if let Token::Identifier(IdentifierType::Type(internal_type)) = internal_type {
                    let property = Property {
                        name,
                        internal_type,
                        definition_type
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

    fn directive(&mut self, directive_type: &DirectiveType) -> Statement<'a> {
        self.index += 1;
        let directive_argument_token = &self.tokens[self.index];
        if let Token::String(arg) = directive_argument_token {
            self.index += 1;
            match directive_type {
                DirectiveType::Header => {
                    Statement::Header(arg)
                },
                DirectiveType::Include => {
                    Statement::Include(arg)
                }
            }
        } else {
            panic!("expected string, found {}", directive_argument_token.to_string());
        }
    }

    fn object(&mut self, identifier_type: &'a IdentifierType) -> Statement<'a> {
        if let IdentifierType::Generic(name) = identifier_type {
            self.index += 1;
            let token = &self.tokens[self.index];
            let mut arguments = Vec::new();
            let mut children = Vec::new();
            let mut setters = Vec::new();
            
            match token {
                Token::StartArgList => {
                    arguments = self.arglist();
                    self.index += 1;
                    
                    let token = &self.tokens[self.index];
                    if let Token::StartBlock = token {
                        children = self.block();
                        self.index += 1;
                    }
                },
                Token::StartBlock => {
                    children = self.block();
                    self.index += 1;
                },
                _ => panic!("expected the start of an argument list or block, found '{}'", token.to_string())
            }

            println!("{}", name);

            loop {
                let token = &self.tokens[self.index];
                
                match token {
                    Token::Identifier(_) | Token::EndBlock => {
                        self.index -= 1;
                        break;
                    },
                    Token::Setter(name) => {
                        self.index += 1;

                        let args = self.arglist();
                        if args.len() != 1 {
                            panic!("expected 1 argument, got {}", args.len());
                        }

                        let value = args[0];

                        match value {
                            Token::Number(_) | Token::String(_) | Token::Bool(_) => {
                                setters.push(Setter {
                                    name,
                                    value
                                })
                            },
                            _ => panic!("expected Number, String, or Bool, found {}", value.to_string())
                        }
                        self.index += 1;
                    },
                    _ => {
                        panic!("expected setter, found {}", token.to_string());
                    }
                }
            }
            
            let object = Object {
                arguments,
                name,
                children,
                setters
            };

            Statement::Object(object)
        } else {
            panic!("expected generic identifier, found type identifier");
        }
    }

    fn parse_statement(&mut self) -> Statement<'a> {
        let token = &self.tokens[self.index];
        match token {
            Token::Definition(definition) => self.definition(definition),
            Token::Directive(directive) => self.directive(directive),
            Token::Identifier(identifier) => self.object(identifier),
            _ => panic!("unexpected {} {:?} {:?} {:?}", token.to_string(), self.tokens[self.index - 1], self.tokens[self.index - 2], self.tokens[self.index - 3])
        }
    }

    // Pubs
    pub fn new(tokens: &Vec<Token>) -> Parser {
        return Parser {
            statements: Vec::new(),
            index: 0,
            tokens
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
