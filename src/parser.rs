use super::lexer::{lex, Token, Definition as LexDefinition, Directive as LexDirective, Identifier as LexIdentifier, TypeIdentifier as LexTypeIdentifier, token_to_string};
use std::path::Path;
use std::fs;

const LIB_DIR: &str = "./examples/libs";

pub enum XMLPlacementType {
    Inline,
    Child
}

pub enum DefinitionType {
    Prop,
    Arg
}

// NOTE: there are no booleans because true and false are implicitly converted to their numberic counterparts (1, 0)
pub enum InternalValue {
    String(String),
    Number(i32),
}

pub enum Argument {
    String(String),
    Number(i32),
    Type(LexTypeIdentifier)
}

pub struct Property {
    xml_placement_type: XMLPlacementType,
    definition_type: DefinitionType,
    internal_value_type: LexTypeIdentifier,
    name: String,
    value: Option<InternalValue>,
}

pub struct Object {
    children: Vec<Object>,
    properties: Vec<Property>,
    definition_name: String
}

pub enum Definition {
    Collective(Vec<Object>),  // Defined as a collection of other Objects
    Raw(Vec<Property>),       // Defined as a collection of Args and Props which get translated into XML
}

pub struct ParsingResult {
    pub definitions: Vec<Definition>,
    pub headers: Vec<String>,
}

fn parse_include(tokens: &Vec<Token>, index: &mut usize, result: &mut ParsingResult) {
    *index += 1;
    if let Token::String(include) = &tokens[*index] {
        let path_str = format!("{}/{}.gui", LIB_DIR, include);
        let path = Path::new(path_str.as_str());
        
        if path.exists() {
            let tokens = match fs::read_to_string(path) {
                Ok(s) => lex(&s),
                Err(e) => panic!("{e}")
            };

            parse(&tokens, result);
        } else {
            panic!("File not found: {}", path_str.as_str());
        }
    } else {
        panic!("expected string, got {}", token_to_string(&tokens[*index]));
    }
}

fn parse_header(tokens: &Vec<Token>, index: &mut usize, result: &mut ParsingResult) {
    *index += 1;
    if let Token::String(header) = &tokens[*index] {
        result.headers.push(String::from(header));
    } else {
        panic!("expected string, got {}", token_to_string(&tokens[*index]));
    }
}

fn parse_arglist(tokens: &Vec<Token>, index: &mut usize, size: usize) -> Vec<Argument> {
    let token = tokens.get(*index);
    let mut arglist: Vec<Argument> = Vec::new();

    if let Some(token) = token {
        match token {
            Token::StartArgList => {
                for _ in 0..size {
                    if *index + 2 >= tokens.len() {
                        panic!("arglist started but not ended");
                    }
                    *index += 1; 
                    let token = tokens.get(*index).unwrap();
                    arglist.push(
                        match token {
                            Token::String(s) => Argument::String(s.to_string()),
                            Token::Number(n) => Argument::Number(*n),
                            Token::Bool(b) => Argument::Number(*b),
                            Token::Identifier(identifier) => {
                                if let LexIdentifier::Type(identifier) = identifier {
                                    Argument::Type(*identifier)
                                } else {
                                    panic!("unrecognized identifier in arglist");
                                }
                            },
                            _ => panic!("unexpected {} in arglist", token_to_string(&token))
                        }
                    );
                    *index += 1;
                    let token = tokens.get(*index).unwrap();
                    match token {
                        Token::ArgListDeliminator => continue,
                        Token::EndArgList => break,
                        _ => panic!("expected ',' or ')', got {}", token_to_string(&token))
                    }
                } 
            },
            _ => panic!("no arglist found")
        }
        arglist
    } else {
        panic!("no arglist found");
    }
}

