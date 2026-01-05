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


/bin/bash start_router.sh &

# /bin/bash build_matcher.sh
# /bin/bash start_matcher.sh &

sleep 2

# Start yagna nodes
/bin/bash start_provider_node.sh "${NUMBER_OF_NODES}" &
/bin/bash start_requestor.sh &

sleep 10

# Start provider
/bin/bash start_provider.sh "${NUMBER_OF_NODES}"

# Start vanity service
/bin/bash start_vanity.sh &
