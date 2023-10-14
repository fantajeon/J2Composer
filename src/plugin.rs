// src/plugin.rs
use crate::ast::{Executable,Param};
use crate::render::render_template;
use crate::wasm_plugin::{WasmFunction, Wasm, WasmDeclartion, WasmFilter};
use crate::shell_plugin::{ShellCommand, ShellFunction, ShellFilter};
use anyhow::{self, Context as _Context};
use log::{debug};
use serde::Deserialize;
use std::collections::HashMap;
use tera::{Context, Filter, Function, Tera};

#[derive(Debug, Deserialize)]
pub struct FunctionDeclartion {
    pub name: String,
    pub params: Option<Vec<Param>>,
    pub env: Option<HashMap<String, String>>,
    pub description: Option<String>,
    pub wasm: Option<Wasm>,
    pub script: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct FilterDeclaration {
    pub name: String,
    pub params: Option<Vec<Param>>,
    pub env: Option<HashMap<String, String>>,
    pub description: Option<String>,
    pub wasm: Option<Wasm>,
    pub script: String,
}

impl FunctionDeclartion {
    pub fn create(&self) -> anyhow::Result<ExecutableFunction> {
        let executor: Box<dyn Executable> = if let Some(wasm_config) = &self.wasm {
            Box::new(WasmFunction {
                decl: WasmDeclartion {
                    wasm: wasm_config.clone(),
                    params: self.params.clone(),
                }
            })
        } else {
            Box::new(ShellFunction {
                command: ShellCommand {
                    script: self.script.as_ref().unwrap().clone(),
                    params: self.params.clone(),
                    env: self.env.clone(),
                }
            })
        };

        Ok(ExecutableFunction {
            executor,
            name: self.name.clone(),
        })
    }
}

pub struct ExecutableFunction {
    executor: Box<dyn Executable>,
    name: String,
}

impl Function for ExecutableFunction {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        debug!("function call: {}, params={:?}", self.name, args);
        let result = self.executor.execute(args, None)?;
        Ok(tera::Value::String(result))
    }
}

impl FilterDeclaration {
    pub fn create(&self) -> anyhow::Result<ExecutableFilter> {
        let executor: Box<dyn Executable> = if let Some(wasm_config) = &self.wasm {
            Box::new(WasmFilter {
                decl: WasmDeclartion {
                    wasm: wasm_config.clone(),
                    params: self.params.clone(),
                }
            })
        } else {
            Box::new(ShellFilter {
                command: ShellCommand{
                    script: self.script.clone(),
                    params: self.params.clone(),
                    env: self.env.clone(),
                },
            })
        };

        Ok(ExecutableFilter {
            executor,
            name: self.name.clone(),
        })
    }
}

pub struct ExecutableFilter {
    executor: Box<dyn Executable>,
    name: String,
}

impl Filter for ExecutableFilter {
    fn filter(
        &self,
        value: &tera::Value,
        args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        debug!(
            "filter call: {}, params={:?}, value={:?}",
            self.name, args, value
        );
        let result = self.executor.execute(args, Some(value))?;
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
        let func_decl = FunctionDeclartion {
            name: "echo_test".to_string(),
            params: Some(vec![Param {
                name: "msg".to_string(),
                description: Some("Echoes a message".to_string()),
                default: None,
            }]),
            description: None,
            env: None,
            wasm: None,
            script: Some("echo $(msg)".to_string()),
        };

        let mut args = HashMap::new();
        args.insert(
            "msg".to_string(),
            Value::String("Hello, world!".to_string()),
        );

        let func = func_decl.create().unwrap();
        let result = func.call(&args).unwrap();
        assert_eq!(result, Value::String("Hello, world!\n".to_string()));
    }
}
