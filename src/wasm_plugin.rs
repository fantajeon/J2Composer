use crate::ast::{Executable, Param};
use wasmtime::*;
use std::collections::HashMap;
use tera;
use serde::Deserialize;
use std::str;
use std::slice;
use plugin;

plugin::host_plugin!();

#[derive(Debug, Deserialize, Clone)]
pub struct Wasm {
    pub path: String,
    pub import: String,
}

pub struct WasmDeclartion {
    pub wasm: Wasm,
    pub params: Option<Vec<Param>>,
}

pub struct WasmFunction {
    pub decl: WasmDeclartion,
}

pub struct WasmFilter {
    pub decl: WasmDeclartion,
}

impl Executable for WasmFunction {
    fn execute(&self, args: &HashMap<String, tera::Value>, _value: Option<&tera::Value>) -> tera::Result<String> {
        execute_wasm(&self.decl, args).map_err(|e| tera::Error::msg(e.to_string()))
    }
}

impl WasmDeclartion {
    fn filter_params(&self, user_params: &HashMap<String, serde_json::Value>) -> HashMap<String, serde_json::Value> {
        let mut filtered_map = HashMap::new();

        // Iterate through each parameter defined in the function_config
        if let Some(params) = &self.params {
            for param in params {
                // Check if the parameter exists in the provided map
                if let Some(value) = user_params.get(&param.name) {
                    filtered_map.insert(param.name.clone(), value.clone());
                }
            }
        }

        filtered_map
    }
}

fn execute_wasm(
    func_decl: & WasmDeclartion,
    arg: &HashMap<String, tera::Value>,
) -> anyhow::Result<String> {
    let mut store: Store<()> = Store::default();
    let print_func = wasmtime::Func::wrap(&mut store, |mut caller: Caller<'_, ()>, ptr: i32, len: i32| {

        let mem = match caller.get_export("memory") {
            Some(Extern::Memory(mem)) => mem,
            _ => anyhow::bail!("failed to find host memory"),
        };
        let data = mem.data(&caller)
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
    });
    let module = Module::from_file(store.engine(), &func_decl.wasm.path)?;
    
    let instance = Instance::new(&mut store, &module, &[print_func.into()])?;
    let arg = func_decl.filter_params(arg);

    let input_data = serde_json::json!(arg);
    let input_bytes = input_data.to_string().into_bytes();

    let memory = instance.get_memory(&mut store, "memory").unwrap();

    // Allocate space in wasm memory for the JSON input
    let input_ptr = memory.data_mut(&mut store).len() as i32;
    memory.grow(&mut store, (input_bytes.len() as u32).into())?;
    memory.data_mut(&mut store)[input_ptr as usize..input_ptr as usize + input_bytes.len()]
        .copy_from_slice(&input_bytes);

    let function = instance
        .get_func(&mut store,  &func_decl.wasm.import)
        .ok_or_else(|| anyhow::anyhow!("Failed to find function: combine_strings"))?;
        
    let mut results = vec![Val::I32(0)];

    println!("run funciton.call");
    function.call(&mut store, &[Val::I32(input_ptr as i32), Val::I32(input_bytes.len() as i32)], &mut results)?;

    let result_ptr = results[0].unwrap_i32() as usize;
    let result_len = std::mem::size_of::<plugin::ReturnValues>();

    let memory_slice = unsafe {
        std::slice::from_raw_parts(
            memory.data(&store)[result_ptr..].as_ptr(),
            result_len,
        )
    };

    let return_values: &plugin::ReturnValues = unsafe {
        &*(memory_slice.as_ptr() as *const plugin::ReturnValues)
    };

    println!("return_values={}, len={}", return_values.ptr, return_values.len);
    let result_ptr = return_values.ptr as usize;
    let result_len = return_values.len as usize;

    // Extract the result string
    let result_str = unsafe {
        let result_bytes = slice::from_raw_parts(
            memory.data(&store)[result_ptr..].as_ptr(),
            result_len,
        );
        std::str::from_utf8(result_bytes)?
    };

    Ok(result_str.to_string())
}

impl Executable for WasmFilter {
    fn execute(&self, args: &HashMap<String, tera::Value>, _value: Option<&tera::Value>) -> tera::Result<String> {
        //execute_wasm_function(&self, args).map_err(|e| tera::Error::msg(e.to_string()))
        unimplemented!()
    }
}
