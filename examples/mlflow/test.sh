#!/bin/bash
jintemplify \
		--template ./examples/mlflow/flavor.yaml.j2 \
		--plugin ./examples/mlflow/plugin.yaml.j2 \
		--output-file flavor.yaml