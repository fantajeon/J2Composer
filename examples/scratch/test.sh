#!/bin/bash
jintemplify \
		--template ./examples/scratch/main.yaml.j2 \
		--variables ./examples/scratch/variables.yaml.j2 \
		--plugin ./examples/scratch/plugin.yaml.j2 \
		--env var1=env1 \
		--include-dir="./examples/scratch:{}"	\
		--default-env MY_ENV=2	\
		--env file_path=./examples/scratch/test.json	\
		--output-file test.txt