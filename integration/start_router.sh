#!/bin/bash
set -x

# Start router
(cp ya-sb-router router-geode)
(./router-geode -l tcp://0.0.0.0:6976)