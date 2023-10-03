// src/plugin.rs
use crate::render::render_template;
use anyhow::{self, Context as _Context};
use log::{debug, error, info};
use serde::Deserialize;
use std::collections::HashMap;
use std::process::Command;
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

impl Function for FunctionDeclartion {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        debug!("function call: {}, params={:?}", self.name, args);
        let mut cmd = self.script.clone();

        if let Some(params) = &self.params {
            for param in params {
                if !args.contains_key(&param.name) && param.default.is_none() {
                    error!(
                        "function call: {}, not provided param={}",
                        self.name, param.name
                    );
                    return Err(tera::Error::msg(format!(
                        "Parameter '{}' not provided for function '{}' and no default value is set.",
                        param.name, self.name
                    )));
                }

                let placeholder = format!("$({})", param.name);
                let value_str = match args.get(&param.name) {
                    Some(tera::Value::String(s)) => s.clone(),
                    Some(v) => v.to_string(),
                    None => param.default.as_ref().unwrap().clone(),
                };
                cmd = cmd.replace(&placeholder, &value_str);
            }
        }

        debug!("shell command: {}, env: {:?}", cmd, self.env);
        let mut shell_cmd = Command::new("sh");
        shell_cmd.arg("-c").arg(&cmd);
        if let Some(env_vars) = &self.env {
            for (key, value) in env_vars.iter() {
                shell_cmd.env(key, value);
            }
        }

        let output = shell_cmd.output();

        match &output {
            Ok(o) if o.status.success() => {
                info!("Command executed successfully: {} => {:?}", cmd, o);
                let output_str = String::from_utf8_lossy(&o.stdout);
                debug!("{} => output_str: {}", self.name, output_str.to_string());
                Ok(tera::Value::String(output_str.to_string()))
            }
            Ok(o) => {
                error!("Command failed: {} => {:?}", cmd, o);
                Err(tera::Error::msg(format!(
                    "Failed to execute command '{}': {}",
                    cmd,
                    String::from_utf8_lossy(&o.stderr)
                )))
            }
            Err(e) => {
                error!("Error executing command: {}", e);
                Err(tera::Error::msg(format!(
                    "Failed to execute command '{}': {}",
                    cmd, e
                )))
            }
        }
    }
}

fn run_with_shebang(
    cmd: &str,
    env_vars: Option<&HashMap<String, String>>,
) -> Result<Command, std::io::Error> {
    let lines: Vec<&str> = cmd.split('\n').collect();
    let interpreter = if lines[0].starts_with("#!") {
        &lines[0][2..]
    } else {
        "sh"
    };

    let mut command = Command::new(interpreter);
    command.arg("-c").arg(cmd);

    if let Some(envs) = env_vars {
        for (key, value) in envs.iter() {
            command.env(key, value);
        }
    }

    command.spawn()?.wait()?;
    Ok(command)
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
        let mut cmd = self.script.clone();

        if let Some(params) = &self.params {
            let default_value = tera::Value::String("".to_string());
            for param in params {
                // input은 value로 직접 처리되므로 이를 건너뛰기
                if param.name == "input" {
                    continue;
                }

                let arg_value = args.get(&param.name).unwrap_or_else(|| &default_value);

                let value_str = match arg_value {
                    tera::Value::String(s) => s.clone(),
                    v => v.to_string(),
                };

                let placeholder = format!("$({})", param.name);
                cmd = cmd.replace(&placeholder, &value_str);
            }
            let input_str = value.to_string();
            cmd = cmd.replace("$(input)", &input_str);
        }

        debug!("shell command: {}, env: {:?}", cmd, self.env);
        let mut shell_cmd = run_with_shebang(&cmd, self.env.as_ref())
            .map_err(|e| tera::Error::msg(format!("Failed to execute command '{}': {}", cmd, e)))?;

        let output = shell_cmd.output();

        match &output {
            Ok(o) if o.status.success() => {
                info!("Command executed successfully: {} => {:?}", cmd, o);
                let output_str = String::from_utf8_lossy(&o.stdout);
                debug!("{} => output_str: {}", self.name, output_str.to_string());
                Ok(tera::Value::String(output_str.to_string()))
            }
            Ok(o) => {
                error!("Command failed: {} => {:?}", cmd, o);
                Err(tera::Error::msg(format!(
                    "Failed to execute command '{}': {}",
                    cmd,
                    String::from_utf8_lossy(&o.stderr)
                )))
            }
            Err(e) => {
                error!("Error executing command: {}", e);
                Err(tera::Error::msg(format!(
                    "Failed to execute command '{}': {}",
                    cmd, e
                )))
            }
        }
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
