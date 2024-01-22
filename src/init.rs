use simplelog::info;
use std::{fs, path::PathBuf};

use lazy_static::lazy_static;

lazy_static! {
    static ref CONFIG_FILE_NAME: std::path::PathBuf = PathBuf::from("config.yaml");
    static ref MODEL_FILE_NAME: std::path::PathBuf = PathBuf::from("model.template");
    static ref SERVICE_FILE_NAME: std::path::PathBuf = PathBuf::from("service.template");
    static ref TEMPLATE_FOLDER_NAME: std::path::PathBuf = PathBuf::from("templates");
}
const CONFIG_FILE: &str = include_str!("../config_data/config.yaml");
const MODEL_FILE: &str = include_str!("../config_data/model.template");
const SERVICE_FILE: &str = include_str!("../config_data/service.template");

/// Initialise the config directory and files
pub fn init() -> anyhow::Result<()> {
    // TODO allow for templates for different languages
    info!("Initialising config directory and files");
    if CONFIG_FILE_NAME.exists() {
        info!("Config file already exists");
    } else {
        info!("Creating config file");
        fs::write(CONFIG_FILE_NAME.as_path(), CONFIG_FILE)?;
    }

    if TEMPLATE_FOLDER_NAME.exists() {
        info!("Template folder already exists");
    } else {
        info!("Creating template folder");
        fs::create_dir(TEMPLATE_FOLDER_NAME.as_path())?;
    }

    let model = TEMPLATE_FOLDER_NAME.join(MODEL_FILE_NAME.as_path());
    let service = TEMPLATE_FOLDER_NAME.join(SERVICE_FILE_NAME.as_path());
    if model.exists() {
        info!("Model template already exists");
    } else {
        info!("Creating model template");
        fs::write(model.as_path(), MODEL_FILE)?;
    }

    if service.exists() {
        info!("Service template already exists");
    } else {
        info!("Creating service template");
        fs::write(service.as_path(), SERVICE_FILE)?;
    }

    info!("The template engine used is Tera (https://keats.github.io/tera/) and very similar to Jinja2, Django templates, Liquid and Twig.");
    Ok(())
}
