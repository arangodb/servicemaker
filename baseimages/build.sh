#!/bin/bash

# Build base image first so derived images can reuse its layers
echo "Building image py13base ..."
docker build -f "Dockerfile.py13base" -t "arangodb/py13base" .

for i in $(cat imagelist.txt) ; do
  if [ "$i" = "py13base" ]; then
    continue
  fi
  echo "Building image $i ..."
  docker build -f "Dockerfile.$i" -t "arangodb/$i" .
done
