use indexmap::IndexMap;
use openapiv3::{ReferenceOr, Schema, StatusCode};
use serde::{Deserialize, Serialize};
use simplelog::{debug, info};

use crate::{Endpoint, EndpointExtracted, TemplateData};

pub fn serde_openapi(contents: String) -> anyhow::Result<TemplateData> {
    let doc: openapiv3::OpenAPI = serde_yaml::from_str(&contents)?;
    let mut endpoints = Vec::new();
    for (path, config) in doc.paths.iter() {
        for (method, data) in config.as_item().unwrap().iter() {
            debug!("Extracting endpoint: {} {}", method, path);
            endpoints.push(Endpoint {
                path: path.to_string(),
                method: method.to_string(),
                parameters: data
                    .parameters
                    .iter()
                    .map(|p| p.clone().into_item().unwrap())
                    .collect(),
                response: data
                    .responses
                    .responses
                    .get(&StatusCode::Code(200))
                    .map(|x| x.as_item().to_owned())
                    .unwrap()
                    .cloned()
                    .to_owned(),
                request: data
                    .request_body
                    .clone()
                    .map(|request| {
                        let cloned_request = request.as_item().unwrap();
                        cloned_request.to_owned()
                    })
                    .clone(),
                description: data.description.clone(),
            });
        }
    }
    let component_schemas = doc.components.unwrap().schemas;
    let mut template_data = TemplateData::default();
    template_data.base_url = doc.servers.iter().map(|x| x.url.clone()).collect();
    info!("Extracting models");
    for endpoint in &endpoints {
        debug!("Endpoint: {:#?}", endpoint);
        let mut new_endpoint: EndpointExtracted = endpoint.clone().into();
        if let Some(request) = &endpoint.request {
            let mut request = extract_model(
                request
                    .content
                    .get("application/json")
                    .as_ref()
                    .unwrap()
                    .schema
                    .as_ref()
                    .unwrap(),
                &component_schemas,
                "",
                false,
            );
            request.process_data();
            request.is_root = true;
            if request.property_type == "Object" {
                request.object_name = Some(request.name.clone());
            }
            debug!("Request: {:#?}", request);
            new_endpoint.request = Some(request);
        }
        if let Some(response) = &endpoint.response {
            if let Some(media) = response.content.get("application/json") {
                let mut response = extract_model(
                    media.schema.as_ref().unwrap(),
                    &component_schemas,
                    "",
                    false,
                );
                response.process_data();
                response.is_root = true;
                if response.property_type == "Object" {
                    response.object_name = Some(response.name.clone());
                }
                debug!("Response: {:#?}", response);
                new_endpoint.response = Some(response);
            }
        }
        template_data.endpoints.push(new_endpoint);
    }
    Ok(template_data)
}

fn extract_model(
    schema: &ReferenceOr<Schema>,
    component_schemas: &IndexMap<String, ReferenceOr<Schema>>,
    name: &str,
    is_array: bool,
) -> DataStructure {
    match schema {
        ReferenceOr::Reference { reference } => {
            debug!("Reference: {}", reference);
            let name = reference.split('/').last().unwrap();
            let reference_schema = component_schemas.get(name).unwrap();
            extract_model(reference_schema, component_schemas, name, is_array)
        }
        ReferenceOr::Item(schema) => {
            extract_model_from_schema(schema, component_schemas, name, is_array)
        }
    }
}

