#!/bin/bash
# Archive project files and dependencies
# Handles both Python (the_venv) and Node.js (node_modules) projects
#
# Python projects:
#   - the_venv/ at /project/the_venv (project-specific dependencies)
#   - entrypoint file at /project/entrypoint
#   - project directory at /project/{project-name}
#
# Node.js projects:
#   - node_modules/ at /project/{project-name}/node_modules (project-specific dependencies)
#   - project directory at /project/{project-name} (includes node_modules automatically)
set -e

cd /project

# Build tar command with only existing files/directories
TAR_ARGS=()

# Python projects: include the_venv if it exists
# This contains project-specific Python dependencies installed during build
if [ -d "the_venv" ]; then
    TAR_ARGS+=("the_venv")
fi

# Python projects: include entrypoint file if it exists
if [ -f "entrypoint" ]; then
    TAR_ARGS+=("entrypoint")
fi

# Always include the project directory (passed as argument)
# For Python: contains source files (pyproject.toml, *.py, etc.)
# For Node.js: contains source files (package.json, *.js, etc.) AND node_modules/
# Note: node_modules is inside the project directory, so it's automatically included
if [ -n "$1" ] && [ -d "$1" ]; then
    TAR_ARGS+=("$1")
    # Verify node_modules exists for Node.js projects (informational only)
    if [ -d "$1/node_modules" ]; then
        echo "Found node_modules in project directory (Node.js project)"
    fi
elif [ -n "$1" ]; then
    echo "Warning: Project directory '$1' not found, but continuing..."
fi

# Create archive only if we have something to archive
if [ ${#TAR_ARGS[@]} -gt 0 ]; then
    echo "Archiving: ${TAR_ARGS[*]}"
    tar czvf /tmp/project.tar.gz "${TAR_ARGS[@]}"
    echo "✓ Archive created successfully at /tmp/project.tar.gz"
else
    echo "ERROR: No files to archive found"
    exit 1
fi
