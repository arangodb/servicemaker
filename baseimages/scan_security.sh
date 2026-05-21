#!/bin/bash
set -euo pipefail

# Script to scan base images for security vulnerabilities using grype
# Reads image list from imagelist.txt and scans each image prefixed with arangodb/
# Also scans in-container install trees when present:
#   - Python images: /home/user/the_venv
#   - Node images:   /home/user/node_modules

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGELIST_FILE="${SCRIPT_DIR}/imagelist.txt"
IMAGELIST_FILE_COPY="/tmp/imagelist.txt"

if [[ ! -f "${IMAGELIST_FILE}" ]]; then
    echo "Error: imagelist.txt not found at ${IMAGELIST_FILE}" >&2
    exit 1
fi

cp $IMAGELIST_FILE $IMAGELIST_FILE_COPY
echo "test-service" >> $IMAGELIST_FILE_COPY
echo "test-service-nodejs" >> $IMAGELIST_FILE_COPY

# Check if grype is installed
if ! command -v grype &> /dev/null; then
    echo "Error: grype is not installed" >&2
    exit 1
fi

# Check if docker is installed
if ! command -v docker &> /dev/null; then
    echo "Error: docker is not installed" >&2
    exit 1
fi

echo "Starting security scan of base images..."
echo "========================================"

FAILED_SCANS=0
TOTAL_SCANS=0

while IFS= read -r image_name || [[ -n "$image_name" ]]; do
    # Skip empty lines and comments
    [[ -z "$image_name" || "$image_name" =~ ^[[:space:]]*# ]] && continue
    
    # Trim whitespace
    image_name=$(echo "$image_name" | xargs)
    
    # Skip if empty after trimming
    [[ -z "$image_name" ]] && continue
    
    FULL_IMAGE_NAME="arangodb/${image_name}"
    TOTAL_SCANS=$((TOTAL_SCANS + 1))
    
    echo ""
    echo "Scanning: ${FULL_IMAGE_NAME}"
    echo "----------------------------------------"
    
    # Scan the Docker image itself
    echo "Scanning Docker image: ${FULL_IMAGE_NAME}"
    if grype -v --fail-on high "${FULL_IMAGE_NAME}"; then
        echo "✓ ${FULL_IMAGE_NAME} passed Docker image security scan"
    else
        echo "✗ ${FULL_IMAGE_NAME} failed Docker image security scan (high severity issues found)"
        FAILED_SCANS=$((FAILED_SCANS + 1))
        continue
    fi
    
    # Scan bundled dependencies inside the container (path depends on image type)
    echo ""
    echo "Scanning in-container install tree: ${FULL_IMAGE_NAME}"
    CONTAINER_ID=""
    set +e
    CONTAINER_ID=$(docker run -d "${FULL_IMAGE_NAME}" sleep 3600 2>&1)
    DOCKER_RUN_EXIT=$?
    set -e
    
    if [[ ${DOCKER_RUN_EXIT} -eq 0 ]] && [[ -n "${CONTAINER_ID}" ]]; then
        SCAN_RESULT=0
        SCAN_TARGET=""
        if docker exec "${CONTAINER_ID}" test -d /home/user/the_venv 2>/dev/null; then
            SCAN_TARGET="/home/user/the_venv"
            echo "  (Python) scanning ${SCAN_TARGET}"
        elif docker exec "${CONTAINER_ID}" test -d /home/user/node_modules 2>/dev/null; then
            SCAN_TARGET="/home/user/node_modules"
            echo "  (Node) scanning ${SCAN_TARGET}"
        else
            echo "  No /home/user/the_venv or /home/user/node_modules — skipping in-container scan"
            SCAN_TARGET=""
        fi

        set +e
        if [[ -n "${SCAN_TARGET}" ]]; then
            docker exec "${CONTAINER_ID}" bash -c "curl -sSfL https://get.anchore.io/grype | sh -s -- -b /home/user/.local/bin && /home/user/.local/bin/grype -v --fail-on high '${SCAN_TARGET}'"
            SCAN_EXIT=$?
        else
            SCAN_EXIT=0
        fi
        set -e
        
        if [[ ${SCAN_EXIT} -eq 0 ]]; then
            echo "✓ ${FULL_IMAGE_NAME} passed in-container dependency scan"
        else
            echo "✗ ${FULL_IMAGE_NAME} failed in-container dependency scan (high severity issues found)"
            SCAN_RESULT=1
            FAILED_SCANS=$((FAILED_SCANS + 1))
        fi
        
        # Clean up container
        docker rm -f "${CONTAINER_ID}" >/dev/null 2>&1 || true
        
        # If scan failed, continue to next image
        if [[ ${SCAN_RESULT} -ne 0 ]]; then
            continue
        fi
    else
        echo "✗ Failed to start container for ${FULL_IMAGE_NAME}: ${CONTAINER_ID}"
        FAILED_SCANS=$((FAILED_SCANS + 1))
    fi
done < "${IMAGELIST_FILE_COPY}"

echo ""
echo "========================================"
echo "Scan complete: ${TOTAL_SCANS} images scanned"

if [[ ${FAILED_SCANS} -gt 0 ]]; then
    echo "ERROR: ${FAILED_SCANS} image(s) failed security scan"
    exit 1
else
    echo "SUCCESS: All images passed security scan"
    exit 0
fi

