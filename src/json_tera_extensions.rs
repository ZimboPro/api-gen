use std::collections::HashMap;

use mock_json::mock;
use rand::{rngs::ThreadRng, Rng};
use tera::{from_value, Function, Value};

use crate::{
    config::Config,
    serde_method::{DataStructure, Int64FloatOrUsize},
};

pub fn json_value_random(config: Config) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("structure") {
                None => Err("Expect value to be set".into()),
                Some(type_name) => match from_value::<DataStructure>(type_name.clone()) {
                    Ok(v) => {
                        let mut r = rand::thread_rng();
                        if v.is_root {
                            let sample_json =
                                data_structure_to_random_json_with_value(&v, &config, &mut r);
                            Ok(mock(&sample_json))
                        } else {
                            Err("Expected the root object".into())
                        }
                    }
                    Err(_) => Err("oops".into()),
                },
            }
        },
    )
}

fn data_structure_to_random_json_with_value(
    data_structure: &DataStructure,
    config: &Config,
    random: &mut ThreadRng,
) -> serde_json::Value {
    let generate_json: bool = random.gen();
    if !data_structure.required && !generate_json {
        return serde_json::Value::Null;
    }
    match data_structure.property_type.as_str() {
        "Object" => {
            let mut map = serde_json::Map::new();
            for property in &data_structure.properties {
                let generate_json: bool = random.gen();
                if property.required || (!property.required && generate_json) {
                    map.insert(
                        property.name.clone(),
                        data_structure_to_random_json_with_value(property, config, random),
                    );
                }
            }
            serde_json::Value::Object(map)
        }
        "Array" => {
            let vec = vec![data_structure_to_random_json_with_value(
                &data_structure.properties[0],
                config,
                random,
            )];
            serde_json::Value::Array(vec)
        }
        "String" => match &data_structure.format {
            Some(format) => match format.as_str() {
                "Date" => serde_json::Value::String("@Date".to_string()),
                "DateTime" => serde_json::Value::String("@DateTime".to_string()),
                "Byte" => serde_json::Value::String("byte data".to_string()),
                "Binary" => serde_json::Value::String("binary data".to_string()),
                _ => serde_json::Value::String("@Sentence".to_string()),
            },
            None => serde_json::Value::String("@Sentence".to_string()),
        },
        "Number" => match &data_structure.format {
            Some(format) => match format.as_str() {
                "Float" => {
                    let min = data_structure
                        .min
                        .clone()
                        .unwrap_or(Int64FloatOrUsize::Float(std::f32::MIN as f64));
                    let max: Int64FloatOrUsize = data_structure
                        .max
                        .clone()
                        .unwrap_or(Int64FloatOrUsize::Float(std::f32::MAX as f64));
                    match (min, max) {
                        (Int64FloatOrUsize::Float(min), Int64FloatOrUsize::Float(max)) => {
                            serde_json::Value::String(format!("@Float|{}~{}", min, max))
                        }
                        _ => unreachable!(),
                    }
                }
                "Double" => {
                    let min = data_structure
                        .min
                        .clone()
                        .unwrap_or(Int64FloatOrUsize::Int(std::i64::MIN));
                    let max: Int64FloatOrUsize = data_structure
                        .max
                        .clone()
                        .unwrap_or(Int64FloatOrUsize::Int(std::i64::MAX));
                    match (min, max) {
                        (Int64FloatOrUsize::Int(min), Int64FloatOrUsize::Int(max)) => {
                            serde_json::Value::String(format!("@Float|{}~{}", min, max))
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            },
            None => {
                let min = data_structure
                    .min
                    .clone()
                    .unwrap_or(Int64FloatOrUsize::Float(std::f32::MIN as f64));
                let max: Int64FloatOrUsize = data_structure
                    .max
                    .clone()
                    .unwrap_or(Int64FloatOrUsize::Float(std::f32::MAX as f64));
                match (min, max) {
                    (Int64FloatOrUsize::Float(min), Int64FloatOrUsize::Float(max)) => {
                        serde_json::Value::String(format!("@Float|{}~{}", min, max))
                    }
                    _ => unreachable!(),
                }
            }
        },
        "Integer" => match &data_structure.format {
            Some(format) => match format.as_str() {
                "Int32" => {
                    let min = data_structure
                        .min
                        .clone()
                        .unwrap_or(Int64FloatOrUsize::Int(std::i32::MIN as i64));
                    let max: Int64FloatOrUsize = data_structure
                        .max
                        .clone()
                        .unwrap_or(Int64FloatOrUsize::Int(std::i32::MAX as i64));
                    match (min, max) {
                        (Int64FloatOrUsize::Int(min), Int64FloatOrUsize::Int(max)) => {
                            serde_json::Value::String(format!("@Number|{}~{}", min, max))
                        }
                        _ => unreachable!(),
                    }
                }
                "Int64" => {
                    let min = data_structure
                        .min
                        .clone()
                        .unwrap_or(Int64FloatOrUsize::Int(std::i64::MIN));
                    let max: Int64FloatOrUsize = data_structure
                        .max
                        .clone()
                        .unwrap_or(Int64FloatOrUsize::Int(std::i64::MAX));
                    match (min, max) {
                        (Int64FloatOrUsize::Int(min), Int64FloatOrUsize::Int(max)) => {
                            serde_json::Value::String(format!("@Number|{}~{}", min, max))
                        }
                        _ => unreachable!(),
                    }
                }
                _ => unreachable!(),
            },
            None => {
                let min = data_structure
                    .min
                    .clone()
                    .unwrap_or(Int64FloatOrUsize::Int(std::i32::MIN as i64));
                let max: Int64FloatOrUsize = data_structure
                    .max
                    .clone()
                    .unwrap_or(Int64FloatOrUsize::Int(std::i32::MAX as i64));
                match (min, max) {
                    (Int64FloatOrUsize::Int(min), Int64FloatOrUsize::Int(max)) => {
                        serde_json::Value::String(format!("@Number|{}~{}", min, max))
                    }
                    _ => unreachable!(),
                }
            }
        },
        "Boolean" => serde_json::Value::String("@Bool".to_string()),
        _ => unreachable!(),
    }
}
