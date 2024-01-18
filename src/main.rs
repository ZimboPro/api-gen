mod config;
mod serde_method;
mod tera_extensions;

use std::{collections::HashMap, ffi::OsStr, path::PathBuf};

use clap::{Args, Parser, Subcommand};
use config::Type;
use merge_yaml_hash::MergeYamlHash;
use oapi::{OApi, OApiParameter, OApiResponse};
use openapiv3::{Parameter, RequestBody, Response};
use serde::Serialize;
use serde_method::DataStructure;
use simplelog::{
    debug, error, info, Color, ColorChoice, ConfigBuilder, Level, LevelFilter, TermLogger,
    TerminalMode,
};
use sppparse::{SparseRoot, SparseSelector};
use tera::{from_value, to_value, Context, Function, Tera, Value};
use tera_extensions::{exists, extended, map_type, map_type_new};
use tera_text_filters::register_all;

use crate::{config::parse_config_file, serde_method::serde_openapi};

#[derive(Debug, Parser, PartialEq, Eq)]
enum Commands {
    /// Generate based off the template
    Generate(GenerateArgs),
    /// Output the markdown help page
    #[command(hide = true)]
    Markdown,
    /// Initialize a new project
    Init,
}

#[derive(Debug, Args, PartialEq, Eq)]
struct GenerateArgs {
    /// OpenAPI file(s) to generate from. It can be a folder
    #[clap(short, long)]
    api: PathBuf,
    /// Output file
    #[clap(short, long)]
    output: PathBuf,
    /// Sets a custom config file
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,
    /// Verbose mode (-v, -vv, -vvv, etc.)
    #[clap(short, long)]
    verbose: bool,
    /// Quiet mode, only displays warnings and errors
    #[clap(short, long)]
    quiet: bool,
}

#[derive(Debug, Clone, Serialize)]
struct Endpoint {
    path: String,
    method: String,
    description: Option<String>,
    parameters: Vec<Parameter>,
    request: Option<RequestBody>,
    response: Option<Response>,
}

#[derive(Debug, Clone, Serialize)]
struct EndpointExtracted {
    path: String,
    method: String,
    description: Option<String>,
    parameters: Vec<Parameter>,
    request: Option<DataStructure>,
    response: Option<DataStructure>,
    flat_response: Vec<DataStructure>,
    flat_request: Vec<DataStructure>,
}

#[derive(Debug, Clone, Serialize, Default)]
struct TemplateData {
    endpoints: Vec<EndpointExtracted>,
    responses: Vec<DataStructure>,
    requests: Vec<DataStructure>,
}

impl TemplateData {
    pub fn combine_responses(&mut self) {
        for endpoint in &self.endpoints {
            for response in &endpoint.flat_response {
                if self.responses.contains(&response) {
                    continue;
                }
                self.responses.push(response.clone());
            }
        }
    }
    pub fn combine_requests(&mut self) {
        for endpoint in &self.endpoints {
            for request in &endpoint.flat_request {
                if self.requests.contains(&request) {
                    continue;
                }
                self.requests.push(request.clone());
            }
        }
    }
}

impl From<Endpoint> for EndpointExtracted {
    fn from(endpoint: Endpoint) -> Self {
        Self {
            path: endpoint.path,
            method: endpoint.method,
            description: endpoint.description,
            parameters: endpoint.parameters,
            request: None,
            response: None,
            flat_response: Vec::new(),
            flat_request: Vec::new(),
        }
    }
}

impl EndpointExtracted {
    pub fn flatten_responses(&mut self) {
        if let Some(response) = self.response.clone() {
            let mut responses = Vec::new();
            flatten_responses(&response, &mut responses);
            self.flat_response = responses;
        }
    }

    pub fn flatten_requests(&mut self) {
        if let Some(request) = self.request.clone() {
            let mut responses = Vec::new();
            flatten_responses(&request, &mut responses);
            self.flat_request = responses;
        }
    }
}

fn flatten_responses(response: &DataStructure, responses: &mut Vec<DataStructure>) {
    if response.property_type == "Object" {
        if responses.contains(response) {
            return;
        }
        responses.push(response.clone());
        for property in response.properties.clone() {
            flatten_responses(&property, responses);
        }
    } else if response.property_type == "Array" {
        responses.push(response.clone());
        for property in response.properties.clone() {
            flatten_responses(&property, responses);
        }
    }
}

