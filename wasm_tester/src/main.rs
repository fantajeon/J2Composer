// tester.rs
use anyhow::{self};
use plugin::ReturnValues;
use serde::{Deserialize, Serialize};
use serde_json;
use std::collections::HashMap;
use std::slice;
use std::str;
use tera;
use wasmtime::*;
#[macro_use]
extern crate maplit;

#[derive(Debug, Serialize, Deserialize)]
pub struct FunctionConfig {
    name: String,
    params: Vec<Param>,
    wasm: Wasm,
}

#[derive(Debug, Serialize, Deserialize)]
struct Wasm {
    path: String,
    import: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct Param {
    name: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct PluginConfig {
    function: Vec<FunctionConfig>,
}

plugin::host_plugin!();

fn load_config() -> anyhow::Result<PluginConfig> {
    let config_str = std::fs::read_to_string("plugin.yaml")?;
    let config: PluginConfig = serde_yaml::from_str(&config_str)?;
    Ok(config)
}

impl FunctionConfig {
    fn filter_params(
        &self,
        params: &HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
        let mut filtered_map = HashMap::new();

        // Iterate through each parameter defined in the function_config
        for param in &self.params {
            // Check if the parameter exists in the provided map
            if let Some(value) = params.get(&param.name) {
                filtered_map.insert(param.name.clone(), value.clone());
            }
        }

        filtered_map
    }
}
#[derive(serde::Serialize)]
struct InputParam {
    params: Vec<serde_json::Value>,
}
pub fn execute_wasm_function(
    config: &FunctionConfig,
    arg: &HashMap<String, tera::Value>,
) -> anyhow::Result<String> {
    let mut store: Store<()> = Store::default();
    let print_func = wasmtime::Func::wrap(
        &mut store,
        |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => mem,
                _ => anyhow::bail!("failed to find host memory"),
            };
            let data = mem
                .data(&caller)
                .get(ptr as u32 as usize..)
                .and_then(|arr| arr.get(..len as u32 as usize));

            // Read the string from the WebAssembly memory
            let string = match data {
                Some(data) => match str::from_utf8(data) {
                    Ok(s) => s,
                    Err(_) => anyhow::bail!("invalid utf-8"),
                },
                None => anyhow::bail!("pointer/length out of bounds"),
            };

            println!("{}", string);
            Ok(())
        },
    );
    let module = Module::from_file(store.engine(), &config.wasm.path)?;

    let instance = Instance::new(&mut store, &module, &[print_func.into()])?;
    //let instance = Instance::new(&mut store, &module, &[])?;
    let arg = config.filter_params(arg);

    let input_data = serde_json::json!(InputParam {
        params: vec![serde_json::json!(arg)]
    });
    let input_bytes = input_data.to_string().into_bytes();

    let memory = instance.get_memory(&mut store, "memory").unwrap();

    // Allocate space in wasm memory for the JSON input
    let input_ptr = memory.data_mut(&mut store).len() as i32;
    memory.grow(&mut store, (input_bytes.len() as u32).into())?;
    memory.data_mut(&mut store)[input_ptr as usize..input_ptr as usize + input_bytes.len()]
        .copy_from_slice(&input_bytes);

    let function = instance
        .get_func(&mut store, &config.wasm.import)
        .ok_or_else(|| anyhow::anyhow!("Failed to find function: combine_strings"))?;

    let mut results = vec![Val::I32(0)];

    println!("run funciton.call");
    function.call(
        &mut store,
        &[
            Val::I32(input_ptr as i32),
            Val::I32(input_bytes.len() as i32),
        ],
        &mut results,
    )?;

    let result_ptr = results[0].unwrap_i32() as usize;
    let result_len = std::mem::size_of::<ReturnValues>();

    let memory_slice = unsafe {
        std::slice::from_raw_parts(memory.data(&store)[result_ptr..].as_ptr(), result_len)
    };

    let return_values: &ReturnValues = unsafe { &*(memory_slice.as_ptr() as *const ReturnValues) };

    println!(
        "return_values={}, len={}",
        return_values.ptr, return_values.len
    );
    let result_ptr = return_values.ptr as usize;
    let result_len = return_values.len as usize;

    // Extract the result string
    let result_str = unsafe {
        let result_bytes =
            slice::from_raw_parts(memory.data(&store)[result_ptr..].as_ptr(), result_len);
        std::str::from_utf8(result_bytes)?
    };

    Ok(result_str.to_string())
}

fn main() {
    match load_config() {
        Ok(config) => {
            for function_config in config.function.iter() {
                println!("Executing function: {}", function_config.name);
                let map = hashmap! {"var1".to_string() => tera::Value::String("Hello".to_string()),
                "var2".to_string() => tera::Value::String(" World!".to_string()),
                };
                match execute_wasm_function(&function_config, &map) {
                    Ok(result) => println!("Result: {}", result),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        }
        Err(e) => eprintln!("Failed to load config: {}", e),
    }
}
