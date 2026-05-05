# Changelog

All notable changes to ServiceMaker will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Node.js Project Support

- **Node.js Base Image**: Added `baseimages/Dockerfile.node22base` for creating Node.js 22 base images
  - Base image `arangodb/node22base:latest` provides immutable foundation for all Node.js services
  - Installs Node.js 22 from NodeSource
  - Pre-installs common packages with version pinning: `arangojs@^10.2.2`, `semver@^7.6.3`, `lodash@^4.17.21`, `dayjs@^1.11.10`, `uuid@^9.0.1`, `dotenv@^16.4.5`, `axios@^1.7.2`, `joi@^17.13.3`, `winston@^3.15.0`, `async@^3.2.5`, `jsonwebtoken@^9.0.2`, `bcrypt@^5.1.1`
  - Creates base `node_modules` at `/home/user/node_modules` with SHA256 checksums for dependency tracking
  - Base image is immutable and pre-scanned for security vulnerabilities
  - Added to `baseimages/imagelist.txt` as `node22base`

- **Node.js Dockerfile Template**: Created `Dockerfile.nodejs.template` for building Node.js service images
  - Uses Node.js 22 base image (`arangodb/node22base:latest`)
  - Copies project directory directly to `/project/{project-name}/`
  - Configures working directory and user permissions
  - Sets `NODE_PATH` environment variable for module resolution (project `node_modules` first, then base)
  - Executes `prepareproject-nodejs.sh` for dependency management
  - Runs applications directly with `node {ENTRYPOINT}` command

- **Dependency Management System**: Added intelligent dependency resolution to avoid duplicating base packages
  - **`scripts/check-base-dependencies.js`**: Analyzes project dependencies against base packages
    - Checks if packages exist in base `node_modules` at `/home/user/node_modules`
    - Uses `semver` to verify version compatibility between base and project requirements
    - Outputs JSON with packages that need installation (missing or incompatible versions) and a `filteredDependencies` object for rewriting `package.json`
    - Provides detailed dependency analysis summary to stderr
  
  - **`scripts/prepareproject-nodejs.sh`**: Installs only missing or incompatible dependencies
    - Base `node_modules` at `/home/user/node_modules` is immutable and never copied
    - Pre-install analysis using `check-base-dependencies.js` to identify required packages
    - Temporarily rewrites `package.json` to contain only the missing dependencies before running `npm install --production`, then restores the original — this prevents npm 7+ from reconciling the full dependency tree and re-installing packages already present in the base image
    - Results in smaller project `node_modules` and `project.tar.gz` files
    - Maintains base image immutability for security scanning

- **Project Type Detection**: Extended `detect_project_type()` in `src/main.rs` to support Node.js projects
  - Detects Node.js projects by presence of `package.json` file
  - Requires absence of `services.json` and `manifest.json` (distinguishes from Foxx services)
  - Project type: `nodejs` (for Node.js applications)
  - Returns error if `services.json` or `manifest.json` is found (not supported)

- **Entrypoint Auto-Detection**: Added automatic entrypoint detection for Node.js projects
  - Function `detect_nodejs_entrypoint()` checks `package.json` `main` field first
  - Falls back to extracting from `start` script if `main` is not present
  - Supports scripts like `"start": "node index.js"` format
  - Provides sensible default (`index.js`) if detection fails

- **Package.json Metadata Support**: Added functions to read Node.js project metadata
  - `read_name_from_package_json()`: Extracts project name from `package.json`
  - `read_service_info_from_package_json()`: Extracts name and version for Helm charts
  - Used for auto-detecting project name and generating Helm charts

- **Environment Variable Support**: Added support for reading environment variables from `.env.example` files
  - Function `read_env_example()` automatically reads `.env.example` file if present in project root
  - Parses `KEY=VALUE` format with support for single and double quotes
  - Handles comments (lines starting with `#`) and empty lines
  - Auto-quotes values containing spaces or special characters for Docker `ENV` directives
  - Injects environment variables into Dockerfile for all project types (Python and Node.js)
  - Works seamlessly with existing project structures

- **Default Base Image Constants**: Introduced compile-time constants for default base images
  - `DEFAULT_PYTHON_BASE_IMAGE`: `"arangodb/py12base:latest"`
  - `DEFAULT_NODEJS_BASE_IMAGE`: `"arangodb/node22base:latest"`
  - Automatically selects appropriate default when user doesn't specify base image
  - Tracks explicit user intent to avoid overriding user choices

### Changed

- **Main Application Logic**: Extended `src/main.rs` to support Node.js projects
  - Added Node.js project type detection and handling in main function
  - Added entrypoint auto-detection for Node.js projects from `package.json`
  - Added environment variable reading from `.env.example` for all project types
  - Added Dockerfile modification function for Node.js projects (`modify_dockerfile_nodejs`)
  - Added Node.js preparation script (`prepareproject-nodejs.sh`) and dependency checker (`check-base-dependencies.js`) to embedded scripts list
  - Modified Dockerfile generation to use appropriate template based on project type (Python or Node.js)
  - Updated Helm chart generation to support both Python and Node.js projects
  - Enhanced project metadata extraction to support both `pyproject.toml` and `package.json`
  - Updated file copying logic to skip `node_modules` directories (prevents copying local dependencies)

- **Entrypoint Script**: Enhanced `baseimages/scripts/entrypoint.sh` to support Node.js services
  - Detects Node.js applications by checking for `package.json` without `services.json` or `manifest.json`
  - Automatically runs `node {ENTRYPOINT}` for Node.js applications
  - Maintains backward compatibility with Python services
  - Uses `NODE_PATH` environment variable for module resolution

- **Archive Creation Script**: Updated `scripts/zipper.sh` to handle both Python and Node.js projects
  - Includes `the_venv/` directory for Python projects
  - Includes project directory (which contains `node_modules/`) for Node.js projects
  - Provides informational messages when `node_modules` is detected
  - Enhanced documentation with clear comments explaining both project types

- **File Copying Logic**: Updated `copy_dir_recursive()` in `src/main.rs` to skip `node_modules` directories
  - Prevents copying local `node_modules` which should be installed fresh in Docker build
  - Ensures only project source code is copied, dependencies are installed in container
  - Maintains consistency with Python's `.venv` exclusion

- **Base Image List**: Updated `baseimages/imagelist.txt` to include `node22base` entry

---

## [0.9.2] - 2024-XX-XX

### Features

- Python service support with `pyproject.toml`
- Base image management for Python 3.13 (`arangodb/py12base:latest`)
- Docker image building and pushing
- Helm chart generation
- Project tar.gz creation
- Virtual environment management with `uv`
- Automatic entrypoint detection for Python projects (single .py file)
- Project metadata extraction from `pyproject.toml`

---

## Notes

- All changes maintain backward compatibility with existing Python projects
- Node.js support is additive and does not affect Python service functionality
- Node.js applications are detected automatically and require no special configuration files
- Environment variables from `.env.example` are automatically injected into Docker images
- Base images must be built separately using `baseimages/build.sh` before use
- Windows users should use WSL or Linux environment for building base images
- The dependency checking system ensures efficient builds by avoiding package duplication while maintaining compatibility
