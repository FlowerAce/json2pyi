mod json;
mod json_schema;
mod optimizer;
mod unioner;

pub use json::infer as infer_from_json;
pub use json_schema::{
    infer as infer_from_json_schema, infer_mult as infer_mult_from_json_schema, JSONSchemaMap,
};
pub use optimizer::Optimizer;
