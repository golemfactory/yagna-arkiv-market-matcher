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
  pkill ya-provider-upper-"$i"
  pkill yagna-upper-"$i"
  pkill vanity-lower-"$i"
  pkill yagna-lower-"$i"
done