fn terminal_setup(args: &GenerateArgs) -> anyhow::Result<()> {
    let config = ConfigBuilder::new()
        .set_level_color(Level::Debug, Some(Color::Cyan))
        .set_level_color(Level::Info, Some(Color::Blue))
        .set_level_color(Level::Warn, Some(Color::Yellow))
        .set_level_color(Level::Error, Some(Color::Magenta))
        .set_level_color(Level::Trace, Some(Color::Green))
        .set_time_level(LevelFilter::Off)
        .build();

    if args.quiet && args.verbose {
        return Err(anyhow::anyhow!(
            "Cannot be quiet and verbose at the same time"
        ));
    }

    let level = if args.quiet {
        LevelFilter::Warn
    } else if args.verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };

    TermLogger::init(level, config, TerminalMode::Stdout, ColorChoice::Auto).unwrap();
    Ok(())
}

fn main() -> anyhow::Result<()> {
    let args = Commands::parse();

    match args {
        Commands::Generate(args) => generate(args),
        Commands::Markdown => Ok(clap_markdown::print_help_markdown::<Commands>()),
        Commands::Init => todo!(),
    }
}

fn generate(args: GenerateArgs) -> anyhow::Result<()> {
    terminal_setup(&args)?;

    if !args.api.exists() {
        return Err(anyhow::anyhow!("OpenAPI file(s) not found"));
    }

    let mut contents = String::new();
    let t = if args.api.is_file() {
        let parent_path = args.api.parent().unwrap();
        let shared_yml = parent_path.join("shared_models.yml");
        let shared_yaml = parent_path.join("shared_models.yaml");
        if shared_yml.exists() {
            info!("Merging with shared_models.yml OpenAPI document");
            let content = vec![
                std::fs::read_to_string(args.api)?,
                std::fs::read_to_string(shared_yml)?,
            ];
            contents = merge(content);
            let merged_file = temp_file::with_contents(contents.as_bytes());
            info!("Parsing OpenAPI document");
            SparseRoot::new_from_file(merged_file.path().to_path_buf())
        } else if shared_yaml.exists() {
            info!("Merging with shared_models.yaml OpenAPI document");
            let content = vec![
                std::fs::read_to_string(args.api)?,
                std::fs::read_to_string(shared_yml)?,
            ];
            contents = merge(content);
            let merged_file = temp_file::with_contents(contents.as_bytes());
            info!("Parsing OpenAPI document");
            SparseRoot::new_from_file(merged_file.path().to_path_buf())
        } else {
            let path = std::env::current_dir().unwrap().join(args.api);
            contents = std::fs::read_to_string(path.clone())?;
            info!("Parsing OpenAPI document");
            SparseRoot::new_from_file(path)
        }
    } else {
        let mut files = find_files(&args.api, OsStr::new("yml"));
        files.append(&mut find_files(&args.api, OsStr::new("yaml")));
        let mut content = Vec::new();
        for file in files {
            content.push(std::fs::read_to_string(file)?);
        }
        contents = merge(content);
        let merged_file = temp_file::with_contents(contents.as_bytes());

        info!("Parsing OpenAPI document");
        SparseRoot::new_from_file(merged_file.path().to_path_buf())
    };

    if let Err(e) = t {
        error!("{}", e);
        return Err(anyhow::anyhow!("OpenAPI file not valid"));
    }
    let doc = OApi::new(t.unwrap());
    if let Err(e) = doc.check() {
        error!("{}", e);
        return Err(anyhow::anyhow!("OpenAPI file not valid"));
    }

    let mut template = serde_openapi(contents)?;
    for e in &mut template.endpoints {
        e.flatten_requests();
        e.flatten_responses();
    }
    template.combine_requests();
    template.combine_responses();

    // sparse_openapi(doc)?;
    let mut tera = match Tera::new("templates/*.dart") {
        Ok(t) => t,
        Err(e) => {
            println!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    register_all(&mut tera);
    let config = parse_config_file(args.config)?;
    tera.register_function("map_type", map_type_new(config.clone()));
    tera.register_function("extended", extended(config.extended.clone()));
    tera.register_function("exists", exists(config.extended));
    let context = Context::from_serialize(&template)?;
    let output = tera.render("service.dart", &context)?;

    std::fs::write(args.output, output)?;
    Ok(())
}

fn merge(files: Vec<String>) -> String {
    let mut hash = MergeYamlHash::new();
    debug!("Merging OpenAPI documents");
    for file in files {
        debug!("Merging file {:?}", file);
        hash.merge(&file);
    }

    hash.to_string()
}

fn find_files(path: &std::path::Path, extension: &OsStr) -> Vec<PathBuf> {
    debug!("Finding files in {:?}", path);
    let mut files = Vec::new();
    for entry in path.read_dir().expect("Failed to read directory").flatten() {
        if entry.path().is_dir() {
            debug!("Found directory {:?}", entry.path());
            files.append(&mut find_files(&entry.path(), extension));
        } else if entry.path().extension() == Some(extension) {
            debug!("Found file {:?}", entry.path());
            files.push(entry.path());
        }
    }
    files
}
