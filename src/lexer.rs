// Crates

use unescape::unescape;
use log::{error};

// Tokens

#[derive(Debug, Clone)]
pub enum DefinitionType {
    InlineProp,
    InlineArg,
    ChildProp,
    ChildArg,
    Object(String)
}

#[derive(Debug, Clone)]
pub enum DirectiveType {
    Include,
    Header
}
#[derive(Debug, Clone)]
pub enum TypeIdentifierType {
    String,
    Number,
    Bool
}

#[derive(Debug, Clone)]
pub enum IdentifierType {
    Generic(String),
    Type(TypeIdentifierType)
}

#[derive(Debug, Clone)]
pub enum Token {
    String(String),             // "mystring"
    Number(i32),                // 0123456789
    Bool(i32),                  // true, false
    Definition(DefinitionType), // @mydefinition
    Directive(DirectiveType),   // #mydirective
    Setter(String),             // .mysetter
    Identifier(IdentifierType), // anything else
    StartBlock,                 // { 
    EndBlock,                   // }
    StartArgList,               // (
    EndArgList,                 // )
    ArgListDeliminator,         // ,
}

// Position

pub struct Position {
    line: i32,
    character: i32
}

impl Token {
    pub fn to_string(&self) -> &str {
        match self {
            Token::String(_) => "string",
            Token::Number(_) => "number",
            Token::Bool(_) => "boolean",
            Token::Definition(_) => "definition",
            Token::Directive(_) => "directive",
            Token::Setter(_) => "setter",
            Token::Identifier(_) => "identifier",
            Token::StartBlock => "{",
            Token::EndBlock => "}",
            Token::StartArgList => "(",
            Token::EndArgList => ")",
            Token::ArgListDeliminator => ","
        }
    }
    
    pub fn value_to_string(&self) -> String {
        match self {
            Token::String(string) => string.to_string(),
            Token::Number(number) => number.to_string(),
            Token::Bool(boolean) => boolean.to_string(),
            _ => todo!("not implemented yet, but not needed yet!")
        }
    }
}

impl DefinitionType {
    pub fn from(definition: &String) -> Token {
        Token::Definition(
            if definition == "InlineProp" {
                DefinitionType::InlineProp
            } else if definition == "InlineArg" {
                DefinitionType::InlineArg
            } else if definition == "ChildProp" {
                DefinitionType::ChildProp
            } else if definition == "ChildArg" {
                DefinitionType::ChildArg
            } else {
                DefinitionType::Object(String::from(definition))
            }
        )
    }
}

impl DefinitionType {
    pub fn to_string(&self) -> &str {
        match self {
            DefinitionType::InlineArg => "InlineArg",
            DefinitionType::InlineProp => "InlineProp",
            DefinitionType::ChildArg => "ChildArg",
            DefinitionType::ChildProp => "ChildProp",
            DefinitionType::Object(_) => "Object"
        }
    }
}

impl DirectiveType {
    pub fn from(directive: &String) -> Token {
        let directive_type: Option<DirectiveType>;

        if directive == "include" {
            directive_type = Some(DirectiveType::Include);
        } else if directive == "header" {
            directive_type = Some(DirectiveType::Header);
        } else {
            directive_type = None;
        }

        match directive_type {
            Some(t) => Token::Directive(t),
            None => panic!("invalid directive")
        }
    }
}

// Lexer

pub struct Lexer {
    pub tokens: Vec<Token>,
    index: usize,
    input: String,
    position: Position
}

impl Lexer {

    // Helper Functions

    #[inline]
    fn move_foward(&mut self) {
        self.index += 1;
        self.position.character += 1;
    }

    #[inline]
    fn move_forward_n(&mut self, n: usize) {
        self.index += n;
        self.position.character += n as i32;
    }

    #[inline]
    fn error(&self, message: &str) {
        error!("{message} (line {}, char {})", self.position.line, self.position.character)
    }

    // Lexing Functions

    fn definition(&mut self) -> Token {
        let mut definition = String::new();
        self.move_foward();
        for c in self.input[self.index..].chars() {
            match c {
                name_range!() => {
                    definition.push(c);
                },
                _ => break
            }
        }

        self.move_forward_n(definition.len());
        DefinitionType::from(&definition)
    }

