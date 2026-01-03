#!/bin/bash
set -x

# Start router
(cd node-deployer/central-net && ./ya-sb-router -l tcp://0.0.0.0:6999)