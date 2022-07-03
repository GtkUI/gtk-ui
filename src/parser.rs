use super::lexer::{Token, Definition as LexDefinition, Directive as LexDirective, TypeIdentifier as LexTypeIdentifier};

enum XMLPlacementType {
    Inline,
    Child
}

enum DefinitionType {
    Prop,
    Arg
}

// NOTE: there are no booleans because true and false are implicitly converted to their numberic counterparts (1, 0)
enum InternalValue {
    String(String),
    Number(i32),
}

struct Property {
    xml_placement_type: XMLPlacementType,
    definition_type: DefinitionType,
    internal_value_type: LexTypeIdentifier,
    name: String,
    value: Option<InternalValue>,
}

struct Object {
    children: Vec<Object>,
    properties: Vec<Property>,
    definition_name: String
}

struct Definition {
    children: Vec<Object>,
    properties: Vec<Property>,
}

struct ParsingResult {
    definitions: Vec<Definition>,
    headers: Vec<String>,
}

pub fn parse(tokens: &Vec<Token>) {
    for token in tokens {
    }
}
