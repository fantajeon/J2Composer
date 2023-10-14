use serde::{Deserialize, Serialize};

plugin::guest_plugin!();

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Input {
    pub var1: String,
    pub var2: String,
}

#[allow(dead_code)]
#[derive(Serialize)]
pub struct Output {
    pub result: String,
    pub exception: Option<String>,
}

#[plugin_macro::plugin_function]
pub fn combine_strings(input: Input) -> Output {
    // Deserialize the input
    let combined_result = format!("{}{}", input.var1, input.var2);

    send_log(&format!("{}", "Hello from Wasm!"));
    // Serialize the output
    Output {
        result: combined_result,
        exception: None,
    }
}

#[plugin_macro::plugin_filter]
pub fn myindent(value: String, input: Input) -> Output {
    // Deserialize the input
    let combined_result = format!("{} => {}{}", value, input.var1, input.var2);
    send_log(&format!("{}", "Hello from Wasm!"));
    // Serialize the output
    Output {
        result: combined_result,
        exception: None,
    }
}
