#!/bin/bash

if [ -z "$1" ]; then
  echo "usage: $0 N"
  exit 1
fi

end=$(( $1 - 1 ))
if [ "$end" -lt 0 ]; then
  echo "no nodes to start"
  exit 0
fi

set -x

for i in $(seq 0 "$end"); do
  (cd "node-deployer/services/geode-$i/yagna" && ./yagna-geode-"$i" service run >/dev/null 2>&1) &
done