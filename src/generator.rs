use super::parser::{
    Statement,
    DefinitionType,
};
use super::lexer::{
    DefinitionType as TokenDefinitionType,
    TypeIdentifierType as TokenTypeIdentifierType,
    Token
};
use std::fs::File;
use std::io::Write;
use std::collections::HashMap;

#[derive(Debug)]
pub struct CachedRawDefinition {
    props: HashMap<String, (TokenTypeIdentifierType, TokenDefinitionType)>,
    args: Vec<(String, TokenTypeIdentifierType, TokenDefinitionType)>
}

#[derive(Debug)]
pub enum CachedDefinition {
    Raw(CachedRawDefinition),
    Collective(String)
}

#[derive(Debug)]
pub struct Generator { 
    statements: Vec<Statement>,
    definitions: HashMap<String, CachedDefinition>,
    header: String
} 

impl Generator {

    pub fn generate_from_collective(&self, children: &Vec<Statement>) -> String {
        let mut result = String::new();

        for child in children {
            match child {
                Statement::Object(object) => {
                    if let Some(definition) = self.definitions.get(&object.name) {
                        match definition {
                            CachedDefinition::Raw(definition) => {
                                if definition.args.len() != object.arguments.len() {
                                    panic!("the '{}' definition expects {} args, {} given", object.name, definition.args.len(), object.arguments.len());
                                }

                                let mut inlines: Vec<(String, String)> = Vec::new();
                                let mut children: Vec<(String, String)> = Vec::new();

                                for i in 0..definition.args.len() {
                                    let defined_arg = &definition.args[i];
                                    let actual_arg = &object.arguments[i];
                                    
                                    // Check if actual and defined are the same type and if so check if the definition specifies it as an inline or a child
                                    match actual_arg {
                                        Token::Bool(_) => {
                                            if let TokenTypeIdentifierType::Bool = defined_arg.1 {
                                                match defined_arg.2 {
                                                    TokenDefinitionType::InlineArg => {
                                                        inlines.push((defined_arg.0.clone(), actual_arg.value_to_string()));
                                                    },
                                                    TokenDefinitionType::ChildArg => {
                                                        children.push((defined_arg.0.clone(), actual_arg.value_to_string()));
                                                    },
                                                    _ => panic!("expected either an InlineArg or a ChildArg, got {}", defined_arg.2.to_string())
                                                }
                                            }
                                        },
                                        Token::String(_) => {
                                            if let TokenTypeIdentifierType::String = defined_arg.1 {
                                                match defined_arg.2 {
                                                    TokenDefinitionType::InlineArg => {
                                                        inlines.push((defined_arg.0.clone(), actual_arg.value_to_string()))
                                                    },
                                                    TokenDefinitionType::ChildArg => {
                                                        children.push((defined_arg.0.clone(), actual_arg.value_to_string()))
                                                    },
                                                    _ => panic!("expected either an InlineArg or a ChildArg, got {}", defined_arg.2.to_string())
                                                }
                                            }
                                        },
                                        Token::Number(_) => {
                                            if let TokenTypeIdentifierType::Number = defined_arg.1 {
                                                match defined_arg.2 {
                                                    TokenDefinitionType::InlineArg => {
                                                        inlines.push((defined_arg.0.clone(), actual_arg.value_to_string()))
                                                    },
                                                    TokenDefinitionType::ChildArg => {
                                                        children.push((defined_arg.0.clone(), actual_arg.value_to_string()))
                                                    },
                                                    _ => panic!("expected either an InlineArg or a ChildArg, got {}", defined_arg.2.to_string())
                                                }
                                            }
                                        },
                                        _ => panic!("expected a Number, String, or Bool, got {}", actual_arg.to_string())
                                    }
                                }

                                for setter in &object.setters {
                                    let defined_prop = definition.props.get(&setter.name);
                                    let actual_prop = &setter.value;

                                    if let Some(defined_prop) = defined_prop {
                                        // Check if actual and defined are the same type and if so check if the definition specifies it as an inline or a child
                                        match actual_prop {
                                            Token::Bool(_) => {
                                                if let TokenTypeIdentifierType::Bool = defined_prop.0 {
                                                    match defined_prop.1 {
                                                        TokenDefinitionType::InlineProp => {
                                                            inlines.push((setter.name.clone(), actual_prop.value_to_string()));
                                                        },
                                                        TokenDefinitionType::ChildProp => {
                                                            children.push((setter.name.clone(), actual_prop.value_to_string()));
                                                        },
                                                        _ => panic!("expected either an InlineArg or a ChildArg, got {}", defined_prop.1.to_string())
                                                    }
                                                }
                                            },
                                            Token::String(_) => {
                                                if let TokenTypeIdentifierType::String = defined_prop.0 {
                                                    match defined_prop.1 {
                                                        TokenDefinitionType::InlineProp => {
                                                            inlines.push((setter.name.clone(), actual_prop.value_to_string()))
                                                        },
                                                        TokenDefinitionType::ChildProp => {
                                                            children.push((setter.name.clone(), actual_prop.value_to_string()))
                                                        },
                                                        _ => panic!("expected either an InlineArg or a ChildArg, got {}", defined_prop.1.to_string())
                                                    }
                                                }
                                            },
                                            Token::Number(_) => {
                                                if let TokenTypeIdentifierType::Number = defined_prop.0 {
                                                    match defined_prop.1 {
                                                        TokenDefinitionType::InlineProp => {
                                                            inlines.push((setter.name.clone(), actual_prop.value_to_string()))
                                                        },
                                                        TokenDefinitionType::ChildProp => {
                                                            children.push((setter.name.clone(), actual_prop.value_to_string()))
                                                        },
                                                        _ => panic!("expected either an InlineArg or a ChildArg, got {}", defined_prop.1.to_string())
                                                    }
                                                }
                                            },
                                            _ => panic!("expected a Number, String, or Bool, got {}", actual_prop.to_string())
                                        }
                                    }

                                }
                                // Generate from the vectors of inlines and children

                                result += "<object class=\"";
                                result += object.name.as_str();
                                result += "\"";

                                for inline in &inlines {
                                    result += " ";
                                    result += inline.0.as_str();
                                    result += "=\"";
                                    result += inline.1.as_str();
                                    result += "\"";
                                }

                                result += ">\n";

                                for child in &children {
                                    result += "<property name=\"";
                                    result += child.0.as_str();
                                    result += "\"";
                                    result += ">";
                                    result += child.1.as_str();
                                    result += "</property>\n";
                                }

                                if object.children.len() > 0 {
                                    result += "<child>";
                                    result += self.generate_from_collective(&object.children).as_str();
                                    result += "</child>\n";
                                }
                                result += "</object>\n"
                            },
                            CachedDefinition::Collective(defintion) => {
                                result += defintion.as_str();
                            }
                        }

                    }
                },
                _ => panic!("found {}, expected object in collective definition", child.to_string())
            }
        }
        

        result
    }

