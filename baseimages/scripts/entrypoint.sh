#!/bin/bash
set -e
cd /project

# Try to download project zip, if the `projectURL` file is there:
if test -e projectURL ; then
  curl -O project.tar.gz $(cat projectURL)
fi

# Try to unzip the project zip, if the `project.tar.gz` file is there:
if test -e project.tar.gz ; then
  tar czvf project.tar.gz
fi

# Run the entrypoint if configured:
if test -L entrypoint ; then
  export PYTHONPATH=/project
  exec entrypoint
fi

echo No entrypoint found, running bash instead...

exec bash