fn extract_model_from_schema(
    schema: &Schema,
    component_schemas: &IndexMap<String, ReferenceOr<Schema>>,
    name: &str,
    is_array: bool,
) -> DataStructure {
    match &schema.schema_kind {
        openapiv3::SchemaKind::Type(t) => match t {
            openapiv3::Type::String(str) => DataStructure {
                name: name.to_string(),
                description: schema.schema_data.description.clone(),
                format: match &str.format {
                    openapiv3::VariantOrUnknownOrEmpty::Item(item) => Some(match item {
                        openapiv3::StringFormat::Date => "Date".to_string(),
                        openapiv3::StringFormat::DateTime => "DateTime".to_string(),
                        openapiv3::StringFormat::Password => "Password".to_string(),
                        openapiv3::StringFormat::Byte => "Byte".to_string(),
                        openapiv3::StringFormat::Binary => "Binary".to_string(),
                    }),
                    openapiv3::VariantOrUnknownOrEmpty::Unknown(format) => Some(format.to_string()),
                    openapiv3::VariantOrUnknownOrEmpty::Empty => None,
                },
                required: false,
                properties: Vec::new(),
                required_properties: Vec::new(),
                property_type: "String".to_string(),
                object_name: None,
                is_root: false,
                pattern: str.pattern.clone(),
                min_length: str.min_length.map(|x| x.into()),
                max_length: str.max_length.map(|x| x.into()),
                ..Default::default()
            },
            openapiv3::Type::Number(num) => DataStructure {
                name: name.to_string(),
                description: schema.schema_data.description.clone(),
                format: match &num.format {
                    openapiv3::VariantOrUnknownOrEmpty::Item(item) => Some(match item {
                        openapiv3::NumberFormat::Float => "Float".to_string(),
                        openapiv3::NumberFormat::Double => "Double".to_string(),
                    }),
                    openapiv3::VariantOrUnknownOrEmpty::Unknown(format) => Some(format.to_string()),
                    openapiv3::VariantOrUnknownOrEmpty::Empty => None,
                },
                required: false,
                properties: Vec::new(),
                required_properties: Vec::new(),
                property_type: "Number".to_string(),
                object_name: None,
                is_root: false,
                pattern: None,
                min: num.minimum.map(|x| x.into()),
                max: num.maximum.map(|x| x.into()),
                ..Default::default()
            },
            openapiv3::Type::Integer(int) => DataStructure {
                name: name.to_string(),
                description: schema.schema_data.description.clone(),
                format: match &int.format {
                    openapiv3::VariantOrUnknownOrEmpty::Item(item) => Some(match item {
                        openapiv3::IntegerFormat::Int32 => "Int32".to_string(),
                        openapiv3::IntegerFormat::Int64 => "Int64".to_string(),
                    }),
                    openapiv3::VariantOrUnknownOrEmpty::Unknown(format) => Some(format.to_string()),
                    openapiv3::VariantOrUnknownOrEmpty::Empty => None,
                },
                required: false,
                properties: Vec::new(),
                required_properties: Vec::new(),
                property_type: "Integer".to_string(),
                object_name: None,
                is_root: false,
                pattern: None,
                max: int.maximum.map(|x| x.into()),
                min: int.minimum.map(|x| x.into()),
                ..Default::default()
            },
            openapiv3::Type::Object(obj) => {
                let mut response = DataStructure {
                    name: name.to_string(),
                    description: schema.schema_data.description.clone(),
                    format: None,
                    required: false,
                    properties: Vec::new(),
                    required_properties: obj.required.clone(),
                    property_type: if is_array {
                        "Array".to_string()
                    } else {
                        "Object".to_string()
                    },
                    object_name: Some(format!("{}Object", name)),
                    is_root: false,
                    pattern: None,
                    ..Default::default()
                };
                for (name, schema) in &obj.properties {
                    match schema {
                        ReferenceOr::Reference { reference } => {
                            let name = reference.split('/').last().unwrap();
                            let reference_schema = component_schemas.get(name).unwrap();
                            extract_model(reference_schema, component_schemas, name, false);
                        }
                        ReferenceOr::Item(item) => {
                            response.properties.push(extract_model_from_schema(
                                item.as_ref(),
                                component_schemas,
                                name,
                                false,
                            ));
                        }
                    }
                }
                response
            }
            openapiv3::Type::Array(arr) => {
                let mut array = DataStructure {
                    name: name.to_string(),
                    description: schema.schema_data.description.clone(),
                    format: None,
                    required: false,
                    properties: Vec::new(),
                    required_properties: Vec::new(),
                    property_type: "Array".to_string(),
                    object_name: None,
                    is_root: false,
                    pattern: None,
                    min_length: arr.min_items.map(|x| x.into()),
                    max_length: arr.max_items.map(|x| x.into()),
                    ..Default::default()
                };

                array.properties.push(match arr.items.as_ref().unwrap() {
                    ReferenceOr::Reference { reference } => {
                        let name = reference.split('/').last().unwrap();
                        let reference_schema = component_schemas.get(name).unwrap();
                        extract_model(reference_schema, component_schemas, name, false)
                    }
                    ReferenceOr::Item(item) => {
                        debug!("Array: {:#?}", item);
                        extract_model_from_schema(
                            item.as_ref(),
                            component_schemas,
                            &format!("{}Item", name),
                            false,
                        )
                    }
                });
                array
            }
            openapiv3::Type::Boolean {} => DataStructure {
                name: name.to_string(),
                description: schema.schema_data.description.clone(),
                format: None,
                required: false,
                properties: Vec::new(),
                required_properties: Vec::new(),
                property_type: "Boolean".to_string(),
                object_name: None,
                is_root: false,
                pattern: None,
                ..Default::default()
            },
        },
        openapiv3::SchemaKind::OneOf { one_of: _ } => todo!("extract one of"),
        openapiv3::SchemaKind::AllOf { all_of: _ } => todo!("extract all of"),
        openapiv3::SchemaKind::AnyOf { any_of: _ } => todo!("extract any of"),
        openapiv3::SchemaKind::Not { not: _ } => todo!("extract not"),
        openapiv3::SchemaKind::Any(_any) => todo!("extract any"),
    }
}

