# Variables
PLUGIN_DIR = plugins/read_file
PLUGIN_TARGET = wasm32-unknown-unknown
PLUGIN_OUTPUT = $(PLUGIN_DIR)/target/$(PLUGIN_TARGET)/release/plugin.wasm

# Default target
all: build

# Build the main application
.PHONY: build
build:
	cargo build --release

# Build the plugin
.PHONY: plugin
plugin:
	cargo build --release --target $(PLUGIN_TARGET) --manifest-path $(PLUGIN_DIR)/Cargo.toml

# Clean the project
.PHONY: clean
clean:
	cargo clean
	rm -f $(PLUGIN_OUTPUT)

# Run the main application
.PHONY: run
run: build
	./target/release/jintemplify

# Test the project
.PHONY: test
test:
	cargo test

# For other utility targets
.PHONY: all build plugin clean run test

.PHONY: ex1
ex1:
	cargo run -- \
		--template ./examples/scratch/main.yaml.j2 \
		--variables ./examples/scratch/variables.yaml.j2 \
		--plugin ./examples/scratch/plugin.yaml.j2 \
		--env var1=env1 \
		--include-dir="./examples/scratch:{}"	\
		--default-env MY_ENV=2	\
		--env file_path=./examples/scratch/test.json	\
		--output-file test.txt

.PHONY: ex-dockerfile
ex-dockerfile:
	RUST_LOG=debug cargo run -- \
		--template ./examples/dockerfile/Dockerfile.j2 \
		--variables ./examples/dockerfile/variables.yaml.j2 \
		--env stage=release	\
		--output-file Dockerfile