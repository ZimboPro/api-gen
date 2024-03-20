use std::collections::HashMap;

use simplelog::debug;
use tera::{from_value, to_value, Function, Value};

use crate::{
    config::{Config, Type},
    serde_method::{DataStructure, Int64FloatOrUsize},
};
use mock_json::mock;

pub fn extended(extended: HashMap<String, String>) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("key") {
                Some(val) => match from_value::<String>(val.clone()) {
                    Ok(v) => Ok(to_value(extended.get(&v).unwrap()).unwrap()),
                    Err(_) => Err("oops".into()),
                },
                None => Err("oops".into()),
            }
        },
    )
}

pub fn exists(extended: HashMap<String, String>) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("key") {
                Some(val) => match from_value::<String>(val.clone()) {
                    Ok(v) => Ok(Value::Bool(extended.contains_key(&v))),
                    Err(_) => Err("oops".into()),
                },
                None => Err("oops".into()),
            }
        },
    )
}

pub fn map_type(types: HashMap<String, Type>) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match (args.get("type"), args.get("format")) {
                (None, None) => Err("Expect type to be set".into()),
                (None, Some(_)) => Err("Expect type to be set".into()),
                (Some(type_name), None) => match from_value::<String>(type_name.clone()) {
                    Ok(v) => Ok(to_value(&types.get(&v).unwrap().default).unwrap()),
                    Err(_) => Err("oops".into()),
                },
                (Some(type_name), Some(format)) => {
                    if !format.is_null() {
                        match (
                            from_value::<String>(type_name.clone()),
                            from_value::<String>(format.clone()),
                        ) {
                            (Ok(type_name), Ok(format)) => Ok(to_value(
                                types
                                    .get(&type_name)
                                    .unwrap()
                                    .format
                                    .as_ref()
                                    .unwrap()
                                    .get(&format)
                                    .unwrap(),
                            )
                            .unwrap()),
                            (Ok(_), Err(_)) => Err("Failed to parse value".into()),
                            (Err(_), Ok(_)) => Err("Failed to parse value".into()),
                            (Err(_), Err(_)) => Err("Failed to parse value".into()),
                            // Ok(v) => Ok(to_value(types.types.get(&v).unwrap().default).unwrap()),
                            // Err(_) => Err("oops".into()),
                        }
                    } else {
                        match from_value::<String>(type_name.clone()) {
                            Ok(v) => Ok(to_value(&types.get(&v).unwrap().default).unwrap()),
                            Err(_) => Err("oops".into()),
                        }
                    }
                }
            }
        },
    )
}

// TODO determine if better way (preprocess the mapped types)
pub fn map_type_new(config: Config) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("type") {
                None => Err("Expect type to be set".into()),
                Some(type_name) => match from_value::<DataStructure>(type_name.clone()) {
                    Ok(v) => {
                        let openapi_type = if let Some(t) = v.object_name {
                            debug!("Array {}", t);
                            t
                        } else {
                            v.property_type.clone()
                        };
                        debug!("Property Type {}", v.property_type);
                        let resulting_type = match v.format {
                            Some(format) => config
                                .types
                                .get(&openapi_type)
                                .unwrap()
                                .format
                                .as_ref()
                                .unwrap()
                                .get(&format)
                                .unwrap(),
                            None => &config.types.get(&v.property_type).unwrap().default,
                        };
                        if v.property_type == "Array" {
                            debug!("Resulting Array {}", resulting_type);
                            return Ok(to_value(
                                config.array_layout.replace("{type}", resulting_type),
                            )
                            .unwrap());
                        }
                        Ok(to_value(resulting_type).unwrap())
                    }
                    Err(_) => Err("oops".into()),
                },
            }
        },
    )
}

pub fn json_typing(config: Config) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("structure") {
                None => Err("Expect value to be set".into()),
                Some(type_name) => match from_value::<DataStructure>(type_name.clone()) {
                    Ok(v) => {
                        if v.is_root {
                            Ok(data_structure_to_json(&v, &config))
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

fn data_structure_to_json(data_structure: &DataStructure, config: &Config) -> serde_json::Value {
    match data_structure.property_type.as_str() {
        "Object" => {
            let mut map = serde_json::Map::new();
            for property in &data_structure.properties {
                map.insert(
                    property.name.clone(),
                    data_structure_to_json(property, config),
                );
            }
            serde_json::Value::Object(map)
        }
        "Array" => {
            let vec = vec![data_structure_to_json(
                &data_structure.properties[0],
                config,
            )];
            serde_json::Value::Array(vec)
        }
        "String" => property_to_type(
            &data_structure.property_type,
            &data_structure.format,
            config,
        ),
        "Number" => property_to_type(
            &data_structure.property_type,
            &data_structure.format,
            config,
        ),
        "Integer" => property_to_type(
            &data_structure.property_type,
            &data_structure.format,
            config,
        ),
        "Boolean" => property_to_type(
            &data_structure.property_type,
            &data_structure.format,
            config,
        ),
        _ => unreachable!(),
    }
}

fn property_to_type(property_type: &String, format: &Option<String>, config: &Config) -> Value {
    let resulting_type = match format {
        Some(format) => config
            .types
            .get(property_type)
            .unwrap()
            .format
            .as_ref()
            .unwrap()
            .get(format.as_str())
            .unwrap()
            .to_string(),
        None => config.types.get(property_type).unwrap().default.to_string(),
    };
    to_value(resulting_type).unwrap()
}

pub fn json_value(config: Config) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("structure") {
                None => Err("Expect value to be set".into()),
                Some(type_name) => match from_value::<DataStructure>(type_name.clone()) {
                    Ok(v) => {
                        if v.is_root {
                            let sample_json = data_structure_to_json_with_value(&v, &config);
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

fn data_structure_to_json_with_value(
    data_structure: &DataStructure,
    _config: &Config,
) -> serde_json::Value {
    match data_structure.property_type.as_str() {
        "Object" => {
            let mut map = serde_json::Map::new();
            for property in &data_structure.properties {
                map.insert(
                    property.name.clone(),
                    data_structure_to_json_with_value(property, _config),
                );
            }
            serde_json::Value::Object(map)
        }
        "Array" => {
            let vec = vec![data_structure_to_json_with_value(
                &data_structure.properties[0],
                _config,
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

pub fn json_minimum_value(config: Config) -> impl Function {
    Box::new(
        move |args: &HashMap<String, Value>| -> tera::Result<Value> {
            match args.get("structure") {
                None => Err("Expect value to be set".into()),
                Some(type_name) => match from_value::<DataStructure>(type_name.clone()) {
                    Ok(v) => {
                        if v.is_root {
                            let sample_json =
                                data_structure_to_minimum_json_with_value(&v, &config);
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

fn data_structure_to_minimum_json_with_value(
    data_structure: &DataStructure,
    _config: &Config,
) -> serde_json::Value {
    if !data_structure.required {
        return serde_json::Value::Null;
    }
    match data_structure.property_type.as_str() {
        "Object" => {
            let mut map = serde_json::Map::new();
            for property in &data_structure.properties {
                if property.required {
                    map.insert(
                        property.name.clone(),
                        data_structure_to_minimum_json_with_value(property, _config),
                    );
                }
            }
            serde_json::Value::Object(map)
        }
        "Array" => {
            let vec = vec![data_structure_to_minimum_json_with_value(
                &data_structure.properties[0],
                _config,
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
