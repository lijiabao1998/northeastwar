#!/bin/bash
# Build the game. usage: tools/build.sh [--release]
cd "$(dirname "$0")/.." && . tools/env.sh && cargo build -p northeast "$@"
