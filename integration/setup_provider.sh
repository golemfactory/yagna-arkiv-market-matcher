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

YAGNA_VERSION=${YAGNA_VERSION}

MACHINE_PROV=${MACHINE_PROV:-geode}
MACHINE_PROV_SECRET=${MACHINE_PROV_SECRET:-"abc123"}
CENTRAL_NET_HOST=${CENTRAL_NET_HOST:-"127.0.0.1:6976"}

git clone git@github.com:salad-x-golem/node-deployer.git
(cd node-deployer && git clean -fdx && git reset --hard && git checkout provider-only && git pull)

(cd node-deployer && ../venv/bin/python keys.py "${NUMBER_OF_NODES}")
(cd node-deployer && mkdir "${MACHINE_PROV}_keys" && mv generated_keys/keys.txt "${MACHINE_PROV}_keys/${MACHINE_PROV}.keys" )
(cd node-deployer && printf "NODE_PREFIX=%s\nNODE_SECRET=%s\nNO_SERVICES=true\nYAGNA_VERSION=${YAGNA_VERSION}\nCENTRAL_NET_HOST=${CENTRAL_NET_HOST}" "${MACHINE_PROV}" "${MACHINE_PROV_SECRET}" > .env )
(cd node-deployer && ../venv/bin/python bootstrap.py)
(cd node-deployer && ./setup-all.sh)
