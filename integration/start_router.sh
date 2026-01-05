#!/bin/bash
set -x

# Start router
(cd node-deployer/central-net && cp ya-sb-router router-upper)
(cd node-deployer/central-net && ./router-upper -l tcp://0.0.0.0:6999)