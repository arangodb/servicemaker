#!/bin/bash
# This is to be run in the project directory to install dependencies.
# It copies base node_modules from the base image and installs additional
# project dependencies, tracking changes similar to the Python script.

set -e

# We're already in /project/{PROJECT_DIR} from the Dockerfile WORKDIR
PROJECT_DIR=$(pwd)

# Copy base node_modules if they exist in base image
if [ -d "/home/user/base_node_modules/node_modules" ]; then
    echo "Copying base node_modules..."
    if [ -d "node_modules" ]; then
        # Merge with existing node_modules if any
        cp -r /home/user/base_node_modules/node_modules/* ./node_modules/ 2>/dev/null || true
    else
        cp -r /home/user/base_node_modules/node_modules ./node_modules
    fi
    
    # Verify node-foxx binary exists after copy
    if [ -f "node_modules/.bin/node-foxx" ]; then
        echo "✓ Base node-foxx binary copied successfully"
    else
        echo "Warning: node-foxx binary not found after copying base node_modules"
    fi
    
    # Track existing files
    cd /home/user
    if [ -f "sums_sha256" ]; then
        sha256sum -c sums_sha256 || true
    fi
    cd "$PROJECT_DIR"
else
    echo "Warning: Base node_modules not found at /home/user/base_node_modules/node_modules"
fi

# Install additional project dependencies if package.json exists
if [ -f "package.json" ]; then
    echo "Installing project dependencies..."
    
    # Always ensure base packages are installed first (they provide node-foxx binary)
    # This ensures node-foxx is available even if npm install does a clean install
    echo "Ensuring base node-foxx packages are installed..."
    npm install --production --no-save \
        @arangodb/node-foxx@^0.0.1-alpha.0 \
        @arangodb/node-foxx-launcher@^0.0.1-alpha.0 \
        @arangodb/arangodb@^0.0.1-alpha.0 || {
        echo "Warning: Failed to install base packages, continuing anyway..."
    }
    
    # Verify node-foxx exists after installing base packages
    if [ ! -f "node_modules/.bin/node-foxx" ]; then
        echo "ERROR: node-foxx binary not found after installing base packages!"
        echo "Listing node_modules/.bin contents:"
        ls -la node_modules/.bin/ 2>/dev/null || echo "node_modules/.bin directory does not exist"
        exit 1
    fi
    echo "✓ Base node-foxx packages installed"
    
    # Install project dependencies (this should preserve base packages)
    npm install --production --no-save
    
    # Final verification that node-foxx still exists
    if [ -f "node_modules/.bin/node-foxx" ]; then
        echo "✓ node-foxx binary exists after installing project dependencies"
    else
        echo "ERROR: node-foxx binary missing after npm install!"
        echo "Reinstalling base packages..."
        npm install --production --no-save \
            @arangodb/node-foxx@^0.0.1-alpha.0 \
            @arangodb/node-foxx-launcher@^0.0.1-alpha.0 \
            @arangodb/arangodb@^0.0.1-alpha.0
        
        # Final check
        if [ ! -f "node_modules/.bin/node-foxx" ]; then
            echo "ERROR: Failed to install node-foxx binary!"
            echo "Current directory: $(pwd)"
            echo "Listing node_modules/.bin contents:"
            ls -la node_modules/.bin/ 2>/dev/null || echo "node_modules/.bin directory does not exist"
            echo "Listing node_modules contents:"
            ls -la node_modules/ 2>/dev/null | head -20
            exit 1
        fi
        echo "✓ node-foxx binary restored"
    fi
    
    # Find all files in node_modules and create checksums
    find node_modules -type f -print0 | xargs -0 sha256sum > /tmp/node_modules_sha256_new 2>/dev/null || true
    
    # Find new files (files in current node_modules that weren't in base)
    if [ -f "/home/user/sums_sha256" ] && [ -f "/tmp/node_modules_sha256_new" ]; then
        cat /tmp/node_modules_sha256_new /home/user/sums_sha256 | sort | uniq -c | grep "^      1 " | awk '{ print $3 }' > /tmp/newfiles || true
    else
        # If no base checksums, all files are new
        if [ -f "/tmp/node_modules_sha256_new" ]; then
            awk '{ print $3 }' /tmp/node_modules_sha256_new > /tmp/newfiles || true
        fi
    fi
    
    # Move new files to /project/node_modules (similar to Python approach)
    if [ -f "/tmp/newfiles" ]; then
        mkdir -p /project/node_modules
        while IFS= read -r filename; do
            # Skip empty lines
            [[ -z "$filename" ]] && continue
            
            # Check if source file exists and is relative to current dir
            if [[ "$filename" == node_modules/* ]] && [ -f "$filename" ]; then
                # Get the directory part of the filename
                DIR=$(dirname "$filename")
                
                # Create the destination directory structure
                mkdir -p "/project/$DIR"
                
                # Copy the file preserving the directory hierarchy
                cp "$filename" "/project/$filename"
            fi
        done < "/tmp/newfiles"
    fi
    
    rm -f /tmp/node_modules_sha256_new /tmp/newfiles
fi

echo "Node.js project prepared successfully"

