use super::parser::{
    Statement,
    StatementValue,
    DefinitionType,
};
use super::lexer::{
    DefinitionType as TokenDefinitionType,
    TypeIdentifierType as TokenTypeIdentifierType,
    Token,
    TokenValue,
    Position
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

    fn is_valid_type(token: &Token, expected_type: &TokenTypeIdentifierType) -> Result<bool, (String, Position)> {
        match token.value {
            TokenValue::Bool(_) => {
                Ok(matches!(expected_type, TokenTypeIdentifierType::Bool))
            },
            TokenValue::Number(_) => {
                Ok(matches!(expected_type, TokenTypeIdentifierType::Number))
            },
            TokenValue::String(_) => {
                Ok(matches!(expected_type, TokenTypeIdentifierType::String))
            },
            _ => Err((format!("{} is not a primitive and therefore it's type cannot be checked", token.to_string()), token.position))
        }
    }

    pub fn generate_from_collective(&self, children: &Vec<Statement>) -> Result<String, (String, Position)> {
        let mut result = String::new();

        for child in children {
            match &child.value {
                StatementValue::Object(object) => {
                    if let Some(definition) = self.definitions.get(&object.name) {
                        match definition {
                            CachedDefinition::Raw(definition) => {
                                if definition.args.len() != object.arguments.len() {
                                    return Err((format!("the '{}' definition expects {} args, {} given", object.name, definition.args.len(), object.arguments.len()), child.position));
                                }

                                let mut inlines: Vec<(String, String)> = Vec::new();
                                let mut children: Vec<(String, String)> = Vec::new();

                                for i in 0..definition.args.len() {
                                    let defined_arg = &definition.args[i];
                                    let actual_arg = &object.arguments[i];
                                    
                                    // Check if actual and defined are the same type and if so check if the definition specifies it as an inline or a child
                                    if let Ok(is_valid) = Generator::is_valid_type(&actual_arg, &defined_arg.1) {
                                        if is_valid {
                                            match defined_arg.2 {
                                                TokenDefinitionType::InlineArg => {
                                                    inlines.push((defined_arg.0.clone(), actual_arg.value_to_string()));
                                                },
                                                TokenDefinitionType::ChildArg => {
                                                    children.push((defined_arg.0.clone(), actual_arg.value_to_string()));
                                                },
                                                _ => return Err((format!("expected either an InlineArg or a ChildArg, got {}", defined_arg.2.to_string()), actual_arg.position))
                                            }
                                        }
                                    }
                                }

                                for setter in &object.setters {
                                    let defined_prop = definition.props.get(&setter.name);
                                    let actual_prop = &setter.value;

                                    if let Some(defined_prop) = defined_prop {
                                        // Check if actual and defined are the same type and if so check if the definition specifies it as an inline or a child
                                        if let Ok(is_valid) = Generator::is_valid_type(&actual_prop, &defined_prop.0) {
                                            if is_valid {
                                                match defined_prop.1 {
                                                    TokenDefinitionType::InlineProp => {
                                                        inlines.push((setter.name.clone(), actual_prop.value_to_string()));
                                                    },
                                                    TokenDefinitionType::ChildProp => {
                                                        children.push((setter.name.clone(), actual_prop.value_to_string()));
                                                    },
                                                    _ => return Err((format!("expected either an InlineArg or a ChildArg, got {}", defined_prop.1.to_string()), actual_prop.position))
                                                }
                                            }
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
                                    result += "<child>\n";
                                    match self.generate_from_collective(&object.children) {
                                        Ok(collective) => {
                                            result += collective.as_str();
                                        },
                                        Err(err) => return Err(err)
                                    }
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
                _ => return Err((format!("found {}, expected object in collective definition", child.to_string()), child.position))
            }
        }
        

        Ok(result)
    }

    pub fn generate_from_raw(properties: &Vec<Statement>) -> Result<CachedRawDefinition, (String, Position)> {
        let mut props: HashMap<String, (TokenTypeIdentifierType, TokenDefinitionType)> = HashMap::new();
        let mut args: Vec<(String, TokenTypeIdentifierType, TokenDefinitionType)> = Vec::new();

        for property in properties {
            if let StatementValue::Property(property_value) = &property.value {
                match property_value.definition_type {
                    TokenDefinitionType::InlineProp | TokenDefinitionType::ChildProp => {
                        props.insert(property_value.name.clone(), (property_value.internal_type.clone(), property_value.definition_type.clone()));
                    },
                    TokenDefinitionType::InlineArg | TokenDefinitionType::ChildArg => {
                        args.push((property_value.name.clone(), property_value.internal_type.clone(), property_value.definition_type.clone()));
                    },
                    _ => return Err((format!("expected a property definition, found {}", property_value.definition_type.to_string()), property.position))
                }
            }
        }

        Ok(CachedRawDefinition {
            props, args
        })
    }
    
    // Pubs
    pub fn generate(&mut self) -> Result<(), (String, Position)> {
        for statement in &self.statements {
            match &statement.value {
                StatementValue::Definition(definition) => {
                    match &definition.definition_type {
                        DefinitionType::Root(filename) => {
                            print!("Writing {}.ui... ", filename);
                            let mut file_content = String::new();
                            match self.generate_from_collective(&definition.children) {
                                Ok(collective) => {
                                    file_content.push_str(collective.as_str());
                                },
                                Err(err) => return Err(err)
                            }
                            file_content.push_str("</interface>");

                            let mut file = File::create(format!("{}.ui", filename)).expect("failed to create output file");
                            writeln!(file, "{}", self.header).expect("failed to write to output file");
                            writeln!(file, "{}", file_content).expect("failed to write to output file");
                            println!("done!");
                        },
                        DefinitionType::Collective => {
                            match self.generate_from_collective(&definition.children) {
                                Ok(collective) => {
                                    self.definitions.insert(definition.name.clone(), CachedDefinition::Collective(collective));
                                },
                                Err(err) => return Err(err)
                            }
                        },
                        DefinitionType::Raw => {
                            match Generator::generate_from_raw(&definition.children) {
                                Ok(raw) => {
                                    self.definitions.insert(definition.name.clone(), CachedDefinition::Raw(raw));
                                },
                                Err(err) => return Err(err)
                            }
                        }
                    }
                },
                StatementValue::Header(header) => {
                    self.header.push_str(header);
                    self.header.push('\n');
                },
                _ => return Err((format!("this should never ever ever ever ever happen. something must be wrong with the parser if this does happen"), statement.position))
            }
        }
        Ok(())
    }

    pub fn new(statements: Vec<Statement>) -> Self {
        Generator {
            statements,
            definitions: HashMap::new(),
            header: String::from("<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<interface>\n")
        }
    }
}

