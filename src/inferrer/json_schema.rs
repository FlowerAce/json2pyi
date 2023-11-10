use std::collections::BTreeMap;

/// Infer a schema from a given JSONSchemaValue
use indexmap::IndexMap;
use inflector::Inflector;
use serde::Deserialize;
use serde_json_schema::{
    property::{Property, PropertyInstance},
    Schema as JSONSchema, TryFrom,
};

use crate::schema::{ArenaIndex, ITypeArena, Map, NameHints, Schema, Type, TypeArena};

use anyhow::Result;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum JSONSchemaError {
    #[error("the JSON schema is empty")]
    Empty,
}

#[derive(Deserialize, Debug)]
pub struct JSONSchemaMap(BTreeMap<String, String>);

pub fn infer_mult(schemas: &JSONSchemaMap, root_name: Option<String>) -> Result<Schema> {
    let mut closure = InferrerClosure::new();
    for (key, value) in schemas.0.iter() {
        let schema = JSONSchema::try_from(value.as_ref())?;
        closure.insert(schema, key)?;
    }
    closure.collect(root_name)
}

/// Infer a `Schema` from a `JSONSchema`
pub fn infer(json: &JSONSchema, root_name: Option<String>) -> Result<Schema> {
    InferrerClosure::new().run(json, root_name)
}

/// An closure for the inferrer to work
struct InferrerClosure {
    arena: TypeArena,
    roots: IndexMap<String, ArenaIndex>,
}

impl Default for InferrerClosure {
    fn default() -> Self {
        Self::new()
    }
}

impl InferrerClosure {
    pub fn new() -> Self {
        let arena = TypeArena::new();
        let roots = IndexMap::new();
        InferrerClosure { arena, roots }
    }

    fn run(mut self, schema: &JSONSchema, root_name: Option<String>) -> Result<Schema> {
        let json = schema.specification().ok_or(JSONSchemaError::Empty)?;
        let root = self.rinfer(json, root_name, schema);
        let arena = self.arena;
        Ok(Schema { arena, root })
    }

    pub fn insert(&mut self, schema: JSONSchema, outer_name: &String) -> Result<()> {
        let Some(json) = schema.specification() else {
            return Ok(());
        };
        let root = self.rinfer(json, Some(outer_name.to_string()), &schema);
        self.roots.insert(outer_name.to_string(), root);
        Ok(())
    }

    pub fn collect(mut self, outer_name: Option<String>) -> Result<Schema> {
        let fields = self.roots;
        let mut name_hints = NameHints::new();
        if let Some(outer_name) = outer_name {
            name_hints.insert(outer_name);
        }
        let root_type = Type::Map(Map { name_hints, fields });
        let root = self.arena.insert(root_type);
        let arena = self.arena;
        Ok(Schema { arena, root })
    }

    fn rinfer(
        &mut self,
        json: &PropertyInstance,
        outer_name: Option<String>,
        schema: &JSONSchema,
    ) -> ArenaIndex {
        match json {
            PropertyInstance::Integer { .. } => self.arena.get_index_of_primitive(Type::Int),
            PropertyInstance::Number { .. } => self.arena.get_index_of_primitive(Type::Float),
            PropertyInstance::Boolean => self.arena.get_index_of_primitive(Type::Bool),
            PropertyInstance::String => self.arena.get_index_of_primitive(Type::String),
            PropertyInstance::Null => self.arena.get_index_of_primitive(Type::Null),
            PropertyInstance::Array { items } => {
                let items = items.as_ref();
                let value_type = self.rinfer(items, outer_name, schema);
                let array_type = Type::Array(value_type);
                self.arena.insert(array_type)
            }
            PropertyInstance::Object { properties, .. } => {
                let mut fields = IndexMap::new();
                for (key, value) in properties.iter() {
                    let v = match value {
                        Property::Value(t) => t,
                        Property::Ref(r) => r.deref(schema).unwrap(),
                    };
                    fields.insert(
                        key.to_owned(),
                        self.rinfer(v, Some(key.to_pascal_case()), schema),
                    );
                }
                let mut name_hints = NameHints::new();
                if let Some(outer_name) = outer_name {
                    name_hints.insert(outer_name + "Type");
                }
                self.arena.insert(Type::Map(Map { name_hints, fields }))
            }
            PropertyInstance::Empty(_) => self.arena.get_index_of_primitive(Type::Any),
        }
    }
}
