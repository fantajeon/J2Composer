// src/plugin.rs
use crate::ast::{
    Executable, ExecutableFunction, FilterDeclaration, FunctionDeclaration, WasmDeclartion,
    WasmFilter, WasmFunction,
};
use crate::render::render_template;
use crate::shell_plugin::{ShellCommand, ShellFilter, ShellFunction};
use anyhow::{self, Context as _Context};
use log::debug;
use serde::Deserialize;
use std::collections::HashMap;
use tera::{Context, Filter, Function, Tera};

impl FunctionDeclaration {
    pub fn create(&self) -> anyhow::Result<ExecutableFunction> {
        let executor: Box<dyn Executable> = if let Some(wasm_config) = &self.wasm {
            Box::new(WasmFunction {
                decl: WasmDeclartion {
                    wasm: wasm_config.clone(),
                    params: self.params.clone(),
                },
            })
        } else {
            Box::new(ShellFunction {
                command: ShellCommand {
                    script: self.script.as_ref().unwrap().clone(),
                    params: self.params.clone(),
                    env: self.env.clone(),
                },
            })
        };

        Ok(ExecutableFunction {
            executor,
            name: self.name.clone(),
        })
    }
}
impl Function for ExecutableFunction {
    fn call(&self, args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        debug!("function call: {}, params={:?}", self.name, args);
        let result = self.executor.execute(args, None)?;
        Ok(result)
    }
}

impl FilterDeclaration {
    pub fn create(&self) -> anyhow::Result<ExecutableFilter> {
        let executor: Box<dyn Executable> = match (&self.wasm, &self.script) {
            (Some(wasm_config), _) => Box::new(WasmFilter {
                decl: WasmDeclartion {
                    wasm: wasm_config.clone(),
                    params: self.params.clone(),
                },
            }),
            (None, Some(script)) => Box::new(ShellFilter {
                command: ShellCommand {
                    script: script.clone(),
                    params: self.params.clone(),
                    env: self.env.clone(),
                },
            }),
            (None, None) => {
                return Err(anyhow::anyhow!(
                    "Neither wasm nor script configurations were provided"
                ));
            }
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
        Ok(result)
    }
}

#[derive(Debug, Deserialize)]
pub struct Plugin {
    pub functions: Option<Vec<FunctionDeclaration>>,
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
    use crate::ast::Param;
    use tera::Value;

    #[test]
    fn test_plugin_function_call() {
        let func_decl = FunctionDeclaration {
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
