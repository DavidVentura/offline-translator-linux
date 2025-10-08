(cd ../ && cargo build) && cp ../target/debug/offline-translator-linux translator && strip translator && clickable desktop  --docker-image clickable/amd64-ut24.04-1.x-amd64
