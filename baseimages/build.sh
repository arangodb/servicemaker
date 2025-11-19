#!/bin/bash
for i in $(cat imagelist.txt) ; do
  docker build -f "Dockerfile.$i" -t "$i" .
done
