// src/main.rs
use clap::{App, Arg};
use std::collections::HashMap;
use std::env;
use tera::{Context, Tera};
mod plugin;
use plugin::{Plugin, PluginFunction};
mod error;
use error::panic_hook;
mod filter;

struct Args {
    envs: HashMap<String, String>,
    template: String,
    variables: Option<String>,
    plugin: Option<String>,
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

    Args {
        envs,
        template: matches.value_of("template").unwrap().to_string(),
        variables: matches.value_of("variables").map(|s| s.to_string()),
        plugin: matches.value_of("plugin").map(|s| s.to_string()),
    }
}

fn render_variables(
    tera: &mut Tera,
    variables_path: Option<&str>,
    context: &Context,
) -> HashMap<String, serde_yaml::Value> {
    if variables_path.is_none() {
        return HashMap::new(); // 바로 빈 HashMap을 반환
    }

    let path = variables_path.unwrap();
    let variables_content =
        std::fs::read_to_string(path).expect("Failed to read variables template file");
    tera.add_raw_template("variables", &variables_content)
        .expect("Failed to add variables template");

    let rendered_variables = tera
        .render("variables", context)
        .map_err(|e| format!("Failed to render variables template:{}", e))
        .unwrap();
    serde_yaml::from_str(&rendered_variables).expect("Failed to parse rendered variables")
}

fn render_template(tera: &mut Tera, template_path: &str, context: &Context) -> String {
    let template_content =
        std::fs::read_to_string(template_path).expect("Failed to read template file");
    tera.add_raw_template(template_path, &template_content)
        .expect("Failed to add template");

    tera.render(template_path, context).unwrap()
}

fn main() {
    panic_hook();

    let args = parse_arguments();
    let mut tera = Tera::default();
    let mut context = Context::new();

    filter::register_filters(&mut tera);
    let mut global_vars: HashMap<String, serde_yaml::Value> = args
        .envs
        .iter()
        .map(|(k, v)| (k.clone(), serde_yaml::Value::String(v.clone())))
        .collect();
    context.insert("vars", &global_vars);

    if let Some(plugin_path) = &args.plugin {
        let plugins = Plugin::load_from_file(plugin_path);
        for (name, plugin) in plugins.into_iter() {
            // 플러그인 등록 논리 (Tera에 custom function 등록)
            let plugin_function = PluginFunction {
                name: name.clone(),
                params: plugin.params,
                script: plugin.script,
            };
            tera.register_function(&name, plugin_function);
            println!("Loaded and registered plugin: {}", name);
        }
    }

    // Render variables
    let rendered_vars = render_variables(&mut tera, args.variables.as_deref(), &context);

    global_vars.extend(rendered_vars);

    let mut context = Context::new();
    context.insert("vars", &global_vars);

    // Render main template
    println!("try main: {}", args.template);
    let rendered = render_template(&mut tera, &args.template, &context);
    println!("{}", rendered);
}
