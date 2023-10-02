// src/main.rs
use anyhow::{self};
use clap::{App, Arg};
use log::{debug, info};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use tera::{Context, Tera};
mod plugin;
use plugin::{Plugin, PluginFunction};
mod error;
use error::panic_hook;
mod filter;
mod render;
use render::{render_template, render_variables};
mod function;

struct Args {
    envs: HashMap<String, String>,
    template: String,
    variables: Option<String>,
    plugin: Option<String>,
    output_file: Option<String>,
    disable_builtin_functions: bool,
    include_dirs: Vec<(String, Option<String>)>,
}

fn parse_arguments() -> Args {
    let matches = App::new("jintemplify")
        .about("A tool to compose files using Jinja2 templates and YAML variables.")
        .long_about("jintemplify allows you to combine Jinja2 templates with YAML variables to produce files in any desired format. Use the --template argument to specify the main Jinja2 template and the --variables argument (optional) to specify the YAML variables template.")
        .arg(
            Arg::with_name("env")
                .short("e")
                .long("env")
                .multiple(true)
                .takes_value(true)
                .help("Environment variables in the format key=value"),
        )
        .arg(
            Arg::with_name("default-env")
                .long("default-env")
                .multiple(true)
                .takes_value(true)
                .help("Optional environment variables in the format key=default_value"),
        )
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("template")
                .required(true)
                .takes_value(true)
                .help("Template file: main.yaml.j2, main.txt.j2, main.json.j2"),
        )
        .arg(
            Arg::with_name("variables")
                .short("v")
                .long("variables")
                .takes_value(true)
                .help("Variables file: variables.yaml.j2"),
        )
        .arg(
            Arg::with_name("plugin")
                .short("p")
                .long("plugin")
                .takes_value(true)
                .help("Path to the plugin configuration: plugin.yaml"),
        )
        .arg(
            Arg::with_name("output_file")
                .long("output-file")
                .value_name("FILE")
                .help("Sets an output file, stdout if not set")
                .takes_value(true),
        )
        .arg(
            Arg::with_name("disable_builtin_functions")
                .long("disable-builtin-functions")
                .help("Disables the registration of built-in functions"),
        )
        .arg(
            Arg::with_name("include-dir")
                .long("include-dir")
                .multiple(true)
                .takes_value(true)
                .help("Include directory for templates. Format: /path/to/dir:alias or /path/to/dir. Use '{}' for direct naming without an alias."),
        )
        .get_matches();

    let mut envs = HashMap::new();
    for (key, value) in env::vars() {
        envs.insert(key, value);
    }

    if let Some(values) = matches.values_of("env") {
        for value in values {
            let parts: Vec<&str> = value.splitn(2, '=').collect();
            if parts.len() == 2 {
                let (key, val) = (parts[0].to_string(), parts[1].to_string());

                envs.insert(key, val);
            } else {
                eprintln!(
                    "Warning: Invalid format for --env '{}'. Expected format is key=value",
                    value
                );
            }
        }
    }

    if let Some(values) = matches.values_of("default-env") {
        for value in values {
            let parts: Vec<&str> = value.splitn(2, '=').collect();
            if parts.len() == 2 {
                let key = parts[0].to_string();
                envs.entry(key).or_insert_with(|| parts[1].to_string());
            } else {
                eprintln!(
                    "Warning: Invalid format for --default-env '{}'. Expected format is key=default_value",
                    value
                );
            }
        }
    }

    let include_dirs = matches
        .values_of("include-dir")
        .unwrap_or_default()
        .map(|s| {
            let parts: Vec<&str> = s.splitn(2, ':').collect();
            (parts[0].to_string(), parts.get(1).map(|&s| s.to_string()))
        })
        .collect();

    Args {
        envs,
        template: matches.value_of("template").unwrap().to_string(),
        variables: matches.value_of("variables").map(|s| s.to_string()),
        plugin: matches.value_of("plugin").map(|s| s.to_string()),
        output_file: matches.value_of("output_file").map(ToOwned::to_owned),
        disable_builtin_functions: matches.is_present("disable_builtin_functions"),
        include_dirs,
    }
}

fn add_templates_from_dir(tera: &mut Tera, dir: &Path, alias: Option<&str>) -> anyhow::Result<()> {
    let files: Vec<_> = fs::read_dir(dir)?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file())
        .collect();

    for file in files {
        let template_name = file.strip_prefix(dir).unwrap().to_str().unwrap();
        let template_name_with_alias = match alias {
            Some(alias_str) => format!("{}.{}", alias_str, template_name),
            None => template_name.to_string(),
        };
        debug!(
            "added tempate: {:?} => {:?}",
            template_name, template_name_with_alias
        );
        tera.add_template_files(vec![(file, Some(template_name_with_alias))])?;
    }

    Ok(())
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    panic_hook();

    let args = parse_arguments();
    let mut context = Context::new();
    let mut tera = Tera::default();
    for (dir_str, alias_opt) in args.include_dirs.iter() {
        let dir_path = Path::new(dir_str);
        let default_alias = dir_path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("")
            .to_string();

        let alias = match alias_opt {
            Some(alias_string) if alias_string == "{}" => None,
            Some(other_alias) => Some(other_alias.clone()),
            None => Some(default_alias),
        };

        info!("{:?} => {:?} from {:?}", dir_path, alias, alias_opt);

        add_templates_from_dir(&mut tera, dir_path, alias.as_deref())?;
    }

    info!(
        "tempate names: {:?}",
        tera.get_template_names().collect::<Vec<&str>>()
    );
    if !args.disable_builtin_functions {
        function::register_functions(&mut tera);
    }
    filter::register_filters(&mut tera);
    let mut global_vars: HashMap<String, serde_yaml::Value> = args
        .envs
        .iter()
        .map(|(k, v)| (k.clone(), serde_yaml::Value::String(v.clone())))
        .collect();
    context.insert("vars", &global_vars);

    if let Some(plugin_path) = &args.plugin {
        let plugins = Plugin::load_from_file(plugin_path, &mut tera, &context)?;
        for (name, plugin) in plugins.into_iter() {
            // Register Plugins
            let plugin_function = PluginFunction {
                name: name.clone(),
                params: plugin.params,
                script: plugin.script,
                env: plugin.env,
            };
            tera.register_function(&name, plugin_function);
            info!("register_function: {}", name);
        }
    }

    // Render variables
    let rendered_vars = render_variables(&mut tera, args.variables.as_deref(), &context)?;

    global_vars.extend(rendered_vars);

    let mut context = Context::new();
    context.insert("vars", &global_vars);

    // Render main template
    info!("try main: {}", args.template);
    let rendered = render_template(&mut tera, &args.template, &context)?;
    match &args.output_file {
        Some(output_path) => {
            std::fs::write(output_path, &rendered).expect("Failed to write to output file");
        }
        None => {
            println!("{}", rendered);
        }
    }

    Ok(())
}
