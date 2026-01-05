#!/bin/bash
set -x
NUMBER_OF_NODES=$1

/bin/bash start_router.sh &

# /bin/bash build_matcher.sh
# /bin/bash start_matcher.sh &

sleep 2

# Start yagna nodes
/bin/bash start_provider_node.sh "${NUMBER_OF_NODES}" &
/bin/bash start_requestor.sh &

sleep 10

# Start provider
/bin/bash start_provider.sh "${NUMBER_OF_NODES}" &

# Start vanity service
/bin/bash start_vanity.sh &
