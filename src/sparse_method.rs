use oapi::{OApi, OApiSchema};
use simplelog::info;
use sppparse::{SparsePointedValue, SparseSelector};

use crate::OapiEndpoint;

pub fn sparse_openapi(doc: OApi) -> anyhow::Result<()> {
    let root = doc.root_get().unwrap();
    let mut endpoints: Vec<OapiEndpoint> = Vec::new();

    for (path, config) in root.paths() {
        info!("Parsing path {}", path);
        if let Some(get) = config.get() {
            endpoints.push(OapiEndpoint {
                path: path.to_string(),
                method: "GET".to_string(),
                parameters: get.parameters().to_owned(),
                response: get.responses().get("200").map(|x| x.to_owned()),
                description: get.description().clone().unwrap(),
            });
        }
        if let Some(post) = config.post() {
            endpoints.push(OapiEndpoint {
                path: path.to_string(),
                method: "POST".to_string(),
                parameters: post.parameters().to_owned(),
                response: post.responses().get("200").map(|x| x.to_owned()),
                description: post.description().clone().unwrap(),
            });
        }
        if let Some(put) = config.put() {
            endpoints.push(OapiEndpoint {
                path: path.to_string(),
                method: "PUT".to_string(),
                parameters: put.parameters().to_owned(),
                response: put.responses().get("200").map(|x| x.to_owned()),
                description: put.description().clone().unwrap(),
            });
        }
        if let Some(delete) = config.delete() {
            endpoints.push(OapiEndpoint {
                path: path.to_string(),
                method: "DELETE".to_string(),
                parameters: delete.parameters().to_owned(),
                response: delete.responses().get("200").map(|x| x.to_owned()),
                description: delete.description().clone().unwrap(),
            });
        }
        if let Some(delete) = config.patch() {
            endpoints.push(OapiEndpoint {
                path: path.to_string(),
                method: "PATCH".to_string(),
                parameters: delete.parameters().to_owned(),
                response: delete.responses().get("200").map(|x| x.to_owned()),
                description: delete.description().clone().unwrap(),
            });
        }
    }
    for endpoint in &endpoints {
        // info!("{:#?}", endpoint);
        if endpoint.response.is_none() {
            continue;
        } else {
            match endpoint.response.as_ref().unwrap() {
                SparseSelector::Ref(spec) => match spec.val() {
                    SparsePointedValue::RefRaw(_item) => {
                        todo!("Ref RefRaw");
                    }
                    SparsePointedValue::Obj(_item) => {
                        todo!("Ref Obj");
                    }
                    SparsePointedValue::Null => todo!("Ref null"),
                    SparsePointedValue::Ref(_) => todo!("Ref Ref"),
                },
                SparseSelector::Obj(spec) => match spec {
                    SparsePointedValue::RefRaw(_item) => {
                        todo!("Obj RefRaw");
                    }
                    SparsePointedValue::Obj(item) => {
                        info!("Obj Obj");
                        // println!("{:#?}", item);
                        let s = item.content().get("application/json").unwrap();
                        let schema = s.schema().as_ref().unwrap();
                        match schema {
                            oapi::OperatorSelector::AnyOf(_s) => {
                                todo!("Obj Obj any");
                            }
                            oapi::OperatorSelector::OneOf(_s) => {
                                todo!("Obj Obj one");
                            }
                            oapi::OperatorSelector::AllOf(_s) => {
                                todo!("Obj Obj all");
                            }
                            oapi::OperatorSelector::Not(_s) => {
                                todo!("Obj Obj not");
                            }
                            oapi::OperatorSelector::Val(s) => {
                                // info!("val {:#?}", s);
                                info!("val");
                                match s {
                                    SparseSelector::Ref(a) => {
                                        // info!("ref {:#?}", a);
                                        match a.val() {
                                            SparsePointedValue::RefRaw(_b) => {
                                                todo!("val ref raw");
                                            }
                                            SparsePointedValue::Obj(_b) => {
                                                todo!("val obj");
                                            }
                                            SparsePointedValue::Ref(b) => {
                                                // info!("val ref {:#?}", b);
                                                // b.
                                                match b.val().as_ref() {
                                                    SparsePointedValue::RefRaw(_c) => {
                                                        todo!("val ref ref raw");
                                                    }
                                                    SparsePointedValue::Obj(c) => {
                                                        // info!("val ref obj {:#?}", c);
                                                        parse_open_api_schema(c)?;
                                                    }
                                                    SparsePointedValue::Ref(_c) => {
                                                        todo!("val ref ref");
                                                    }
                                                    SparsePointedValue::Null => {
                                                        todo!("val ref null")
                                                    }
                                                }
                                            }
                                            SparsePointedValue::Null => todo!("val null"),
                                        }
                                    }
                                    SparseSelector::Obj(_a) => {
                                        todo!("val obj");
                                    }
                                    SparseSelector::Null => todo!("val null"),
                                }
                            }
                        }
                    }
                    SparsePointedValue::Null => todo!("Obj null"),
                    SparsePointedValue::Ref(_) => todo!("Obj Ref"),
                },
                SparseSelector::Null => todo!("null"),
            }
            // println!("{:#?}", endpoint.response);
        }
    }
    Ok(())
}

