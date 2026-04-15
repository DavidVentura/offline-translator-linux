#!/bin/bash
#run this once manually
#Xvfb :99 -screen 0 600x1024x24 &
set -eu
export QMAKE=/usr/bin/qmake
export CLICKABLE_DESKTOP_MODE=1
export START_SCREEN=${START_SCREEN:-}
cargo build
export DISPLAY=:99
cargo run &
sleep 1
import -window root screenshot.png
kill %1
