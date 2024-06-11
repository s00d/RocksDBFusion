git clone https://github.com/cross-rs/cross
cd cross
git submodule update --init --remote

cargo build-docker-image i686-apple-darwin-cross \
--build-arg 'MACOS_SDK_URL=$URL'

docker tag ghcr.io/cross-rs/i686-apple-darwin-cross:local s00d/i686-apple-darwin-cross:latest
docker push s00d/i686-apple-darwin-cross:latest


cargo build-docker-image aarch64-apple-darwin-cross \
--build-arg 'MACOS_SDK_URL=$URL'

docker tag ghcr.io/cross-rs/aarch64-apple-darwin-cross:local s00d/aarch64-apple-darwin-cross:latest
docker push s00d/aarch64-apple-darwin-cross:latest