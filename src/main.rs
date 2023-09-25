use clap::{App, Arg};
use std::collections::HashMap;
use tera::{Context, Tera};

struct Args {
    envs: HashMap<String, String>,
    template: Option<String>,
    variables: Option<String>,
}

fn parse_arguments() -> Args {
    let matches = App::new("j2yaml-composer")
        .arg(
            Arg::with_name("env")
                .short("e")
                .long("env")
                .multiple(true)
                .takes_value(true)
                .help("Environment variables in the format key=value"),
        )
        .arg(
            Arg::with_name("template")
                .short("t")
                .long("template")
                .takes_value(true)
                .help("Template file"),
        )
        .arg(
            Arg::with_name("variables")
                .short("v")
                .long("variables")
                .takes_value(true)
                .help("Variables file"),
        )
        .get_matches();

    let mut envs = HashMap::new();

    if let Some(values) = matches.values_of("env") {
        for value in values {
            let parts: Vec<&str> = value.splitn(2, '=').collect();
            if parts.len() == 2 {
                envs.insert(parts[0].to_string(), parts[1].to_string());
            } else {
                eprintln!(
                    "Warning: Invalid format for --env '{}'. Expected format is key=value",
                    value
                );
            }
        }
    }

    Args {
        envs,
        template: matches.value_of("template").map(|s| s.to_string()),
        variables: matches.value_of("variables").map(|s| s.to_string()),
    }
}

fn render_variables(tera: &mut Tera, variables_path: &str) -> HashMap<String, serde_yaml::Value> {
    let variables_content =
        std::fs::read_to_string(variables_path).expect("Failed to read variables template file");
    tera.add_raw_template("variables", &variables_content)
        .expect("Failed to add variables template");

    let rendered_variables = tera
        .render("variables", &Context::new())
        .expect("Failed to render variables template");
    serde_yaml::from_str(&rendered_variables).expect("Failed to parse rendered variables")
}

fn render_template(tera: &mut Tera, template_path: &str, context: &Context) -> String {
    let template_content =
        std::fs::read_to_string(template_path).expect("Failed to read template file");
    tera.add_raw_template(template_path, &template_content)
        .expect("Failed to add template");

    tera.render(template_path, context)
        .expect(format!("Failed to render template:{}", template_path).as_str())
}
fn main() {
    let args = parse_arguments();

    let mut tera = Tera::default();
    let mut context = Context::new();

    context.insert("envs", &args.envs);

    // Render variables
    let global_vars = render_variables(
        &mut tera,
        &args
            .variables
            .unwrap_or_else(|| "./template/variables.yaml.j2".to_string()),
    );
    context.insert("vars", &global_vars);

    // Render main template
    let rendered = render_template(
        &mut tera,
        &args
            .template
            .unwrap_or_else(|| "./template/main.yaml.j2".to_string()),
        &context,
    );
    println!("{}", rendered);
}
