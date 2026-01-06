#!/bin/bash
set -x

MACHINE_PROV="geode"
MACHINE_REQ="lower"
MACHINE_PROV_SECRET="abc123"
MACHINE_REQ_SECRET="bca321"

# Start router
(cd node-deployer/central-net && ./ya-sb-router -l tcp://0.0.0.0:6976)&