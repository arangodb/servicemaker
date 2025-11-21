#!/bin/bash
set -euo pipefail

# Script to scan base images for security vulnerabilities using grype
# Reads image list from imagelist.txt and scans each image prefixed with arangodb/
# Also scans the virtual environment (/home/user/the_venv) inside each base image

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
IMAGELIST_FILE="${SCRIPT_DIR}/imagelist.txt"

if [[ ! -f "${IMAGELIST_FILE}" ]]; then
    echo "Error: imagelist.txt not found at ${IMAGELIST_FILE}" >&2
    exit 1
fi

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
    
    # Scan the virtual environment inside the container
    echo ""
    echo "Scanning virtual environment in container: ${FULL_IMAGE_NAME}"
    CONTAINER_ID=""
    set +e
    CONTAINER_ID=$(docker run -d "${FULL_IMAGE_NAME}" sleep 3600 2>&1)
    DOCKER_RUN_EXIT=$?
    set -e
    
    if [[ ${DOCKER_RUN_EXIT} -eq 0 ]] && [[ -n "${CONTAINER_ID}" ]]; then
        # Install grype inside the container and scan the virtual environment
        SCAN_RESULT=0
        set +e
        docker exec "${CONTAINER_ID}" bash -c "curl -sSfL https://get.anchore.io/grype | sh -s -- -b /home/user/.local/bin && /home/user/.local/bin/grype -v --fail-on high /home/user/the_venv"
        SCAN_EXIT=$?
        set -e
        
        if [[ ${SCAN_EXIT} -eq 0 ]]; then
            echo "✓ ${FULL_IMAGE_NAME} passed virtual environment security scan"
        else
            echo "✗ ${FULL_IMAGE_NAME} failed virtual environment security scan (high severity issues found)"
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
done < "${IMAGELIST_FILE}"

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

