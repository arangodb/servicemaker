#!/bin/bash
set -euo pipefail

# ARCH (optional): target architecture, e.g. amd64 or arm64.
# Unset: build for the host platform and tag arangodb/<image> (local default).
# Set: build with --platform linux/$ARCH and additionally tag arangodb/<image>:latest-$ARCH.
ARCH="${ARCH:-}"

SECURITY_REFRESH="${SECURITY_REFRESH:-${CIRCLE_BUILD_NUM:-$(date -u +%Y-%m-%d)}}"
PLATFORM_ARGS=()
if [ -n "${ARCH}" ]; then
  PLATFORM_ARGS=(--platform "linux/${ARCH}")
fi
BUILD_ARGS=(--pull --build-arg "SECURITY_REFRESH=${SECURITY_REFRESH}")

tag_args() {
  echo "-t arangodb/$1"
  if [ -n "${ARCH}" ]; then
    echo "-t arangodb/$1:latest-${ARCH}"
  fi
}

# Build base image first so derived images can reuse its layers
echo "Building image py12base ${ARCH:+for linux/${ARCH} }..."
# shellcheck disable=SC2046
docker build "${PLATFORM_ARGS[@]}" "${BUILD_ARGS[@]}" -f "Dockerfile.py12base" $(tag_args py12base) .

for i in $(cat imagelist.txt) ; do
  if [ "$i" = "py12base" ]; then
    continue
  fi
  echo "Building image $i ${ARCH:+for linux/${ARCH} }..."
  # shellcheck disable=SC2046
  if [ "$i" = "py12cugraph" ] || [ "$i" = "py12torch" ]; then
    # FROM arangodb/py12base: uses the py12base built above (same platform)
    docker build "${PLATFORM_ARGS[@]}" -f "Dockerfile.$i" $(tag_args "$i") .
  else
    docker build "${PLATFORM_ARGS[@]}" "${BUILD_ARGS[@]}" -f "Dockerfile.$i" $(tag_args "$i") .
  fi
done
