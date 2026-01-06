#!/bin/bash
set -x

wget https://github.com/golemfactory/ya-service-bus/releases/download/v0.6.3/ya-sb-router-linux-v0.6.3.tar.gz
tar -xvf ya-sb-router-linux-v0.6.3.tar.gz
mv ya-sb-router-linux-v0.6.3/ya-sb-router .
rm -rf ya-sb-router-linux-v0.6.3
rm ya-sb-router-linux-v0.6.3.tar.gz
./ya-sb-router --version