fn parse_property_definition(tokens: &Vec<Token>, index: &mut usize, token_definition: &LexDefinition, props: &mut Vec<Property>) {
    if let LexDefinition::Object(_) = token_definition {
        panic!("cannot define an object inside another object")
    }
    *index += 1;
    let arglist: Vec<Argument> = parse_arglist(tokens, index, 2);
    if let Argument::String(name_arg) = arglist.get(0).unwrap()  {
        if let Argument::Type(type_arg) = arglist.get(1).unwrap() {
            props.push(
                match token_definition {
                    LexDefinition::Object(_) => panic!("cannot define an object inside another object"),
                    LexDefinition::InlineProp => Property {
                        xml_placement_type: XMLPlacementType::Inline,
                        definition_type: DefinitionType::Prop,
                        internal_value_type: *type_arg,
                        name: name_arg.to_string(),
                        value: None
                    },
                    LexDefinition::InlineArg => Property {
                        xml_placement_type: XMLPlacementType::Inline,
                        definition_type: DefinitionType::Arg,
                        internal_value_type: *type_arg,
                        name: name_arg.to_string(),
                        value: None
                    },
                    LexDefinition::ChildProp => Property {
                        xml_placement_type: XMLPlacementType::Child,
                        definition_type: DefinitionType::Prop,
                        internal_value_type: *type_arg,
                        name: name_arg.to_string(),
                        value: None
                    },
                    LexDefinition::ChildArg => Property {
                        xml_placement_type: XMLPlacementType::Child,
                        definition_type: DefinitionType::Arg,
                        internal_value_type: *type_arg,
                        name: name_arg.to_string(),
                        value: None
                    },
                }
            );
        }
    }
}

fn parse_object(tokens: &Vec<Token>, index: &mut usize, definition_name: &String) -> Object {
    let object = Object {
        children: Vec::new(),
        definition_name: definition_name.to_string(),
        properties: Vec::new()
    };
    
    object
}

fn parse_object_definition(tokens: &Vec<Token>, index: &mut usize) -> Definition {
    *index += 1;

    match tokens[*index] {
        Token::StartBlock => {
            let mut definition: Option<Definition> = None;
            loop {
                *index += 1;
                let token = tokens.get(*index);

                if let Some(token) = token {
                    match token {
                        Token::EndBlock => {
                            return definition.expect("useless object definition");
                        },
                        Token::Definition(token_definition) => {
                            if definition.is_none() {
                                definition = Some(Definition::Raw(Vec::new()))
                            }
                            match &mut definition {
                                Some(definition) => {
                                    match definition {
                                        Definition::Collective(_) => {
                                            panic!("cannot define children in a raw object definition");
                                        },
                                        Definition::Raw(props) => {
                                            parse_property_definition(tokens, index, token_definition, props);
                                        }
                                    }
                                },
                                None => ()
                            }
                        },  
                        Token::Identifier(token_identifier) => {
                            if definition.is_none() {
                                definition = Some(Definition::Collective(Vec::new()));
                            }
                            match definition.as_mut().unwrap() {
                                Definition::Collective(children) => {
                                    match token_identifier {
                                        LexIdentifier::Type(_) => panic!("expected identifier, found type identifier"),
                                        LexIdentifier::Generic(generic_identifier) => children.push(parse_object(tokens, index, generic_identifier))
                                    }
                                } ,
                                Definition::Raw(_) => {
                                    panic!("cannot define Props or Args on a collective object definition");
                                }
                            }
                        }
                        _ => todo!()
                    }
                } else {
                    panic!("block not closed");
                }
            }
        },
        _ => panic!("expected strong, got {}", token_to_string(&tokens[*index]))
    };
}

pub fn parse(tokens: &Vec<Token>, result: &mut ParsingResult) {
    for token in tokens {
        println!("{:?}", token);
    }
    let mut index = 0;
    loop {
        let token = tokens.get(index);
        if let Some(token) = token {
            match token {
                Token::Directive(dir) => {
                    match dir {
                        LexDirective::Include => {
                            parse_include(tokens, &mut index, result);
                        },
                        LexDirective::Header => {
                            parse_header(tokens, &mut index, result);
                        }
                    }
                },
                Token::Definition(def) => {
                    match def {
                        LexDefinition::Object(_name) => {
                            result.definitions.push(parse_object_definition(tokens, &mut index));
                        },
                        _ => panic!("only object definitions are allowed globally")
                    }
                }
                _ => ()
            }
            index += 1;
        } else {
            break;
        }
    }
}
