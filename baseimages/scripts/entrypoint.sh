#!/bin/bash
set -e
cd /project

# Try to download project zip, if the `projectURL` file is there:
if test -e projectURL ; then
  echo Downloading project.tar.gz ...
  # Note: uses -o (lowercase) to specify output filename; -O (uppercase) would ignore the filename
  curl -o project.tar.gz "$(cat projectURL)"
fi

# Try to download project archive from ARCHIVE_FILE env variable.
# On the platform the archive is a File Manager package, and the File Manager
# checks Platform RBAC on the download (ai:filemanager.package.read on
# ai:service:arango-user-defined). The operator mounts the pod's
# ArangoPermissionToken and puts its path in ARANGO_TOKEN; it is sent as a
# Bearer token. Without a token (local run, cluster without the token) the
# download is unauthenticated, as before. The token is never printed.
echo "ARCHIVE_FILE=${ARCHIVE_FILE:-<not set>}"
if [ -n "$ARCHIVE_FILE" ] ; then
  AUTH_HEADER=()
  if [ -n "${ARANGO_TOKEN:-}" ] && [ -r "$ARANGO_TOKEN" ] ; then
    AUTH_HEADER=(-H "Authorization: Bearer $(cat "$ARANGO_TOKEN")")
    echo "Sending the pod's ArangoPermissionToken with the download"
  else
    echo "No ArangoPermissionToken found (ARANGO_TOKEN unset or unreadable); downloading without a token"
  fi
  echo "Downloading project.tar.gz from $ARCHIVE_FILE ..."
  # -f: an HTTP error (401, 403, 404, ...) fails the download instead of saving
  # the error page as project.tar.gz; -sS: no progress bar, errors still shown.
  curl -fsS "${AUTH_HEADER[@]}" -o project.tar.gz "$ARCHIVE_FILE" || {
    rc=$?
    echo "Download of $ARCHIVE_FILE failed (curl exit code $rc)"
    exit "$rc"
  }
  echo "Downloaded file info:"
  ls -lh project.tar.gz
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
