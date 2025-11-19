#!/bin/bash
for i in $(cat imagelist.txt) ; do
  echo Building image $i ...
  docker build -f "Dockerfile.$i" -t "neunhoef/$i" .
done
