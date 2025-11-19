#!/bin/bash
# To be run as user `user` in /home/user to prepare uv and `the_venv`

PYTHON_VERSION=$1
shift

curl -LsSf https://astral.sh/uv/install.sh | sh

export UV_HTTP_TIMEOUT=3600
rm -rf the_venv
. /home/user/.local/bin/env
uv venv --python $PYTHON_VERSION the_venv
. the_venv/bin/activate
uv pip install "$@"

find the_venv -type f -print0 | xargs -0 sha256sum >> sums_sha256
