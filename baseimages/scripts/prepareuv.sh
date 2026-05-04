#!/bin/bash
# To be run as user `user` in /home/user to prepare uv and `the_venv`.
# Uses the distro Python from PATH (e.g. Ubuntu's python3), not a uv-managed CPython pin.

curl -LsSf https://astral.sh/uv/install.sh | sh

export UV_HTTP_TIMEOUT=3600
rm -rf the_venv
. /home/user/.local/bin/env
uv venv --python python3 the_venv
. the_venv/bin/activate
uv pip install "$@"

find the_venv -type f -print0 | xargs -0 sha256sum > sums_sha256
uv pip freeze > /home/user/constraints.txt

# # Uninstall uv (cleanup)
rm -f /home/user/.local/bin/uv
rm -f /home/user/.local/bin/uvx