// Crates
use unescape::unescape;

// Macros

macro_rules! name_range{() => {'a'..='z' | 'A'..='Z' | '-' | '_'}}

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
    input: String
}

impl Lexer {
    // Lexing Functions
    fn definition(&mut self) -> Token {
        let mut definition = String::new();
        self.index += 1;
        for c in self.input[self.index..].chars() {
            match c {
                name_range!() => {
                    definition.push(c);
                    self.index += 1;
                },
                _ => break
            }
        }

        DefinitionType::from(&definition)
    }

    fn directive(&mut self) -> Token {
        let mut directive = String::new();
        self.index += 1;
        for c in self.input[self.index..].chars() {
            match c {
                name_range!() => {
                    directive.push(c);
                    self.index += 1;
                },
                _ => break
            }
        }
        
        DirectiveType::from(&directive)
    }

    fn string(&mut self) -> Token {
        let mut string = String::new();
        self.index += 1;
        loop {
            let c = self.input.chars().nth(self.index).unwrap();
            match c {
                '\\' => {
                    string.push(c);
                    string.push(self.input.chars().nth(self.index + 1).unwrap());
                    self.index += 2;
                },
                '"' => {
                    self.index += 1;
                    break
                },
                '\n' => {
                    panic!("unexpected end of string");
                },
                _ => {
                    self.index += 1;
                    string.push(c);
                }
            }
        }

        Token::String(unescape(string.as_str()).unwrap())
    }

    fn setter(&mut self) -> Token {
        let mut setter = String::new();
        self.index += 1;
        for c in self.input[self.index..].chars() {
            match c {
                name_range!() => {
                    setter.push(c);
                    self.index += 1;
                },
                _ => break
            }
        }

        Token::Setter(setter)
    }

    fn number(&mut self) -> Token {
        let mut number = String::new();
        for c in self.input[self.index..].chars() {
            if ! c.is_digit(10) {
                break
            }
            number.push(c);
            self.index += 1;
        }

        match number.parse::<i32>() {
            Ok(num) => Token::Number(num),
            Err(e) => panic!("{e}")
        }
    }

    fn identifier(&mut self) -> Token {
        let mut identifier = String::new();
        for c in self.input[self.index..].chars() {
            match c {
                name_range!() => {
                    self.index += 1;
                    identifier.push(c);
                },
                _ => break
            }
        }

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
        self.index += 1;
        token
    }

    fn comment(&mut self) -> bool {
        for c in self.input[self.index..].chars() {
            self.index += 1;
            if c == '\n' {
                return true;
            }
        }

        return false;
    }

    // Pubs
    pub fn new(s: String) -> Self {
        Self {
            tokens: Vec::new(),
            index: 0,
            input: s, 
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
                        self.index += 1;
                        continue
                    },
                    '/' => {
                        if self.comment() {
                            continue
                        } else {
                            panic!("Unexpected token '/'");
                        }
                    },
                    _ => Token::Number(-1),
                };
                self.tokens.push(token);
            } else {
                break;
            }
        }
    }
}
