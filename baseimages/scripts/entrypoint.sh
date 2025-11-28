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

# Detect service type and run accordingly
if test -e entrypoint ; then
  ENTRYPOINT=$(cat entrypoint)
  echo Running project ...
  
  # Check if it's a Node.js/Foxx service
  if [ -f "package.json" ] && [ -f "services.json" ]; then
    # Node.js/Foxx service
    echo "Detected Node.js/Foxx service"
    if [ -f "node_modules/.bin/node-foxx" ]; then
      exec node_modules/.bin/node-foxx
    else
      echo "Error: node-foxx not found. Make sure node_modules are installed."
      exit 1
    fi
  elif [ -f "package.json" ]; then
    # Generic Node.js service
    echo "Detected Node.js service"
    if [ -f "$ENTRYPOINT" ]; then
      exec node "$ENTRYPOINT"
    else
      echo "Error: Entrypoint file not found: $ENTRYPOINT"
      exit 1
    fi
  else
    # Python service (existing logic)
    echo "Detected Python service"
    . /home/user/.local/bin/env
    . /home/user/the_venv/bin/activate
    for p in /project/the_venv/lib/python*/site-packages ; do
      export PYTHONPATH=$p
    done
    exec python $ENTRYPOINT
  fi
fi

echo No entrypoint found, running bash instead...

exec bash
