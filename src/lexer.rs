macro_rules! name_range{() => {'a'..='z' | 'A'..='Z' | '-' | '_'}}

// Tokens

#[derive(Debug)]
pub enum Definition {
    InlineProp,
    InlineArg,
    ChildProp,
    ChildArg,
    Object(String)
}

#[derive(Debug)]
pub enum Directive {
    Include,
    Header
}
#[derive(Debug, Clone, Copy)]
pub enum TypeIdentifier {
    String,
    Number,
    Bool
}

#[derive(Debug)]
pub enum Identifier {
    Generic(String),
    Type(TypeIdentifier)
}

#[derive(Debug)]
pub enum Token {
    String(String),             // "mystring"
    Number(i32),                // 0123456789
    Bool(i32),                  // true, false
    Definition(Definition),     // @mydefinition
    Directive(Directive),       // #mydirective
    Setter(String),             // .mysetter
    Identifier(Identifier),     // anything else
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
}

impl Definition {
    pub fn from(definition: &String) -> Token {
        Token::Definition(
            if definition == "InlineProp" {
                Definition::InlineProp
            } else if definition == "InlineArg" {
                Definition::InlineArg
            } else if definition == "ChildProp" {
                Definition::ChildProp
            } else if definition == "ChildArg" {
                Definition::ChildArg
            } else {
                Definition::Object(String::from(definition))
            }
        )
    }
}

impl Directive {
    pub fn from(directive: &String) -> Token {
        let directive_type: Option<Directive>;

        if directive == "include" {
            directive_type = Some(Directive::Include);
        } else if directive == "header" {
            directive_type = Some(Directive::Header);
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

pub struct Lexer<'a> {
    tokens: Vec<Token>,
    index: usize,
    input: &'a String
}

impl<'a> Lexer<'a> {
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

        Definition::from(&definition)
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
        
        Directive::from(&directive)
    }

    fn string(&mut self) -> Token {
        let mut string = String::new();
        for c in self.input[(self.index + 1)..].chars() {
            match c {
                '"' => {
                    self.index += 2;
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

        Token::String(string)
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
            "String" => Token::Identifier(Identifier::Type(TypeIdentifier::String)),
            "Number" => Token::Identifier(Identifier::Type(TypeIdentifier::Number)),
            "Bool"   => Token::Identifier(Identifier::Type(TypeIdentifier::Bool)),
            _        => Token::Identifier(Identifier::Generic(identifier))
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
    pub fn new(s: &'a String) -> Lexer<'a> {
        return Lexer {
            tokens: Vec::new(),
            index: 0,
            input: s, 
        }
    }
    pub fn lex(&mut self) -> &Vec<Token> {
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
        &self.tokens
    }
}
