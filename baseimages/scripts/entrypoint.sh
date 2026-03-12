#!/bin/bash
set -e
cd /project

# Try to download project zip, if the `projectURL` file is there:
if test -e projectURL ; then
  echo Downloading project.tar.gz ...
  # Note: uses -o (lowercase) to specify output filename; -O (uppercase) would ignore the filename
  curl -o project.tar.gz "$(cat projectURL)"
fi

# Try to download project archive from ARCHIVE_FILE env variable:
echo "ARCHIVE_FILE=${ARCHIVE_FILE:-<not set>}"
if [ -n "$ARCHIVE_FILE" ] ; then
  echo "Downloading project.tar.gz from $ARCHIVE_FILE ..."
  curl -o project.tar.gz "$ARCHIVE_FILE"
  echo "curl exit code: $?"
  echo "Downloaded file info:"
  ls -lh project.tar.gz 2>/dev/null || echo "project.tar.gz not found after download"
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
  
  # Check if it's a Node.js/Foxx service (requires both package.json and services.json)
  if [ -f "package.json" ] && [ -f "services.json" ]; then
    # Node.js/Foxx service
    echo "Detected Node.js/Foxx service"
    if [ -f "node_modules/.bin/node-foxx" ]; then
      exec node_modules/.bin/node-foxx
    elif [ -f "/home/user/node_modules/.bin/node-foxx" ]; then
      # Fallback to base node-foxx binary
      exec /home/user/node_modules/.bin/node-foxx
    else
      echo "Error: node-foxx not found. Make sure node_modules are installed."
      exit 1
    fi
  elif [ -f "package.json" ] && [ ! -f "services.json" ] && [ ! -f "manifest.json" ] && grep -q '"express"' package.json 2>/dev/null; then
    # Express.js application
    echo "Detected Express.js application"
    exec node $ENTRYPOINT
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