    fn directive(&mut self) -> Token {
        let mut directive = String::new();
        self.move_foward();
        for c in self.input[self.index..].chars() {
            match c {
                name_range!() => {
                    directive.push(c);
                },
                _ => break
            }
        }

        self.move_forward_n(directive.len());
        DirectiveType::from(&directive)
    }

    fn string(&mut self) -> Token {
        let mut string = String::new();
        self.move_foward();
        loop {
            let c = self.input.chars().nth(self.index).unwrap();
            match c {
                '\\' => {
                    string.push(c);
                    string.push(self.input.chars().nth(self.index + 1).unwrap());
                    self.move_forward_n(2);
                },
                '"' => {
                    self.move_foward();
                    break
                },
                '\n' => {
                    self.error("unexpected end of string");
                    std::process::exit(1);
                },
                _ => {
                    self.move_foward();
                    string.push(c);
                }
            }
        }

        Token::String(unescape(string.as_str()).unwrap())
    }

    fn setter(&mut self) -> Token {
        let mut setter = String::new();
        self.move_foward();
        for c in self.input[self.index..].chars() {
            match c {
                name_range!() => {
                    setter.push(c);
                },
                _ => break
            }
        }

        self.move_forward_n(setter.len());
        Token::Setter(setter)
    }

    fn number(&mut self) -> Token {
        let mut number = String::new();
        for c in self.input[self.index..].chars() {
            if ! c.is_digit(10) {
                break
            }
            number.push(c);
        }

        self.move_forward_n(number.len());

        match number.parse::<i32>() {
            Ok(num) => Token::Number(num),
            Err(e) => {
                self.error(e.to_string().as_str());
                std::process::exit(1);
            }
        }
    }

    // TODO: Rename this function to include its use with parsing booleans
    fn identifier(&mut self) -> Token {
        let mut identifier = String::new();
        for c in self.input[self.index..].chars() {
            match c {
                name_range!() => {
                    identifier.push(c);
                },
                _ => break
            }
        }

        self.move_forward_n(identifier.len());

        match identifier.as_str() {
            "true"   => Token::Bool(1),
            "false"  => Token::Bool(0),
            "String" => Token::Identifier(IdentifierType::Type(TypeIdentifierType::String)),
            "Number" => Token::Identifier(IdentifierType::Type(TypeIdentifierType::Number)),
            "Bool"   => Token::Identifier(IdentifierType::Type(TypeIdentifierType::Bool)),
            _        => Token::Identifier(IdentifierType::Generic(identifier))
        }
    }

    fn add_and_move(&mut self, token: Token) -> Token {
        self.move_foward();
        token
    }

    fn comment(&mut self) {
        loop {
            let c = self.input.chars().nth(self.index);
            if let Some(c) = c {
                if c == '\n' {
                    break
                }
                self.move_foward();
            } else {
                break;
            }
        }
    }

    // Pubs
    pub fn new(s: String) -> Self {
        Self {
            tokens: Vec::new(),
            index: 0,
            input: s,
            position: Position { line: 0, character: 0 }
        }
    }
    pub fn lex(&mut self) {
        loop {
            let input_char = self.input.chars().nth(self.index);
            if let Some(c) = input_char {
                let token = match c {
                    '@'           => self.definition(),
                    '#'           => self.directive(),
                    '"'           => self.string(),
                    '.'           => self.setter(),
                    '0'..='9'     => self.number(),
                    name_range!() => self.identifier(),
                    '{'           => self.add_and_move(Token::StartBlock),
                    '}'           => self.add_and_move(Token::EndBlock),
                    ','           => self.add_and_move(Token::ArgListDeliminator),
                    '('           => self.add_and_move(Token::StartArgList),
                    ')'           => self.add_and_move(Token::EndArgList),
                    ' ' | '\n'    => {
                        self.move_foward();
                        continue
                    },
                    '/' => {
                        self.comment();
                        continue
                    },
                    _ => panic!("unrecognized character '{}'", c),
                };
                self.tokens.push(token);
            } else {
                break;
            }
        }
    }
}
