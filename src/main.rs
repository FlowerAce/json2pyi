use itertools::Itertools;
use std::{
    convert::TryFrom,
    env,
    io::{self, Read},
};

use anyhow::Result;
use json2pyi::{
    inferrer::*,
    schema::Schema as PySchema,
    target::{GenOutput, Indentation, PythonClass, PythonKind, TargetGenerator},
};
use serde_json_schema::Schema;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let root_name = Some(args[1].to_owned());
    let second_param = args[2].to_owned();
    let mut raw_data = String::new();
    io::stdin().read_to_string(&mut raw_data)?;
    if second_param == "--mult" {
        let schemas: JSONSchemaMap = serde_json::from_str(&raw_data).unwrap();
        let schema = infer_mult_from_json_schema(&schemas, root_name)?;
        generate(schema);
        return Ok(());
    }
    let json = Schema::try_from(raw_data.to_owned())?;
    let schema = infer_from_json_schema(&json, root_name)?;
    generate(schema);
    Ok(())
}

fn generate(mut schema: PySchema) {
    Optimizer {
        to_merge_similar_datatypes: false,
        to_merge_name_datatypes: true,
        to_merge_same_unions: false,
    }
    .optimize(&mut schema);
    let target = &PythonClass {
        kind: PythonKind::TypedDict,
        to_generate_type_alias_for_union: true,
        indentation: Indentation::Space(4),
    };
    let GenOutput {
        header,
        body,
        additional,
    } = target.generate(&schema);
    let out = [&header, &body, &additional]
        .iter()
        .cloned()
        .filter(|s| !s.is_empty())
        .join("\n");
    println!("{}", out);
}
