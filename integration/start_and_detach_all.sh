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

set -x


./start_router.sh &

# ./build_matcher.sh
# ./start_matcher.sh &

sleep 2

# Start yagna nodes
./start_provider_node.sh "${NUMBER_OF_NODES}" &
./start_requestor.sh &

sleep 10

# Start provider
./start_provider.sh "${NUMBER_OF_NODES}" &

# Start vanity service
./start_vanity.sh &
