# Jintemplify

`jintemplify` is a CLI(Command Line Interface) tool that enables users to combine Jinja2([rust Tera library](https://keats.github.io/tera/)) templates with YAML variables, producing files in any desired format, including Dockerfiles and Makefiles. Designed for flexibility, `jintemplify` seamlessly integrates with Jenkins, Tekton, and other CI(Continuous Integration) systems in modern DevOps workflows. The application also supports a plugin system based on shell scripts and Webassembler(WASM) modules, allowing users to extend its functionality with familiar scripting techniques.

## Features

- **Template Rendering**: Use Jinja2 templates to define the structure of your file.
- **Variable Support**: Combine your templates with YAML-defined variables.
- **Plugin System (Shell Script Based)**: Extend the application's functionality with custom shell-script-based plugins. This allows for a wide range of extensibility using familiar scripting methods and the performance benefits of Wasm modules.

# Advanced Templating Features

## Accessing Environment Variables and Command Line Arguments

Within Jinja2 templates, you can use the vars object to access environment variables or any values passed using the --env command line option. This provides a seamless way to incorporate dynamic values into your templates based on the environment or runtime conditions.

```jinja2
{{ vars.my_environment_variable }}
{{ vars.my_cli_argument }}
```

In the above example, `my_environment_variable` could be an environment variable, and `my_cli_argument` could be a value passed via `--env`.

## Plugins

For those looking to extend the application's functionality with plugins, here's the basic structure for the plugin configuration:

```yaml
functions:
  - name: "function_name"
    params:
      - name: parameter_name
    env:
      CC: clang
      MAKEVARS: ...
    script: your_shell_script_command_here
  - name: ...
    wasm:
      path: path/to/myplugin.wasm
      import: your_function
```

Within this configuration:

- `name`: Represents the name of the function you're adding, which can be invoked directly within your Jinja2 templates.
- `params`: Enumerates the parameters your function or filter mandates. For filters, access the default input (symbolizing the value being filtered) using `$(input)`.
- `env`: Dictates environment variables to which the shell command will have access upon execution. This is pivotal when tailoring the behavior of your scripts contingent on the environment.
- `script`: Incorporates the shell command that gets executed when the function is called.
- `wasm`: Specifies Wasm plugin configurations. Within this:
  - `path`: Points to the wasm binary encapsulating the plugin functions.
  - `import`: Denotes the specific plugin function encapsulated within the wasm binary.

## Filters and Functions

With `jintemplify`, you're not limited to just basic Jinja2 templating. We've introduced specialized filters and functions to provide more flexibility:

### Functions

- **Reading Files Directly**: With the read_file function, you can directly read the contents of a file into your Jinja2 template. This is especially useful for including large chunks of data or content without manually copying them into the template.
- **Converting Arrays to Objects**: With the `to_object` filter, you can convert arrays with alternating keys and values (e.g., `[key1, value1, key2, value2]`) into a Jinja2 accessible object (`{key1: value1, key2: value2}`).
- **Executing Shell Commands**: With the `shell` function, you can directly execute shell commands from within your Jinja2 template. This can be especially useful for dynamic content generation based on command outputs. For example, `shell("echo hello", world="Jinja")` will set the environment variable `world` to "Jinja" before executing the `echo hello` command.

### Filters

- **Reading From Strings**: If you have data embedded within your templates as strings, you can convert them into usable Jinja2 objects with the following filters:
  - `from_read_json`: Parse a JSON string and convert it to a Jinja2 object.
  - `from_read_yaml`: Parse a YAML string and convert it to a Jinja2 object.
  - `from_read_toml`: Parse a TOML string and convert it to a Jinja2 object.

By using these filters, you can seamlessly integrate inline data within your templates and then manipulate them using Jinja2's powerful templating capabilities.

### Example: Using my_read_file in plugin.yaml.j2 with JSON Parsing

One of the powerful combinations you can use in `jintemplify` is to read a file directly and then parse its content. Here's a quick example:

```yaml
{# plugin.yaml.j2 #}
functions:
  - name: my_read_file
    params:
      - name: file_path
        description: file path
    script: cat $(file_path)

  - name: my_echo
    env:
      my_var: "hello plugin"
    script: echo ${my_var}

filters:
  - name: my_indent
    params:
      - name: prefix
    description: "description for my filter"
    script: |
      #!/bin/bash
      echo -e $(input) | sed 's/^/$(prefix)/'

```

```jinja
{# main.yaml.j2 #}
{% set conf = my_read_file(file_path='./examples/test.json') | from_json %}
{{conf.hello}}
```

In this example, we're using the `read_file` function to read the contents of `test.json`. We then utilize the `from_json` filter to parse the read JSON string, converting it into a usable Jinja2 object. This allows you to directly access properties of the JSON, like `conf.hello` in the example above.

# Installation

## Using Cargo

If you have Rust and Cargo installed, you can easily install `jintemplify` using:

```bash
cargo instll jintemplify
```

## Manual Installation

1. Clone the repository:

```bash
git clone https://github.com/your_username/jintemplify.git
```

2. Navigate to the project directory and build using Cargo:

```bash
cd jintemplify
cargo build --release
```

## Usage

```bash
jintemplify -t <template_path> -v <variables_path> --plugin <plugin_path>
```

```bash
jintemplify --help
```

```plaintext
jintemplify allows you to combine Jinja2 templates with YAML variables to produce files in any desired format. Use the --template argument to specify the main Jinja2 template and the --variables argument (optional) to specify the YAML variables template.

Usage: jintemplify [OPTIONS] --template <template>

Options:
  -e, --env <env>
          Environment variables in the format key=value

      --default-env <default-env>
          Optional environment variables in the format key=default_value

  -t, --template <template>
          Template file: main.yaml.j2, main.txt.j2, main.json.j2

  -v, --variables <variables>
          Variables file: variables.yaml.j2

  -p, --plugin <plugin>
          Path to the plugin configuration: plugin.yaml

      --output-file <FILE>
          Sets an output file, stdout if not set

      --disable-builtin-functions
          Disables the registration of built-in functions

      --include-dir <include-dir>
          Include directory for templates. Format: /path/to/dir:alias or /path/to/dir. Use '{}' for direct naming without an alias.

  -h, --help
          Print help (see a summary with '-h')

  -V, --version
          Print version
```

## Development

To add new filters, modify `filter.rs`. For adding or modifying plugins, see `plugin.rs`.

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

## Supported Platforms

https://rust-lang.github.io/rustup-components-history/

## License

[MIT](https://choosealicense.com/licenses/mit/)
