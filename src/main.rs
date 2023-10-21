use itertools::Itertools;
use std::{
    convert::TryFrom,
    env,
    io::{self, Read},
};

use anyhow::Result;
use json2pyi::{
    inferrer::*,
    target::{GenOutput, Indentation, PythonClass, PythonKind, TargetGenerator},
};
use serde_json_schema::Schema;

fn main() -> Result<()> {
    let args: Vec<String> = env::args().collect();
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer)?;
    let root_name = Some(args[1].to_owned());
    let json = Schema::try_from(buffer.to_owned())?;
    let mut schema = infer_from_json_schema(&json, root_name)?;
    Optimizer {
        to_merge_similar_datatypes: true,
        to_merge_same_unions: true,
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
    Ok(())
}
