#!/bin/bash

SECURITY_REFRESH="${SECURITY_REFRESH:-${CIRCLE_BUILD_NUM:-$(date -u +%Y-%m-%d)}}"
BUILD_ARGS=(--pull --build-arg "SECURITY_REFRESH=${SECURITY_REFRESH}")

# Build base image first so derived images can reuse its layers
echo "Building image py12base ..."
docker build "${BUILD_ARGS[@]}" -f "Dockerfile.py12base" -t "arangodb/py12base" .

for i in $(cat imagelist.txt) ; do
  if [ "$i" = "py12base" ]; then
    continue
  fi
  echo "Building image $i ..."
  if [ "$i" = "py12cugraph" ] || [ "$i" = "py12torch" ]; then
    docker build -f "Dockerfile.$i" -t "arangodb/$i" .
  else
    docker build "${BUILD_ARGS[@]}" -f "Dockerfile.$i" -t "arangodb/$i" .
  fi
done
