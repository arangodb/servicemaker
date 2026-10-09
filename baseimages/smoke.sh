#!/bin/bash
set -euo pipefail

# Smoke-test locally built images: check the image architecture and run a
# minimal command inside each one. Never pushes anything.
#   ARCH=arm64 ./smoke.sh                  # all images from imagelist.txt
#   ARCH=arm64 ./smoke.sh test-service     # selected images
# ARCH set: checks arangodb/<image>:latest-$ARCH (tagged by build.sh / make test-service).
# ARCH unset: checks arangodb/<image> and skips the architecture check.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARCH="${ARCH:-}"
if [ "$#" -gt 0 ]; then
  IMAGES="$*"
else
  IMAGES="$(cat "${SCRIPT_DIR}/imagelist.txt")"
fi

VENV='. /home/user/the_venv/bin/activate'

run_in() {
  local img="$1"
  shift
  docker run --rm ${ARCH:+--platform "linux/${ARCH}"} "${img}" bash -c "$*"
}

for i in ${IMAGES}; do
  img="arangodb/${i}${ARCH:+:latest-${ARCH}}"
  echo "=== ${img} ${ARCH:+(linux/${ARCH})}"
  got="$(docker image inspect -f '{{.Os}}/{{.Architecture}}' "${img}")"
  echo "image platform: ${got}"
  if [ -n "${ARCH}" ] && [ "${got}" != "linux/${ARCH}" ]; then
    echo "ERROR: ${img} is ${got}, expected linux/${ARCH}" >&2
    exit 1
  fi
  run_in "${img}" 'echo "uname -m: $(uname -m)"'
  case "${i}" in
    py12base)
      run_in "${img}" "${VENV} && python3 --version && python3 -c 'import arango, phenolrs, networkx, urllib3; print(\"imports ok\")'"
      ;;
    py12torch)
      run_in "${img}" "${VENV} && python3 --version && python3 -c 'import torch, torch_geometric, torchaudio; print(\"torch\", torch.__version__, \"torch_geometric\", torch_geometric.__version__)'"
      ;;
    py12cugraph)
      run_in "${img}" "${VENV} && python3 --version && python3 -c 'import importlib.metadata as m; print(\"cugraph-cu12\", m.version(\"cugraph-cu12\"))'"
      # libcugraph.so must be a native binary for this architecture.
      run_in "${img}" "${VENV} && python3 -c '
import glob, platform, struct
libs = sorted(glob.glob(\"/home/user/the_venv/**/libcugraph.so*\", recursive=True))
assert libs, \"libcugraph.so not found\"
machine = struct.unpack(\"<H\", open(libs[0], \"rb\").read(20)[18:20])[0]
want = {\"x86_64\": 62, \"aarch64\": 183}[platform.machine()]
print(libs[0], \"e_machine\", machine, \"host\", platform.machine())
assert machine == want, \"libcugraph.so architecture mismatch\"
'"
      # Importing cugraph needs a CUDA driver for some code paths; there is no
      # GPU on the CI runners, so a failed import is reported but not fatal.
      if run_in "${img}" "${VENV} && . /scripts/nvidia_lib_path.sh && python3 -c 'import cugraph; print(\"cugraph\", cugraph.__version__)'"; then
        echo "import cugraph: ok"
      else
        echo "WARNING: import cugraph failed (no GPU / CUDA driver on this runner)"
      fi
      ;;
    node22base)
      run_in "${img}" "node --version && npm --version && node -e \"for (const m of ['express','arangojs','axios','bcrypt','winston']) require(m); console.log('modules ok')\""
      ;;
    test-service)
      run_in "${img}" "${VENV} && python3 --version && ls /project"
      ;;
    test-service-nodejs)
      run_in "${img}" "node --version && ls /project"
      ;;
    *)
      echo "no smoke command for ${i}, platform check only"
      ;;
  esac
done
echo "Smoke tests passed: ${IMAGES//$'\n'/ }"
