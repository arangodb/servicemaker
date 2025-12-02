#!/bin/bash
# This script installs only missing project dependencies to the project's node_modules.
# Base node_modules at /home/user/node_modules is immutable and never copied.
# npm automatically resolves from both locations via NODE_PATH.

set -e

# We're in /project/{service-name} (WORKDIR)
# services.json is in the current directory
# package.json is in the current directory
# node_modules will be created in the current directory
PROJECT_DIR=$(pwd)

# Verify base node_modules exists (immutable, pre-scanned)
if [ ! -d "/home/user/node_modules" ]; then
    echo "ERROR: Base node_modules not found at /home/user/node_modules"
    exit 1
fi

echo "Base node_modules found at /home/user/node_modules (immutable)"

# Verify base node-foxx binary exists
if [ -f "/home/user/node_modules/.bin/node-foxx" ]; then
    echo "✓ Base node-foxx binary available"
else
    echo "WARNING: node-foxx binary not found in base node_modules"
fi

# Install project dependencies if package.json exists
if [ -f "package.json" ]; then
    echo "Installing project dependencies..."
    echo "npm will automatically:"
    echo "  - Check base node_modules at /home/user/node_modules"
    echo "  - Install only missing or incompatible packages"
    echo "  - Handle version conflicts (project version takes precedence)"
    
    # npm install will:
    # 1. Check if packages exist in base (/home/user/node_modules)
    # 2. Compare versions with package.json requirements
    # 3. Install only missing or incompatible packages to ./node_modules
    # 4. Handle version conflicts automatically
    npm install --production --no-save
    
    echo "✓ Project dependencies installed"
    
    # Verify node-foxx is accessible (either from base or project node_modules)
    if [ -f "node_modules/.bin/node-foxx" ]; then
        echo "✓ node-foxx binary found in project node_modules"
    elif [ -f "/home/user/node_modules/.bin/node-foxx" ]; then
        echo "✓ node-foxx binary available from base node_modules (will be resolved via NODE_PATH)"
    else
        echo "ERROR: node-foxx binary not found in base or project node_modules!"
        exit 1
    fi
else
    echo "No package.json found, skipping dependency installation"
fi

echo "Node.js project prepared successfully"

