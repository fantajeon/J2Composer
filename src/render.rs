use anyhow::{self, Context as _Context};
use tera::{Context, Tera};
use std::collections::HashMap;

pub fn render_template(tera: &mut Tera, template_path: &str, context: &Context) -> anyhow::Result<String> {
    let template_content =
        std::fs::read_to_string(template_path).context("Failed to read template file")?;
    tera.add_raw_template(template_path, &template_content)
        .context("Failed to add template")?;

    tera.render(template_path, context).with_context(|| format!("Failed to render template: {}", template_path))
}


pub fn render_variables(
    tera: &mut Tera,
    variables_path: Option<&str>,
    context: &Context,
) -> anyhow::Result<HashMap<String, serde_yaml::Value>> {
    if variables_path.is_none() {
        return Ok(HashMap::new()); // 바로 빈 HashMap을 반환
    }

    let path = variables_path.ok_or_else(|| anyhow::anyhow!("variables_path is None"))?;
    let variables_content =
        std::fs::read_to_string(path).context("Failed to read variables template file")?;
    tera.add_raw_template("variables", &variables_content)
        .with_context(|| format!("Failed to add variables template:{}", path))?;

    let rendered_variables = tera
        .render("variables", context)
        .with_context(|| format!("Failed to render variables template:{}", path))?;
    serde_yaml::from_str(&rendered_variables).context("Failed to parse rendered variables")
}