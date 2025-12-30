#!/bin/bash
for i in $(cat imagelist.txt) ; do
  echo Building image $i ...
  docker build -f "Dockerfile.$i" --platform "linux/amd64,linux/arm64" -t "neunhoef/$i" .
done
