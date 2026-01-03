set -x

./start_router.sh &
./start_matcher.sh &

sleep 2

# Start yagna nodes
./start_provider_node.sh &
./start_requestor.sh &

sleep 10

# Start provider
./start_provider.sh &

# Start vanity service
./start_vanity.sh
