#!/bin/bash
set -x

# Start router
(cd req-deployer/services/lower-0/yagna && ./yagna service run)