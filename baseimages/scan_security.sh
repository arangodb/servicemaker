#!/bin/bash
set -euo pipefail

# Script to scan base images for security vulnerabilities using grype
# Reads image list from imagelist.txt and scans each image prefixed with arangodb/

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
    
    if grype --fail-on high "${FULL_IMAGE_NAME}"; then
        echo "✓ ${FULL_IMAGE_NAME} passed security scan"
    else
        echo "✗ ${FULL_IMAGE_NAME} failed security scan (high severity issues found)"
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

