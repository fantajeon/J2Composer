// tester.rs
use anyhow::{self};
use jintemplify::{FilterDeclaration, FunctionDeclaration};
use serde::Deserialize;
use std::str;
use tera;
use tera::{Filter, Function};
#[macro_use]
extern crate maplit;

#[derive(Debug, Deserialize)]
struct PluginConfig {
    function: Vec<FunctionDeclaration>,
    filter: Vec<FilterDeclaration>,
}

jintemplify_plugin::host_plugin!();

fn load_config() -> anyhow::Result<PluginConfig> {
    let config_str = std::fs::read_to_string("plugin.yaml")?;
    let config: PluginConfig = serde_yaml::from_str(&config_str)?;
    Ok(config)
}

fn execute_function(config: &PluginConfig) {
    for func in config.function.iter() {
        println!("Executing function: {}", func.wasm.as_ref().unwrap().path);
        let execute = func.create().unwrap();
        let args = hashmap! {"var1".to_string() => tera::Value::String("Hello".to_string()),
        "var2".to_string() => tera::Value::String(" World!".to_string()),
        };
        match execute.call(&args) {
            Ok(result) => println!("Result: {}", result),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn execute_filter(config: &PluginConfig) {
    for filter in config.filter.iter() {
        println!("Executing filter: {}", filter.wasm.as_ref().unwrap().path);
        let execute = filter.create().unwrap();
        let args = hashmap! {"var1".to_string() => tera::Value::String("Hello".to_string()),
        "var2".to_string() => tera::Value::String(" World!".to_string()),
        };
        match execute.filter(&tera::Value::String("test2".to_string()), &args) {
            Ok(result) => println!("Result: {}", result),
            Err(e) => eprintln!("Error: {}", e),
        }
    }
}

fn main() {
    match load_config() {
        Ok(config) => {
            execute_function(&config);
            execute_filter(&config);
        }
        Err(e) => eprintln!("Failed to load config: {}", e),
    }
}
