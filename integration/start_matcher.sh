#!/bin/bash
set -x

# Start router
(cd ../ && cargo run -p ya-sb-matcher)