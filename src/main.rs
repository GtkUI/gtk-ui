use std::fs;
use std::env;
use std::process;

#[derive(Debug)]
enum Definition {
    InlineProp,
    InlineArg,
    ChildProp,
    ChildArg,
    Class,
    Object(String)
}

#[derive(Debug)]
enum Directive {
    Include,
    Header
}

#[derive(Debug)]
enum Token {
    String(String),             // "..."
    Number(i32),                // 0123456789
    Definition(Definition),           // @...
    Directive(Directive),       // #...
    StartBlock,                 // { 
    EndBlock,                   // }
    StartArgList,               // (
    EndArgList,                 // )
    ArgListDeliminator,         // ,
}

fn string_to_definition(definition: &str) -> Token {
    Token::Definition(
        if definition == "InlineProp" {
            Definition::InlineProp
        } else if definition == "InlineArg" {
            Definition::InlineArg
        } else if definition == "ChildProp" {
            Definition::ChildProp
        } else if definition == "ChildArg" {
            Definition::ChildArg
        } else if definition == "Class" {
            Definition::Class
        } else {
            Definition::Object(String::from(definition))
        }
    )
}

fn string_to_directive(directive: &str) -> Result<Token, &'static str> {
    let directive_type: Option<Directive>;

    if directive == "include" {
        directive_type = Some(Directive::Include);
    } else if directive == "header" {
        directive_type = Some(Directive::Header);
    } else {
        directive_type = None;
    }

    match directive_type {
        Some(t) => Ok(Token::Directive(t)),
        None => Err("invalid directive")
    }
}

fn lex_definition(input: &String, index: &mut usize) -> Token {
    let mut definition = String::new();
    *index += 1;
    for c in input[*index..].chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '-' | '_' => {
                definition.push(c);
                *index += 1;
            },
            _ => break
        }
    }

    string_to_definition(&definition)
}

fn lex_directive(input: &String, index: &mut usize) -> Token {
    let mut directive = String::new();
    *index += 1;
    for c in input[*index..].chars() {
        match c {
            'a'..='z' | 'A'..='Z' | '-' | '_' => {
                directive.push(c);
                *index += 1;
            },
            _ => break
        }
    }

    let parsed_directive = string_to_directive(&directive);
    
    match parsed_directive {
        Ok(k) => k,
        Err(m) => panic!("{m} '{directive}'")
    }
}

fn lex_string(input: &String, index: &mut usize) -> Token {
    let mut string = String::new();
    for c in input[(*index + 1)..].chars() {
        match c {
            '"' => {
                *index += 2;
                break
            },
            '\n' => {
                panic!("unexpected end of string");
            },
            _ => {
                *index += 1;
                string.push(c);
            }
        }
    }

    Token::String(string)
}

fn lex_number(input: &String, index: &mut usize) -> Token {
    let mut number = String::new();
    for c in input[*index..].chars() {
        if ! c.is_digit(10) {
            break
        }
        number.push(c);
        *index += 1;
    }

    match number.parse::<i32>() {
        Ok(num) => Token::Number(num),
        Err(e) => panic!("{e}")
    }
}

fn process_comment(input: &String, index: &mut usize) -> bool {
    for c in input[*index..].chars() {
        *index += 1;
        if c == '\n' {
            return true;
        }
    }

    return false;
}

fn add_and_move(token: Token, index: &mut usize) -> Token {
    *index += 1;
    token
}

fn lex(input: &String) -> Vec<Token> {
    let mut tokens: Vec<Token> = Vec::new();
    let mut index = 0;
    loop {
        let input_char = input.chars().nth(index);
        if let Some(c) = input_char {
            tokens.push(match c {
                '@' => lex_definition(&input, &mut index),
                '#' => lex_directive(&input, &mut index),
                '"' => lex_string(&input, &mut index),
                '0'..='9' => lex_number(&input, &mut index),
                '/' => {
                    if process_comment(&input, &mut index) {
                        continue
                    } else {
                        panic!("Unexpected token '/'");
                    }
                },
                '{' => add_and_move(Token::StartBlock, &mut index),
                '}' => add_and_move(Token::EndBlock, &mut index),
                ',' => add_and_move(Token::ArgListDeliminator, &mut index),
                '(' => add_and_move(Token::StartArgList, &mut index),
                ')' => add_and_move(Token::EndArgList, &mut index),
                ' ' | '\n' => {
                    index += 1;
                    continue
                },
                _ => Token::Number(-1),
            })
        } else {
            break;
        }
    }
    tokens
}

fn print_help() {
    println!("Usage: gtk-ui [FILENAME]");
}

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() == 1 {
        print_help();
        process::exit(0);
    }

    let filename = &args[1];
    let file_content = fs::read_to_string(filename)
        .expect("Something went wrong while trying to read the file");
    
    let tokens = lex(&file_content);
    let mut unparsed_chars = 0;
    for token in tokens {
        if let Token::Number(n) = token {
            if n == -1 {
                unparsed_chars += 1;
                continue
            }
        }
        println!("{:?}", token);
    }
    println!("Unparsed Chars: {}/{} ({:.3}%)", unparsed_chars, file_content.len(), (unparsed_chars as f32)/(file_content.len() as f32)*100.0);
}
