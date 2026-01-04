#!/bin/bash
set -x

(cd node-deployer/services/upper-0/yagna && ./ya-provider run)&
(cd node-deployer/services/upper-1/yagna && ./ya-provider run)&
(cd node-deployer/services/upper-2/yagna && ./ya-provider run)&
(cd node-deployer/services/upper-3/yagna && ./ya-provider run)&
(cd node-deployer/services/upper-4/yagna && ./ya-provider run)&
(cd node-deployer/services/upper-5/yagna && ./ya-provider run)&
(cd node-deployer/services/upper-6/yagna && ./ya-provider run)&
(cd node-deployer/services/upper-7/yagna && ./ya-provider run)&
(cd node-deployer/services/upper-8/yagna && ./ya-provider run)&
(cd node-deployer/services/upper-9/yagna && ./ya-provider run)&
