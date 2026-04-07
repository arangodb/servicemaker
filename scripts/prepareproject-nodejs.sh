#!/bin/bash
# This script installs only missing or incompatible project dependencies to the project's node_modules.
# Base node_modules at /home/user/node_modules is immutable and never copied.
# Uses check-base-dependencies.js to avoid duplicating packages that exist in base.

set -e

# We're in /project/{project-name} (WORKDIR)
# package.json is in the current directory
# node_modules will be created in the current directory
PROJECT_DIR=$(pwd)

# Verify base node_modules exists (immutable, pre-scanned)
if [ ! -d "/home/user/node_modules" ]; then
    echo "ERROR: Base node_modules not found at /home/user/node_modules"
    exit 1
fi

echo "Base node_modules found at /home/user/node_modules (immutable)"

# Install project dependencies if package.json exists
if [ -f "package.json" ]; then
    echo "Analyzing dependencies against base node_modules..."
    
    # Check which packages need to be installed
    CHECK_SCRIPT="/scripts/check-base-dependencies.js"
    if [ ! -f "$CHECK_SCRIPT" ]; then
        echo "ERROR: check-base-dependencies.js not found at $CHECK_SCRIPT"
        exit 1
    fi
    
    # Run dependency check script
    # Capture stdout (JSON) only, stderr (user messages) automatically goes to console
    INSTALL_DATA=$(node "$CHECK_SCRIPT")
    CHECK_RESULT=$?
    
    if [ $CHECK_RESULT -ne 0 ]; then
        echo "ERROR: Failed to check base dependencies (exit code: $CHECK_RESULT)"
        exit 1
    fi
    
    if [ -z "$INSTALL_DATA" ] || ! echo "$INSTALL_DATA" | grep -q '^{'; then
        echo "ERROR: Could not parse dependency check output"
        echo "Received: $INSTALL_DATA"
        exit 1
    fi
    
    # Count packages
    TOTAL_DEPS=$(echo "$INSTALL_DATA" | node -e "const data=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(data.totalDependencies)")
    FROM_BASE=$(echo "$INSTALL_DATA" | node -e "const data=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(data.packagesFromBase)")
    TO_INSTALL_COUNT=$(echo "$INSTALL_DATA" | node -e "const data=JSON.parse(require('fs').readFileSync(0,'utf8')); console.log(data.packagesToInstall.length)")
    
    echo ""
    echo "Dependency summary:"
    echo "  Total dependencies: $TOTAL_DEPS"
    echo "  Available in base: $FROM_BASE"
    echo "  To install: $TO_INSTALL_COUNT"
    echo ""

    if [ "$TO_INSTALL_COUNT" -gt 0 ]; then
        # npm 7+ reconciles the full dependency tree from package.json on every
        # `npm install`, even when a specific package is named.  To prevent it
        # from re-installing packages already in the base image we temporarily
        # replace package.json with one that only lists the missing deps.
        cp package.json package.json.bak

        node -e "
            const data = JSON.parse(require('fs').readFileSync(0, 'utf8'));
            const orig = JSON.parse(require('fs').readFileSync('package.json.bak', 'utf8'));
            const filtered = { ...orig, dependencies: data.filteredDependencies };
            delete filtered.devDependencies;
            require('fs').writeFileSync('package.json', JSON.stringify(filtered, null, 2));
        " <<< "$INSTALL_DATA"

        echo "Installing missing/incompatible packages..."
        npm install --production

        # Restore the original package.json so the project metadata stays intact
        mv package.json.bak package.json
        echo "✓ Project-specific dependencies installed"
    else
        echo "✓ All dependencies satisfied by base node_modules (no installation needed)"
        # Create empty node_modules directory if it doesn't exist (for consistency)
        mkdir -p node_modules
    fi
else
    echo "No package.json found, skipping dependency installation"
fi

echo "Node.js project prepared successfully"
