#!/bin/bash

# Build base image first so derived images can reuse its layers
echo "Building image py12base ..."
docker build -f "Dockerfile.py12base" -t "arangodb/py12base" .

for i in $(cat imagelist.txt) ; do
  if [ "$i" = "py12base" ]; then
    continue
  fi
  echo "Building image $i ..."
  docker build -f "Dockerfile.$i" -t "arangodb/$i" .
done
