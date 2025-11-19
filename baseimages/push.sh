#!/bin/bash
for i in $(cat imagelist.txt) ; do 
  docker push $i
done
