use clap::{App, Arg};
use std::collections::HashMap;
use tera::{Context, Tera};

struct Args {
    envs: HashMap<String, String>,
    template: String,
    variables: Option<String>,
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
        template: matches.value_of("template").unwrap().to_string(),
        variables: matches.value_of("variables").map(|s| s.to_string()),
    }
}

fn render_variables(
    tera: &mut Tera,
    variables_path: Option<&str>,
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
    let global_vars = render_variables(&mut tera, args.variables.as_deref());
    context.insert("vars", &global_vars);

    // Render main template
    let rendered = render_template(&mut tera, &args.template, &context);
    println!("{}", rendered);
}
