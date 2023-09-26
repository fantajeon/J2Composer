// src/plugin.rs
use serde::Deserialize;
use std::process::Command;
use std::{collections::HashMap};
use tera::{Function, Context, Tera};
use anyhow::{self, Context as _Context};
use crate::render::render_template;
use log::{info, error, debug};

#[derive(Debug, Deserialize)]
pub struct Param {
    pub name: String,
    pub description: String,
    pub default: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FunctionDeclartion {
    pub params: Vec<Param>,
    pub script: String,
}

#[derive(Debug, Deserialize)]
pub struct PluginFunction {
    pub name: String,
    pub params: Vec<Param>,
    pub script: String,
}

impl Function for PluginFunction {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        debug!("function call: {}, params={:?}", self.name, args);
        let mut cmd = self.script.clone();

        for param in &self.params {
            if !args.contains_key(&param.name) && param.default.is_none() {
                error!("function call: {}, not provided param={}", self.name, param.name);
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

        debug!("shell command: {}", cmd);
        let output = Command::new("sh").arg("-c").arg(&cmd).output();
        match &output {
            Ok(_) => info!("Command executed successfully: {}", cmd),
            Err(e) => error!("Error executing command: {}", e),
        }
        let output = output
            .map_err(|e| tera::Error::msg(format!("Failed to execute command '{}': {}", cmd, e)))?;

        let output_str = String::from_utf8_lossy(&output.stdout);
        Ok(tera::Value::String(output_str.to_string()))
    }
}

#[derive(Debug, Deserialize)]
pub struct Plugin {}

impl Plugin {
    pub fn load_from_file(path: &str, tera: &mut Tera, context: &Context) -> anyhow::Result<HashMap<String, FunctionDeclartion>> {
        let content = render_template(tera, path, context)?;
        serde_yaml::from_str(&content).context("Failed to parse plugin file")
    }
}
