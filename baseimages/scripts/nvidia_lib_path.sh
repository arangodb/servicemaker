#!/bin/bash
# NVIDIA/RAPIDS wheels (py12cugraph) ship native .so files under site-packages.
# Use the path file baked into py12cugraph when present; otherwise discover at runtime.
_nvidia_path=""
if [ -f /home/user/.nvidia_ld_library_path ]; then
  _nvidia_path="$(tr -d '\n' < /home/user/.nvidia_ld_library_path)"
else
  _nvidia_path="$(
    { find /home/user/the_venv -path '*/nvidia/*/lib' -type d 2>/dev/null
      find /home/user/the_venv -path '*/libcugraph/lib64' -type d 2>/dev/null
      find /home/user/the_venv -name 'libcugraph.so*' -exec dirname {} \; 2>/dev/null
    } | sort -u | paste -sd: -
  )"
fi
if [ -n "$_nvidia_path" ]; then
  export LD_LIBRARY_PATH="${_nvidia_path}:${LD_LIBRARY_PATH:-}"
fi
