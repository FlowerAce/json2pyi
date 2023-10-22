/// Infer a schema from a given JSONSchemaValue
use indexmap::IndexMap;
use inflector::Inflector;
use serde_json_schema::{
    property::{Property, PropertyInstance},
    Schema as JSONSchema,
};

use crate::schema::{ArenaIndex, ITypeArena, Map, NameHints, Schema, Type, TypeArena};

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSONSchemaError {
    #[error("the JSON schema is empty")]
    Empty,
}

/// Infer a `Schema` from a `JSONSchema`
pub fn infer(json: &JSONSchema, root_name: Option<String>) -> Result<Schema> {
    InferrerClosure::new().run(json, root_name)
}

/// An closure for the inferrer to work
struct InferrerClosure {
    arena: TypeArena,
}

impl InferrerClosure {
    fn new() -> Self {
        let arena = TypeArena::new();
        InferrerClosure { arena }
    }

    fn run(mut self, schema: &JSONSchema, root_name: Option<String>) -> Result<Schema> {
        let json = schema.specification().ok_or(JSONSchemaError::Empty)?;
        let root = self.rinfer(&json, root_name);

        let arena = self.arena;
        Ok(Schema { arena, root })
    }

    fn rinfer(&mut self, json: &PropertyInstance, outer_name: Option<String>) -> ArenaIndex {
        match json {
            PropertyInstance::Integer { .. } => self.arena.get_index_of_primitive(Type::Int),
            PropertyInstance::Number { .. } => self.arena.get_index_of_primitive(Type::Float),
            PropertyInstance::Boolean => self.arena.get_index_of_primitive(Type::Bool),
            PropertyInstance::String => self.arena.get_index_of_primitive(Type::String),
            PropertyInstance::Null => self.arena.get_index_of_primitive(Type::Null),
            PropertyInstance::Array { ref items } => {
                let value_type = self.rinfer(&items, None);
                let array_type = Type::Array(value_type);
                self.arena.insert(array_type)
            }
            PropertyInstance::Object { properties, .. } => {
                let mut fields = IndexMap::new();
                for (key, value) in properties.iter() {
                    let v = match value {
                        Property::Value(t) => t,
                        Property::Ref(_) => todo!(),
                    };
                    fields.insert(key.to_owned(), self.rinfer(v, Some(key.to_pascal_case())));
                }
                let mut name_hints = NameHints::new();
                if let Some(outer_name) = outer_name {
                    name_hints.insert(outer_name);
                }
                self.arena.insert(Type::Map(Map { name_hints, fields }))
            }
        }
    }
}
