use serde::{Deserialize, Serialize};

plugin::guest_plugin!();

#[allow(dead_code)]
#[derive(Deserialize)]
pub struct Input {
    pub var1: String,
    pub var2: String,
}

#[plugin_macro::plugin_function]
pub fn combine_strings(input: Input) -> String {
    // Deserialize the input
    let combined_result = format!("{}{}", input.var1, input.var2);

    send_log(&format!("{}", "Hello from Wasm!"));
    combined_result
}

#[plugin_macro::plugin_filter]
pub fn my_test_filter(value: String, input: Input) -> String {
    // Deserialize the input
    let combined_result = format!("filter: {} => {}{}", value, input.var1, input.var2);
    send_log(&format!("filter: {}", "Hello from Wasm!"));
    // Serialize the output
    combined_result
}
