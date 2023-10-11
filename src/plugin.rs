// src/plugin.rs
use crate::command::execute_shell_command;
use crate::render::render_template;
use anyhow::{self, Context as _Context};
use log::{debug, error, info};
use serde::Deserialize;
use std::collections::HashMap;
use tera::{Context, Filter, Function, Tera};

#[derive(Debug, Deserialize)]
pub struct Param {
    pub name: String,
    pub description: Option<String>,
    pub default: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FunctionDeclartion {
    pub name: String,
    pub params: Option<Vec<Param>>,
    pub env: Option<HashMap<String, String>>,
    pub description: Option<String>,
    pub script: String,
}

#[derive(Debug, Deserialize)]
pub struct FilterDeclaration {
    pub name: String,
    pub params: Option<Vec<Param>>,
    pub env: Option<HashMap<String, String>>,
    pub description: Option<String>,
    pub script: String,
}

fn replace_placeholder(
    cmd: &mut String,
    param: &Param,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<()> {
    if !args.contains_key(&param.name) && param.default.is_none() {
        return Err(tera::Error::msg(format!(
            "Parameter '{}' not provided and no default value is set.",
            param.name
        )));
    }
    let placeholder = format!("$({})", param.name);
    let value_str = args
        .get(&param.name)
        .map(|v| v.to_string())
        .unwrap_or_else(|| param.default.as_ref().unwrap().clone());
    *cmd = cmd.replace(&placeholder, &value_str);
    Ok(())
}

fn prepare_command(
    script: &str,
    params: &Option<Vec<Param>>,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<String> {
    let mut cmd = script.clone().to_string();
    if let Some(parameters) = params {
        for param in parameters {
            replace_placeholder(&mut cmd, param, args)?;
        }
    }
    Ok(cmd)
}

fn prepare_command_filter(
    script: &str,
    params: &Option<Vec<Param>>,
    value: &tera::Value,
    args: &HashMap<String, tera::Value>,
) -> tera::Result<String> {
    let mut cmd = script.clone().to_string();
    if let Some(parameters) = params {
        for param in parameters {
            // input은 value로 직접 처리되므로 이를 건너뛰기
            if param.name != "input" {
                replace_placeholder(&mut cmd, param, args)?;
            } else {
                cmd = cmd.replace("$(input)", &value.to_string());
            }
        }
    }
    Ok(cmd)
}

impl Function for FunctionDeclartion {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        debug!("function call: {}, params={:?}", self.name, args);
        let cmd = prepare_command(&self.script, &self.params, args)?;
        let result = execute_shell_command(&cmd, &self.env, None)?;
        Ok(tera::Value::String(result))
    }
}

impl Filter for FilterDeclaration {
    fn filter(
        &self,
        value: &tera::Value,
        args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        debug!(
            "filter call: {}, params={:?}, value={:?}",
            self.name, args, value
        );
        let cmd = prepare_command_filter(&self.script, &self.params, value, args)?;
        let result = execute_shell_command(&cmd, &self.env, None)?;
        Ok(tera::Value::String(result))
    }
}

#[derive(Debug, Deserialize)]
pub struct Plugin {
    pub functions: Option<Vec<FunctionDeclartion>>,
    pub filters: Option<Vec<FilterDeclaration>>,
}

impl Plugin {
    pub fn load_from_file(
        path: &str,
        tera: &mut Tera,
        context: &Context,
    ) -> anyhow::Result<Plugin> {
        let content = render_template(tera, path, context)?;
        serde_yaml::from_str(&content).context("Failed to parse plugin file")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tera::Value;

    #[test]
    fn test_plugin_function_call() {
        let func = FunctionDeclartion {
            name: "echo_test".to_string(),
            params: Some(vec![Param {
                name: "msg".to_string(),
                description: Some("Echoes a message".to_string()),
                default: None,
            }]),
            description: None,
            env: None,
            script: "echo $(msg)".to_string(),
        };

        let mut args = HashMap::new();
        args.insert(
            "msg".to_string(),
            Value::String("Hello, world!".to_string()),
        );

        let result = func.call(&args).unwrap();
        assert_eq!(result, Value::String("Hello, world!\n".to_string()));
    }
}
