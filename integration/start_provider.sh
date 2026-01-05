#!/bin/bash
set -x

for i in $(seq 0 "$1"); do
  (cd "node-deployer/services/upper-$i/yagna" && ./ya-provider run >/dev/null 2>&1) &
done