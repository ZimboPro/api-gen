use std::{collections::HashMap, path::PathBuf};

use anyhow::Ok;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub types: HashMap<String, Type>,
    #[serde(default)]
    pub extended: HashMap<String, String>,
    pub array_layout: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Type {
    pub default: String,
    pub format: Option<HashMap<String, String>>,
}

pub fn parse_config_file(path: Option<PathBuf>) -> anyhow::Result<Config> {
    if let Some(file) = path {
        if file.exists() && file.is_file() {
            let config_file = std::fs::read_to_string(&file).unwrap();
            let extension = file.extension().unwrap();
            let config: Config = if extension == "json" {
                serde_json::from_str(&config_file).unwrap()
            } else {
                serde_yaml::from_str(&config_file).unwrap()
            };
            Ok(config)
        } else {
            Err(anyhow::anyhow!(
                "Config file '{}' is not a file",
                file.display()
            ))
        }
    } else {
        let cwd = std::env::current_dir().unwrap();
        let json_config_file = cwd.join("config.json");
        let yml_config_file = cwd.join("config.yml");
        let yaml_config_file = cwd.join("config.yaml");
        if json_config_file.exists() && json_config_file.is_file() {
            let config_file = std::fs::read_to_string(json_config_file).unwrap();
            let config: Config = serde_json::from_str(&config_file).unwrap();
            Ok(config)
        } else if yml_config_file.exists() && yml_config_file.is_file() {
            let config_file = std::fs::read_to_string(yml_config_file).unwrap();
            let config: Config = serde_yaml::from_str(&config_file).unwrap();
            Ok(config)
        } else if yaml_config_file.exists() && yaml_config_file.is_file() {
            let config_file = std::fs::read_to_string(yaml_config_file).unwrap();
            let config: Config = serde_yaml::from_str(&config_file).unwrap();
            Ok(config)
        } else {
            Err(anyhow::anyhow!(
                "Config file not found in {}",
                cwd.display()
            ))
        }
    }
}
