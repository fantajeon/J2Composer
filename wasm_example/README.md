# Walkthrough

This guide will walk you through the creation of a WebAssembly (Wasm) plugin using Rust. For a more detailed example, refer to the `wasm_example` project.

## 1. Set Up Your Rust Library Project

Your Rust project should be set up as a library. This means your main file should be `lib.rs`. If you've initialized your project using `cargo new`, ensure you use the `--lib` flag:

```bash
cargo new your_project_name --lib
cd your_project_name
```

## 2. Update Cargo.toml

Add the necessary dependencies and library paths to your `Cargo.toml`:

```toml
[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1"
plugin_macro = { git = "https://github.com/fantajeon/jintemplify-plugin", package = "plugin_macro", version = "*" }
plugin = { git = "https://github.com/fantajeon/jintemplify-plugin", package = "plugin", version = "*" }

```

## 3. Implement Your Plugin in lib.rs

Inside your `lib.rs` file, start defining your plugin:

```rust
// Import the required macro for Wasm plugin functionality.
plugin::guest_plugin!();

// Define your input type.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct YourInputType {
    // ... your input fields here ...
}

// Define your return type.
#[allow(dead_code)]
#[derive(Serialize)]
pub struct YourReturnType {
    // ... your return fields here ...
}

// Define your plugin function.
#[plugin_macro::plugin_function]
pub fn your_plugin(input: YourInputType) -> YourReturnType {
    // ... your plugin logic here ...
}

// Define an additional input type, if needed.
#[allow(dead_code)]
#[derive(Deserialize)]
pub struct YourValueInputType {
    // ... your additional input fields here ...
}

#[allow(dead_code)]
#[derive(Deserialize)]
YourParameterType {
   // ... your additional input fields here ...
}

// Define your plugin filter, if required.
#[plugin_macro::plugin_filter]
pub fn your_plugin_filter(value: YourValueInputType, input: YourParameterType) -> YourReturnType {
    // ... your filter logic here ...
}

```

Make sure to adjust the fields in the structs (`YourInputType`, `YourReturnType`, `YourValueInputType`, `YourParameterType`) to match the specific needs of your project. When structuring these data types, always consider the type and structure of data you expect to work with within the context of the Jinja template.

`YourReturnType` can be a complex structure like a struct, or it can be a primitive type such as `i32`, `String`, etc. This gives you the flexibility to decide how you want to structure the response from your plugin. Importantly, ensure that `YourReturnType` has a JSON serialization implementation so that its data can be seamlessly integrated back into the Jinja environment after processing.

### For the function plugin:

- **YourInputType**: This is the primary data type you'll use to receive information from the Jinja template. It should be structured to capture all the data you expect to be passed to your plugin. Ensure that each field in this struct corresponds to an expected piece of data from your template.

In Jinja, when you call `your_function(val1, val2)`, `val1` and `val2` will correspond to members of `YourInputType`.

### For the filter function plugin:

- **YourValueInputType**: When using a filter in a Jinja template, this type represents the immediate result or value that's passed to your filter. It's essentially the current value in the pipeline that you want to process or modify.

- **YourParameterType**: Filters in Jinja can take additional arguments to influence their behavior. This type is designed to capture those arguments. When defining this, think about any additional parameters your filter might need to function correctly.

In practice, when you use a filter in Jinja, it works somewhat like this: `value|your_filter(arg1="val1", arg2="val2")`, and `value` is corresponed to `YourValueInputType`, `arg1` and `arg2` corresponds to a member of `YourParameterType`.

Remember to keep these data structures lean and only include fields that are essential to your plugin's operation. This ensures flexible data transfer (through `JSON-serialization`) and efficient processing within the Wasm environment.

## 4. Compile Wasm

To compile your project to WebAssembly, you'll need to add the `wasm32-unknown-unknown` target architecture:

```bash
rustup target add wasm32-unknown-unknown
```

Next, compile your project:

```bash
cargo build --release --target wasm32-unknown-unknown
```

Note: After compiling, your WebAssembly output (.wasm file) will be located in the target/wasm32-unknown-unknown/release/ directory.

## 5. Write plugin.yaml.j2 and tester.j2 with jintemplify

### plugin.yaml.j2

This configuration file specifies the details and the metadata about your WebAssembly plugin functions and filters:

```yaml
function:
    - name: your_function
    params:
      - name: var1
      - name: var2
    wasm:
      path: ./wasm_example/target/wasm32-unknown-unknown/release/wasm_example.wasm
      import: your_function
filters:
    - name: your_filter
        params:
        - name: var1
        - name: var2
        wasm:
        path: ./wasm_example/target/wasm32-unknown-unknown/release/wasm_example.wasm
        import: your_filter
```

### tester.j2

This is a Jinja template to test the functionality of your WebAssembly plugin:

```jinja
{{your_function(var1="...",var2="...")}}
{{"value1" | your_filter(var1="...", var2="...")}}
```

Here:

{{your_function(var1="...",var2="...")}} tests the function plugin by invoking your_function with specified parameters.

{{"value1" | your_filter(var1="...", var2="...")}} tests the filter function plugin, where "value1" is the value that's processed by your filter.

For a more comprehensive understanding, and to see real-world examples, navigate to the examples/scratch directory in the project.

## Further Reading

For an in-depth look at the creation and usage of the Wasm plugin, please refer to the `src/lib.rs` file inside the `wasm_example` project.
