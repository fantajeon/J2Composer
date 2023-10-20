use crate::ast::{Executable, WasmDeclartion, WasmFilter, WasmFunction};
use jintemplify_plugin;
use log::{debug, info};
use std::collections::HashMap;
use std::slice;
use std::str;
use tera;
use wasmtime::*;

jintemplify_plugin::host_plugin!();

impl Executable for WasmFunction {
    fn execute(
        &self,
        args: &HashMap<String, tera::Value>,
        _value: Option<&tera::Value>,
    ) -> tera::Result<tera::Value> {
        let mut execute =
            WasmExecutor::new(&self.decl).map_err(|e| tera::Error::msg(e.to_string()))?;
        Ok(execute
            .execute(args, None)
            .map_err(|e| tera::Error::msg(e.to_string()))?)
        //Ok(execute_wasm(&self.decl, args, None).map_err(|e| tera::Error::msg(e.to_string()))?)
    }
}

impl WasmDeclartion {
    fn filter_params(
        &self,
        user_params: &HashMap<String, serde_json::Value>,
    ) -> HashMap<String, serde_json::Value> {
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

pub struct WasmExecutor<'a> {
    func_decl: &'a WasmDeclartion,
    store: Store<()>,
    instance: Instance,
}

fn get_module(engine: &Engine, file: &str) -> Result<Module, anyhow::Error> {
    let module = Module::from_file(engine, file)?;
    Ok(module)
}

fn get_imports<T>(store: &mut Store<T>) -> Vec<Extern> {
    let print_type = wasmtime::FuncType::new(
        [wasmtime::ValType::I32, wasmtime::ValType::I32]
            .iter()
            .cloned(),
        [].iter().cloned(),
    );
    let print_func = wasmtime::Func::new(
        store,
        print_type,
        |mut caller: Caller<'_, T>, params: &[wasmtime::Val], _results: &mut [wasmtime::Val]| {
            let mem = match caller.get_export("memory") {
                Some(Extern::Memory(mem)) => Ok(mem),
                _ => Err(anyhow::anyhow!("failed to find host memory")),
            }
            .unwrap();
            let data = mem
                .data(&caller)
                .get(params[0].unwrap_i32() as usize..)
                .and_then(|arr| arr.get(..params[1].unwrap_i32() as usize));

            // Read the string from the WebAssembly memory
            let string = match data {
                Some(data) => match str::from_utf8(data) {
                    Ok(s) => Ok(s),
                    Err(_) => Err(anyhow::anyhow!("invalid utf-8")),
                },
                None => Err(anyhow::anyhow!("invalid utf-8")),
            }?;

            info!("{}", string);
            Ok(())
        },
    );
    vec![wasmtime::Extern::Func(print_func.into())]
}

impl<'a> WasmExecutor<'a> {
    pub fn new(func_decl: &'a WasmDeclartion) -> Result<Self> {
        let engine = Engine::default();
        let module = get_module(&engine, &func_decl.wasm.path)?;
        let mut store = Store::new(&engine, ());
        let imports = get_imports(&mut store);
        let instance = Instance::new(&mut store, &module, &imports)?;
        Ok(Self {
            func_decl,
            store: store,
            instance,
        })
    }

    fn free_return_values(&mut self, ptr: u32) {
        let guest_free = self
            .instance
            .get_typed_func::<u32, ()>(&mut self.store, "guest_free")
            .unwrap();

        guest_free.call(&mut self.store, ptr as u32).unwrap();
    }

    fn prepare_input_data(
        &self,
        arg: &HashMap<String, tera::Value>,
        value: Option<&tera::Value>,
    ) -> serde_json::Value {
        let params = match &value {
            Some(v) => vec![serde_json::json!(v), serde_json::json!(arg)],
            None => vec![serde_json::json!(arg)],
        };
        serde_json::json!(jintemplify_plugin::InputWrapper { params: params })
    }

    pub fn execute(
        &mut self,
        arg: &HashMap<String, tera::Value>,
        value: Option<&tera::Value>,
    ) -> anyhow::Result<tera::Value> {
        let arg = self.func_decl.filter_params(arg);
        let input_data = self.prepare_input_data(&arg, value);
        let input_bytes = input_data.to_string().into_bytes();
        let memory = self.instance.get_memory(&mut self.store, "memory").unwrap();

        // Allocate space in wasm memory for the JSON input
        let input_ptr = memory.data_mut(&mut self.store).len() as i32;
        memory.grow(&mut self.store, (input_bytes.len() as u32).into())?;
        memory.data_mut(&mut self.store)
            [input_ptr as usize..input_ptr as usize + input_bytes.len()]
            .copy_from_slice(&input_bytes);

        let function = self
            .instance
            .get_typed_func::<(u32, u32), u32>(&mut self.store, &self.func_decl.wasm.import)?;

        let ptr = function.call(
            &mut self.store,
            (input_ptr as u32, input_bytes.len() as u32),
        )?;

        let result_ptr = ptr as usize;
        let result_len = std::mem::size_of::<jintemplify_plugin::ReturnValues>();

        let memory_slice = unsafe {
            std::slice::from_raw_parts_mut(
                memory.data_mut(&mut self.store)[result_ptr..].as_mut_ptr(),
                result_len,
            )
        };

        let return_values: &mut jintemplify_plugin::ReturnValues =
            unsafe { &mut *(memory_slice.as_mut_ptr() as *mut jintemplify_plugin::ReturnValues) };

        debug!(
            "return_values={}, len={}",
            (*return_values).ptr,
            (*return_values).len
        );
        let result_ptr = (*return_values).ptr as usize;
        let result_len = (*return_values).len as usize;

        // Extract the result string
        let result_str = unsafe {
            let result_bytes =
                slice::from_raw_parts(memory.data(&self.store)[result_ptr..].as_ptr(), result_len);
            std::str::from_utf8(result_bytes)?
        };

        self.free_return_values(ptr);

        let output: jintemplify_plugin::OutputWrapper = match serde_json::from_str(result_str) {
            Ok(val) => val,
            Err(err) => return Err(anyhow::anyhow!(err)),
        };

        debug!("jintemplify_plugin::OutputWrapper :{:?}", output);
        Ok(output.result)
    }
}

impl Executable for WasmFilter {
    fn execute(
        &self,
        args: &HashMap<String, tera::Value>,
        value: Option<&tera::Value>,
    ) -> tera::Result<tera::Value> {
        //Ok(execute_wasm(&self.decl, args, value).map_err(|e| tera::Error::msg(e.to_string()))?)
        let mut execute =
            WasmExecutor::new(&self.decl).map_err(|e| tera::Error::msg(e.to_string()))?;
        Ok(execute
            .execute(args, value)
            .map_err(|e| tera::Error::msg(e.to_string()))?)
    }
}
