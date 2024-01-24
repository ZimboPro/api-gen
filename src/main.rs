mod config;
mod init;
mod serde_method;
mod tera_extensions;

use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

use clap::{Args, Parser};

use config::Config;
use init::init;
use merge_yaml_hash::MergeYamlHash;
use oapi::OApi;
use openapiv3::{Parameter, RequestBody, Response};
use serde::Serialize;
use serde_method::DataStructure;
use simplelog::{
    debug, error, info, warn, Color, ColorChoice, ConfigBuilder, Level, LevelFilter, TermLogger,
    TerminalMode,
};
use sppparse::SparseRoot;
use tera::{Context, Tera};
use tera_extensions::{exists, extended, map_type_new};
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
    /// Outputs the context as JSON
    Context(ContextGenerateArgs),
}

// TODO implement validation method
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

#[derive(Debug, Args, PartialEq, Eq)]
struct ContextGenerateArgs {
    /// OpenAPI file(s) to generate from. It can be a folder
    #[clap(short, long)]
    api: PathBuf,
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
                if self.responses.contains(response) {
                    continue;
                }
                self.responses.push(response.clone());
            }
        }
    }
    pub fn combine_requests(&mut self) {
        for endpoint in &self.endpoints {
            for request in &endpoint.flat_request {
                if self.requests.contains(request) {
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

fn terminal_setup(quiet: bool, verbose: bool) -> anyhow::Result<()> {
    let config = ConfigBuilder::new()
        .set_level_color(Level::Debug, Some(Color::Cyan))
        .set_level_color(Level::Info, Some(Color::Blue))
        .set_level_color(Level::Warn, Some(Color::Yellow))
        .set_level_color(Level::Error, Some(Color::Magenta))
        .set_level_color(Level::Trace, Some(Color::Green))
        .set_time_level(LevelFilter::Off)
        .build();

    if quiet && verbose {
        return Err(anyhow::anyhow!(
            "Cannot be quiet and verbose at the same time"
        ));
    }

    let level = if quiet {
        LevelFilter::Warn
    } else if verbose {
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
        Commands::Markdown => {
            clap_markdown::print_help_markdown::<Commands>();
            Ok(())
        }
        Commands::Init => init(),
        Commands::Context(args) => generate_context(args),
    }
}

fn generate_context(args: ContextGenerateArgs) -> anyhow::Result<()> {
    terminal_setup(args.quiet, args.verbose)?;
    if !args.api.exists() {
        return Err(anyhow::anyhow!("OpenAPI file(s) not found"));
    }
    let contents = get_open_api_content_and_doc(&args.api)?;

    let mut template = serde_openapi(contents)?;
    for e in &mut template.endpoints {
        e.flatten_requests();
        e.flatten_responses();
    }
    template.combine_requests();
    template.combine_responses();
    let mut tera = Tera::default();
    let context = Context::from_serialize(&template)?;
    let output = tera.render_str("{{ __tera_context }}", &context)?;
    std::fs::write("context.json", output)?;
    Ok(())
}

fn get_open_api_content_and_doc(api: &PathBuf) -> anyhow::Result<String> {
    let mut contents = String::new();
    let t = if api.is_file() {
        let parent_path = api.parent().unwrap();
        let shared_yml = parent_path.join("shared_models.yml");
        let shared_yaml = parent_path.join("shared_models.yaml");
        if shared_yml.exists() {
            info!("Merging with shared_models.yml OpenAPI document");
            let content = vec![
                std::fs::read_to_string(api)?,
                std::fs::read_to_string(shared_yml)?,
            ];
            contents = merge(content);
            let merged_file = temp_file::with_contents(contents.as_bytes());
            info!("Parsing OpenAPI document");
            SparseRoot::new_from_file(merged_file.path().to_path_buf())
        } else if shared_yaml.exists() {
            info!("Merging with shared_models.yaml OpenAPI document");
            let content = vec![
                std::fs::read_to_string(api)?,
                std::fs::read_to_string(shared_yml)?,
            ];
            contents = merge(content);
            let merged_file = temp_file::with_contents(contents.as_bytes());
            info!("Parsing OpenAPI document");
            SparseRoot::new_from_file(merged_file.path().to_path_buf())
        } else {
            let path = std::env::current_dir().unwrap().join(api);
            contents = std::fs::read_to_string(path.clone())?;
            info!("Parsing OpenAPI document");
            SparseRoot::new_from_file(path)
        }
    } else {
        let mut files = find_files(api, OsStr::new("yml"));
        files.append(&mut find_files(api, OsStr::new("yaml")));
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
    Ok(contents)
}

fn generate(args: GenerateArgs) -> anyhow::Result<()> {
    terminal_setup(args.quiet, args.verbose)?;

    if !args.api.exists() {
        return Err(anyhow::anyhow!("OpenAPI file(s) not found"));
    }

    let contents = get_open_api_content_and_doc(&args.api)?;

    let mut template = serde_openapi(contents)?;
    for e in &mut template.endpoints {
        e.flatten_requests();
        e.flatten_responses();
    }
    template.combine_requests();
    template.combine_responses();

    // sparse_openapi(doc)?;
    // TODO template dir and files
    let mut tera = match Tera::new("templates/**/*.*") {
        // TODO templates part of config or default
        Ok(t) => t,
        Err(e) => {
            error!("Parsing error(s): {}", e);
            ::std::process::exit(1);
        }
    };
    register_all(&mut tera);
    let config = parse_config_file(args.config)?;
    config.validate()?;
    tera.register_function("map_type", map_type_new(config.clone()));
    tera.register_function("extended", extended(config.extended.clone()));
    tera.register_function("exists", exists(config.extended.clone()));
    let context = Context::from_serialize(&template)?;
    // TODO render all files in dir
    // General render section
    // let output = tera.render("service.dart", &context)?;
    // std::fs::write(&args.output, output)?;
    // // Model section with multiple outputs
    // let parent = args.output.parent().unwrap();
    // for request in &template.requests {
    //     if request.name != "Array" {
    //         let output_file_name =
    //             tera.render_str(&config.model_file_name.clone().unwrap(), &context)?;
    //         let mut context = Context::from_serialize(&request)?;
    //         context.insert("file_name", &output_file_name);
    //         let output = tera.render("model.dart", &context)?;
    //         std::fs::write(parent.join(output_file_name), output)?;
    //     }
    // }

    let template_dir = Path::new("templates");
    let files = get_files(template_dir);

    for file in files {
        let file_name = file.file_name().unwrap().to_str().unwrap();
        if file_name.starts_with("model.") {
            // Renders all models and outputs multiple files
            info!("Rendering model files");
            for request in &template.requests {
                generate_model_file(request, &config.clone(), &mut tera, &args.output, file_name)?;
            }
            for response in &template.responses {
                generate_model_file(
                    response,
                    &config.clone(),
                    &mut tera,
                    &args.output,
                    file_name,
                )?;
            }
        } else if file_name.starts_with("model-endpoint.") {
            // Renders all models and outputs multiple files
            info!("Rendering model files");
            for endpoint in &template.endpoints {
                if !endpoint.flat_request.is_empty() && config.model_file_name.is_some() {
                    generate_endpoint_model_file(
                        &endpoint.flat_request,
                        &config,
                        &mut tera,
                        &args.output,
                        file_name,
                    )?;
                } else if !endpoint.flat_request.is_empty() && config.model_file_name.is_none() {
                    warn!("modelFileName is not set in config")
                }
                if !endpoint.flat_response.is_empty() && config.model_file_name.is_some() {
                    generate_endpoint_model_file(
                        &endpoint.flat_response,
                        &config,
                        &mut tera,
                        &args.output,
                        file_name,
                    )?;
                    // TODO setup a default is not set?
                } else if !endpoint.flat_response.is_empty() && config.model_file_name.is_none() {
                    warn!("modelFileName is not set in config")
                }
            }
        } else {
            // Normal file render with full context
            info!("Rendering file {:?}", file_name);
            let mut file_context = context.clone();
            file_context.insert("file_name", &file_name);
            let output = tera.render(file_name, &file_context)?;
            std::fs::write(&args.output.join(file.file_name().unwrap()), output)?;
        }
    }
    Ok(())
}

fn generate_model_file(
    structure: &DataStructure,
    config: &Config,
    tera: &mut Tera,
    output_folder: &PathBuf,
    file_name: &str,
) -> anyhow::Result<()> {
    if structure.name != "Array" {
        let mut model_context = Context::from_serialize(structure)?;
        let output_file_name =
            tera.render_str(&config.model_file_name.clone().unwrap(), &model_context)?;
        debug!("Generated file name: {:#?}", output_file_name);
        model_context.insert("file_name", &output_file_name);
        debug!("Context prepared: {:#?}", model_context);
        let output = tera.render(file_name, &model_context)?;
        debug!("Output rendered");
        // TODO retain folder structure
        std::fs::write(output_folder.join(output_file_name), output)?;
    }
    Ok(())
}

fn generate_endpoint_model_file(
    structure: &Vec<DataStructure>,
    config: &Config,
    tera: &mut Tera,
    output_folder: &PathBuf,
    file_name: &str,
) -> anyhow::Result<()> {
    let root = structure.iter().find(|x| x.is_root).unwrap();
    // TODO cater for nested arrays
    if root.property_type == "Array" && root.properties[0].property_type != "Object" {
        // Array of primitives
        return Ok(());
    }
    let root_model_context = Context::from_serialize(root)?;
    let output_file_name = tera.render_str(
        &config.model_file_name.clone().unwrap(),
        &root_model_context,
    )?;
    debug!("Generated file name: {:#?}", output_file_name);
    let mut context = Context::default();
    context.insert("file_name", &output_file_name);
    context.insert("models", &structure);
    let output = tera.render(file_name, &context)?;
    std::fs::write(output_folder.join(output_file_name), output)?;
    Ok(())
}

fn get_files(path: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    for entry in path.read_dir().unwrap().flatten() {
        if entry.path().is_dir() {
            files.append(&mut get_files(&entry.path()));
        } else if entry.path().is_file()
            && !entry
                .path()
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .starts_with('_')
        {
            files.push(entry.path().clone());
        }
    }
    files
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
