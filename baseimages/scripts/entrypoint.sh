#!/bin/bash
set -e
cd /project

# Try to download project zip, if the `projectURL` file is there:
if test -e projectURL ; then
  echo Downloading project.tar.gz ...
  curl -O project.tar.gz $(cat projectURL)
fi

# Try to unzip the project zip, if the `project.tar.gz` file is there:
if test -e project.tar.gz ; then
  echo Extracting project.tar.gz ...
  tar xzvf project.tar.gz > /dev/null
fi

# Run the entrypoint if configured:
if test -L entrypoint ; then
  echo Running project ...
  . /home/user/.local/bin/env
  . /home/user/the_venv/bin/activate
  for p in /project/the_venv/lib/python*/site-packages ; do
    export PYTHONPATH=$p
  done
  exec uv run ./entrypoint
fi

echo No entrypoint found, running bash instead...

exec bash
