#!/bin/bash

if [ -z "$1" ]; then
  echo "usage: $0 N"
  exit 1
fi

NUMBER_OF_NODES=$1
if [ "$NUMBER_OF_NODES" -lt 0 ]; then
  echo "no nodes to setup"
  exit 0
fi

MACHINE_PROV="upper"
MACHINE_REQ="lower"
MACHINE_PROV_SECRET="abc123"
MACHINE_REQ_SECRET="bca321"
YAGNA_VERSION="pre-rel-v0.17.6-preview.noarkiv.14"

set -x

# rm -fr venv
# rm -fr node-deployer
# rm -fr req-deployer

# Prepare Python environment
python -m venv venv
venv/bin/python -m pip install eth-account requests dotenv

# Clone required repositories
git clone git@github.com:salad-x-golem/node-deployer.git
(cd node-deployer && git clean -fdx && git reset --hard && git checkout provider-only && git pull)
git clone git@github.com:salad-x-golem/req-deployer.git
(cd req-deployer && git clean -fdx && git reset --hard && git checkout main && git pull)

(cd node-deployer && ../venv/bin/python keys.py "${NUMBER_OF_NODES}")
(cd node-deployer && mkdir "${MACHINE_PROV}_keys" && mv generated_keys/keys.txt "${MACHINE_PROV}_keys/${MACHINE_PROV}.keys" )
(cd node-deployer && printf "NODE_PREFIX=%s\nNODE_SECRET=%s\nNO_SERVICES=true\nYAGNA_VERSION=${YAGNA_VERSION}\nCENTRAL_NET_HOST=http://127.0.0.1:6976" "${MACHINE_PROV}" "${MACHINE_PROV_SECRET}" > .env )
(cd node-deployer && ../venv/bin/python bootstrap.py)

(cd req-deployer && ../venv/bin/python keys.py 1)
(cd req-deployer && mkdir "${MACHINE_REQ}_keys" && mv generated_keys/keys.txt "${MACHINE_REQ}_keys/${MACHINE_REQ}.keys" )
(cd req-deployer && printf "NODE_PREFIX=%s\nNODE_SECRET=%s\nNO_SERVICES=true\nYAGNA_VERSION=${YAGNA_VERSION}\nCENTRAL_NET_HOSTS=http://127.0.0.1:6976" "${MACHINE_REQ}" "${MACHINE_REQ_SECRET}" > .env )
(cd req-deployer && ../venv/bin/python bootstrap.py)

(cd node-deployer && ./setup-all.sh)
(cd req-deployer && ./setup-all.sh)

# Router
(cd node-deployer/central-net && ./download_router.sh)

