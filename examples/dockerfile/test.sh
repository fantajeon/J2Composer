#!/bin/bash
jintemplify \
    --template ./examples/dockerfile/Dockerfile.j2 \
    --variables ./examples/dockerfile/variables.yaml.j2 \
    --env stage=release	\
    --output-file Dockerfile