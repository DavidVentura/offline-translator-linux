#!/bin/bash
set -e
./clickable/package-click.sh -a arm64 --skip-review
./venv/bin/clickable install  -c clickable.yaml -a arm64  --serial-number 93BAY08W6S  
./venv/bin/clickable launch  -c clickable.yaml -a arm64  --serial-number 93BAY08W6S  

