#!/bin/bash
set -x

# Start router
(cd ../ && cargo build -p yagna-offer-server)