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
  
  # Check if it's a Node.js application (package.json exists, no services.json or manifest.json)
  if [ -f "package.json" ]; then
    # Node.js application
    echo "Detected Node.js application"
    exec node $ENTRYPOINT
  else
    # Python service (has pyproject.toml or no package.json)
    echo "Detected Python service"
    . /home/user/.local/bin/env
    . /home/user/the_venv/bin/activate
    . /scripts/nvidia_lib_path.sh
    for p in /project/the_venv/lib/python*/site-packages ; do
      export PYTHONPATH=$p
    done
    exec python $ENTRYPOINT
  fi
fi

echo No entrypoint found, running bash instead...

exec bash
