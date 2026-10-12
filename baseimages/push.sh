#!/bin/bash
set -euo pipefail

# ARCH (optional): push arangodb/<image>:latest-$ARCH (built by `ARCH=... ./build.sh`).
# Unset: push arangodb/<image> (latest), as before.
ARCH="${ARCH:-}"

for i in $(cat imagelist.txt) ; do
  docker push "arangodb/$i${ARCH:+:latest-${ARCH}}"
done