// fn matching(point: &SparsePointedValue<OApiSchema>) {
//     match point {
//         SparsePointedValue::RefRaw(val) => {
//             info!("RefRaw");
//             matching(val.val());
//         }
//         SparsePointedValue::Obj(val) => {
//             info!("Obj");
//             matching(val.val());
//         }
//         SparsePointedValue::Ref(val) => {
//             info!("Ref");
//             matching(val.val());
//         }
//         SparsePointedValue::Null => todo!(),
//     }
// }

fn parse_open_api_schema(schema: &OApiSchema) -> anyhow::Result<()> {
    match schema {
        OApiSchema::Obj(obj) => match obj.as_ref() {
            oapi::OperatorSelector::AnyOf(_) => todo!("obj any"),
            oapi::OperatorSelector::OneOf(_) => todo!("obj one"),
            oapi::OperatorSelector::AllOf(_) => todo!("obj all"),
            oapi::OperatorSelector::Not(_) => todo!("obj not"),
            oapi::OperatorSelector::Val(s) => match s {
                SparseSelector::Ref(_) => todo!("obj val ref"),
                SparseSelector::Obj(_obj) => todo!("obj val obj"),
                SparseSelector::Null => todo!("obj val null"),
            },
        },
        OApiSchema::Array(arr) => {
            // info!("arr {:#?}", arr)
            match arr.as_ref() {
                oapi::OperatorSelector::AnyOf(_) => todo!("arr any"),
                oapi::OperatorSelector::OneOf(_) => todo!("arr one"),
                oapi::OperatorSelector::AllOf(_) => todo!("arr all"),
                oapi::OperatorSelector::Not(_) => todo!("arr not"),
                oapi::OperatorSelector::Val(_) => todo!("arr val"),
            }
        }
        OApiSchema::Numeric(num) => {
            info!("num {:#?}", num);
            match num.as_ref() {
                oapi::OperatorSelector::AnyOf(_) => todo!("num any"),
                oapi::OperatorSelector::OneOf(_) => todo!("num one"),
                oapi::OperatorSelector::AllOf(_) => todo!("num all"),
                oapi::OperatorSelector::Not(_) => todo!("num not"),
                oapi::OperatorSelector::Val(_) => todo!("num val"),
            }
        }
        OApiSchema::String(str) => {
            info!("str {:#?}", str);
            match str.as_ref() {
                oapi::OperatorSelector::AnyOf(_) => todo!("str any"),
                oapi::OperatorSelector::OneOf(_) => todo!("str one"),
                oapi::OperatorSelector::AllOf(_) => todo!("str all"),
                oapi::OperatorSelector::Not(_) => todo!("str not"),
                oapi::OperatorSelector::Val(_) => todo!("str val"),
            }
        }
        OApiSchema::Bool => {
            info!("bool")
        }
        OApiSchema::Null => {
            info!("null")
        }
    }
    Ok(())
}
