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
  pkill -9 yp-geode-"$i"
  pkill -9 yagna-geode-"$i"
  pkill -9 vanity-lower-"$i"
  pkill -9 yagna-lower-"$i"
done

pkill -9 router-geode