    pub fn generate_from_raw(properties: &Vec<Statement>) -> CachedRawDefinition {
        let mut props: HashMap<String, (TokenTypeIdentifierType, TokenDefinitionType)> = HashMap::new();
        let mut args: Vec<(String, TokenTypeIdentifierType, TokenDefinitionType)> = Vec::new();

        for property in properties {
            if let Statement::Property(property) = property {
                match property.definition_type {
                    TokenDefinitionType::InlineProp | TokenDefinitionType::ChildProp => {
                        props.insert(property.name.clone(), (property.internal_type.clone(), property.definition_type.clone()));
                    },
                    TokenDefinitionType::InlineArg | TokenDefinitionType::ChildArg => {
                        args.push((property.name.clone(), property.internal_type.clone(), property.definition_type.clone()));
                    },
                    _ => panic!("expected a property definition, found {}", property.definition_type.to_string())
                }
            }
        }

        CachedRawDefinition {
            props, args
        }
    }
    
    // Pubs
    pub fn generate(&mut self) {
        for statement in &self.statements {
            match statement {
                Statement::Definition(definition) => {
                    match &definition.definition_type {
                        DefinitionType::Root(filename) => {
                            let mut file_content = String::new();
                            
                            file_content.push_str(self.generate_from_collective(&definition.children).as_str());

                            file_content.push_str("</interface>");

                            let mut file = File::create(format!("{}.ui", filename)).expect("failed to create output file");
                            writeln!(file, "{}", self.header).expect("failed to write to output file");
                            writeln!(file, "{}", file_content).expect("failed to write to output file");
                        },
                        DefinitionType::Collective => {
                            self.definitions.insert(definition.name.clone(), CachedDefinition::Collective(self.generate_from_collective(&definition.children)));
                        },
                        DefinitionType::Raw => {
                            self.definitions.insert(definition.name.clone(), CachedDefinition::Raw(Generator::generate_from_raw(&definition.children)));
                        }
                    }
                },
                Statement::Header(header) => {
                    self.header.push_str(header);
                    self.header.push('\n');
                },
                _ => panic!("this should never ever ever ever ever happen. something must be wrong with the parser if this does happen")
            }
        }
    }

    pub fn new(statements: Vec<Statement>) -> Self {
        Generator {
            statements,
            definitions: HashMap::new(),
            header: String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<interface>\n")
        }
    }
}

