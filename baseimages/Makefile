# ARCH (optional, amd64|arm64): build for linux/$(ARCH) and tag/push
# arangodb/<image>:latest-$(ARCH). Unset: host platform, arangodb/<image> (latest).
ARCH ?=
export ARCH

all: build

build:
	./build.sh

push:
	./push.sh

smoke:
	./smoke.sh
