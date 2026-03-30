#!/bin/bash
# This is to be run in the project directory to install dependencies into
# the virtual environment, then we identify all new files there and move
# them over to their final destination outside the virtual env (to be found
# via PYTHONPATH).

set -e

trap 'echo "[prepareproject] ERROR: command failed at line $LINENO: $BASH_COMMAND" >&2' ERR

echo "[prepareproject] Activating venv and installing dependencies..."
export UV_HTTP_TIMEOUT=3600
. /home/user/.local/bin/env
. /home/user/the_venv/bin/activate
uv pip install -c /home/user/constraints.txt -r pyproject.toml
echo "[prepareproject] Dependencies installed."

# First find all files which have changed, if any has changed, we abort:
echo "[prepareproject] Verifying base image integrity (sha256sum check)..."
cd /home/user
sha256sum -c sums_sha256
echo "[prepareproject] Integrity check passed."

# Now find all files which have been added:
echo "[prepareproject] Identifying newly installed files..."
find the_venv -type f -print0 | xargs -0 sha256sum >> sums_sha256_new
cat sums_sha256_new sums_sha256 | sort | uniq -c | grep "^      1 " | awk '{ print $3 }' > /tmp/newfiles
rm sums_sha256_new
NEW_COUNT=$(wc -l < /tmp/newfiles)
echo "[prepareproject] Found $NEW_COUNT new file(s) to relocate."

# Now move all files over to their new home under /project/the_venv:
echo "[prepareproject] Relocating new files to /project/the_venv..."
mkdir -p /project/the_venv   # just in case nothing is added
while IFS= read -r filename; do
    echo Moving "$filename" to "project/$filename" ...

    # Skip empty lines
    [[ -z "$filename" ]] && continue
    
    # Check if source file exists
    if [[ ! -f "$filename" ]]; then
        echo "Warning: File not found: $filename"
        exit 1
    fi
    
    # Define your destination base directory
    DEST_BASE="/project"
    
    # Get the directory part of the filename
    DIR=$(dirname "$filename")
    
    # Create the destination directory structure
    mkdir -p "$DEST_BASE/$DIR"
    
    # Copy the file preserving the directory hierarchy
    cp "$filename" "$DEST_BASE/$filename"
    
    # And remove it in the virtual env:
    rm -f $FILENAME
done < "/tmp/newfiles"

rm /tmp/newfiles
echo "[prepareproject] Done. $NEW_COUNT file(s) relocated successfully."
