#!/bin/bash
set -x

/bin/bash start_router.sh &
/bin/bash start_matcher.sh &

sleep 2

# Start yagna nodes
/bin/bash start_provider_node.sh &
/bin/bash start_requestor.sh &

sleep 10

# Start provider
/bin/bash start_provider.sh &

# Start vanity service
/bin/bash start_vanity.sh
