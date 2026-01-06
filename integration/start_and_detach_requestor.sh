#!/bin/bash

set -x

./build_matcher.sh
./start_matcher.sh &

sleep 2

# Start yagna nodes
./start_requestor.sh &

sleep 10

# Start vanity service
./start_vanity.sh &
