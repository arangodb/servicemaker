#!/bin/bash
set -euo pipefail

# Scan base/test images for security vulnerabilities using Trivy.
# CircleCI uses arangodb/trivy-scan orb for image scans; this script handles:
#   - Local full scans (image + in-container trees)
#   - CI in-container scans only (IN_CONTAINER_ONLY=1)
#
# In-container targets when present:
#   - Python images: /home/user/the_venv
#   - Node images:   /home/user/node_modules

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGELIST_FILE="${SCRIPT_DIR}/imagelist.txt"
IMAGELIST_FILE_COPY="/tmp/imagelist.txt"
TRIVY_IMAGE="${TRIVY_IMAGE:-aquasec/trivy:latest}"
TRIVY_SEVERITY="${TRIVY_SEVERITY:-HIGH,CRITICAL}"
IN_CONTAINER_ONLY="${IN_CONTAINER_ONLY:-0}"

if [[ ! -f "${IMAGELIST_FILE}" ]]; then
    echo "Error: imagelist.txt not found at ${IMAGELIST_FILE}" >&2
    exit 1
fi

cp "${IMAGELIST_FILE}" "${IMAGELIST_FILE_COPY}"
echo "test-service" >> "${IMAGELIST_FILE_COPY}"
echo "test-service-nodejs" >> "${IMAGELIST_FILE_COPY}"

if ! command -v docker &> /dev/null; then
    echo "Error: docker is not installed" >&2
    exit 1
fi

scan_image_with_trivy() {
    local full_image_name="$1"
    echo "Scanning Docker image: ${full_image_name}"
    docker run --rm \
        -v /var/run/docker.sock:/var/run/docker.sock \
        "${TRIVY_IMAGE}" image \
        --severity "${TRIVY_SEVERITY}" \
        --ignore-unfixed \
        --exit-code 1 \
        "${full_image_name}"
}

scan_in_container_tree() {
    local full_image_name="$1"
    local container_id=""
    local scan_target=""
    local scan_exit=0

    echo ""
    echo "Scanning in-container install tree: ${full_image_name}"
    set +e
    container_id=$(docker run -d "${full_image_name}" sleep 3600 2>&1)
    local docker_run_exit=$?
    set -e

    if [[ ${docker_run_exit} -ne 0 ]] || [[ -z "${container_id}" ]]; then
        echo "✗ Failed to start container for ${full_image_name}: ${container_id}" >&2
        return 1
    fi

    if docker exec "${container_id}" test -d /home/user/the_venv 2>/dev/null; then
        scan_target="/home/user/the_venv"
        echo "  (Python) scanning ${scan_target}"
    elif docker exec "${container_id}" test -d /home/user/node_modules 2>/dev/null; then
        scan_target="/home/user/node_modules"
        echo "  (Node) scanning ${scan_target}"
    else
        echo "  No /home/user/the_venv or /home/user/node_modules — skipping in-container scan"
        docker rm -f "${container_id}" >/dev/null 2>&1 || true
        return 0
    fi

    set +e
    docker run --rm \
        --volumes-from "${container_id}" \
        "${TRIVY_IMAGE}" fs \
        --severity "${TRIVY_SEVERITY}" \
        --ignore-unfixed \
        --exit-code 1 \
        "${scan_target}"
    scan_exit=$?
    set -e

    docker rm -f "${container_id}" >/dev/null 2>&1 || true

    if [[ ${scan_exit} -eq 0 ]]; then
        echo "✓ ${full_image_name} passed in-container dependency scan"
        return 0
    fi

    echo "✗ ${full_image_name} failed in-container dependency scan (${TRIVY_SEVERITY} severity issues found)" >&2
    return 1
}

echo "Starting security scan of base images..."
echo "========================================"

FAILED_SCANS=0
TOTAL_SCANS=0

while IFS= read -r image_name || [[ -n "$image_name" ]]; do
    [[ -z "$image_name" || "$image_name" =~ ^[[:space:]]*# ]] && continue
    image_name=$(echo "$image_name" | xargs)
    [[ -z "$image_name" ]] && continue

    FULL_IMAGE_NAME="arangodb/${image_name}"
    TOTAL_SCANS=$((TOTAL_SCANS + 1))

    echo ""
    echo "Scanning: ${FULL_IMAGE_NAME}"
    echo "----------------------------------------"

    if [[ "${IN_CONTAINER_ONLY}" != "1" ]]; then
        if scan_image_with_trivy "${FULL_IMAGE_NAME}"; then
            echo "✓ ${FULL_IMAGE_NAME} passed Docker image security scan"
        else
            echo "✗ ${FULL_IMAGE_NAME} failed Docker image security scan" >&2
            FAILED_SCANS=$((FAILED_SCANS + 1))
            continue
        fi
    fi

    if ! scan_in_container_tree "${FULL_IMAGE_NAME}"; then
        FAILED_SCANS=$((FAILED_SCANS + 1))
    fi
done < "${IMAGELIST_FILE_COPY}"

echo ""
echo "========================================"
echo "Scan complete: ${TOTAL_SCANS} images scanned"

if [[ ${FAILED_SCANS} -gt 0 ]]; then
    echo "ERROR: ${FAILED_SCANS} image(s) failed security scan"
    exit 1
fi

echo "SUCCESS: All images passed security scan"
exit 0
