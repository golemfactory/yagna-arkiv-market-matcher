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

tar -cf requestor.tar.gz req-deployer/services/lower-0/yagna/yagnadir
for i in $(seq 0 "$end"); do
  tar -cf provider-yagnadir-"$i".tar.gz node-deployer/services/upper-"$i"/yagna/yagnadir
  tar -cf provider-provdir-"$i".tar.gz node-deployer/services/upper-"$i"/yagna/provdir
done

tar -cf all-logs.tar.gz *.tar.gz