#[derive(Debug, Clone, Default, Serialize, PartialEq, Eq, Deserialize)]
pub struct DataStructure {
    pub name: String,
    pub description: Option<String>,
    pub format: Option<String>,
    pub required: bool,
    pub properties: Vec<DataStructure>,
    pub required_properties: Vec<String>,
    pub property_type: String,
    pub object_name: Option<String>,
    pub is_root: bool,
    pub pattern: Option<String>,
    pub min: Option<Int64FloatOrUsize>,
    pub max: Option<Int64FloatOrUsize>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Int64FloatOrUsize {
    Int(i64),
    UInt(usize),
    Float(f64),
}

impl PartialEq for Int64FloatOrUsize {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Int(a), Self::Int(b)) => a == b,
            (Self::UInt(a), Self::UInt(b)) => a == b,
            (Self::Float(a), Self::Float(b)) => a == b,
            _ => false,
        }
    }
}

impl Eq for Int64FloatOrUsize {}

impl Default for Int64FloatOrUsize {
    fn default() -> Self {
        Self::UInt(0)
    }
}

impl From<usize> for Int64FloatOrUsize {
    fn from(value: usize) -> Self {
        Self::UInt(value)
    }
}

impl From<i64> for Int64FloatOrUsize {
    fn from(value: i64) -> Self {
        Self::Int(value)
    }
}

impl From<f64> for Int64FloatOrUsize {
    fn from(value: f64) -> Self {
        Self::Float(value)
    }
}

impl DataStructure {
    fn process_data(&mut self) {
        if self.property_type == "Array" {
            debug!("Array: {:#?}", self);
            if !self.properties.is_empty() {
                self.object_name = if let Some(name) = self.properties[0].object_name.clone() {
                    Some(name)
                } else {
                    Some(self.properties[0].property_type.clone())
                };
            } else {
                self.object_name = Some(self.property_type.clone());
            }
        }
        for property in &mut self.properties {
            if self.required_properties.contains(&property.name) {
                property.required = true;
            }
            property.process_data();
        }
    }
}
