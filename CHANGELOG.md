# Changelog

All notable changes to ServiceMaker will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Express Application Support
- **Express Project Detection**: Added automatic detection of Express.js applications
  - Detects Express apps by checking for `express` dependency in `package.json`
  - Requires absence of `services.json` and `manifest.json` (distinguishes from Foxx services)
  - Project type: `express`
  
- **Express Dockerfile Template**: Created `Dockerfile.express.template` for Express applications
  - Uses Node.js 22 base image (`arangodb/node22base:latest`)
  - Runs Express apps directly with `node {ENTRYPOINT}` instead of `node-foxx`
  - No `services.json` or `manifest.json` required
  - Entrypoint auto-detected from `package.json` `main` field or `start` script
  - Defaults to `index.js` if not found

- **Express Preparation Script**: Added `scripts/prepareproject-express.sh`
  - Similar to Node.js preparation but without `node-foxx` checks
  - Installs only missing/incompatible dependencies
  - Uses base `node_modules` from base image via NODE_PATH

- **Environment Variables from `.env.example`**: Added support for reading environment variables
  - Automatically reads `.env.example` file if present in project root
  - Parses `KEY=VALUE` format with support for quoted values
  - Injects environment variables as `ENV` directives in Dockerfile
  - Supports comments (lines starting with `#`) and empty lines
  - Handles values with spaces or special characters (auto-quotes for Docker)
  - Works for all project types (Python, Foxx, Express)

#### Node.js/Foxx Service Support
- **Node.js Base Image**: Added `Dockerfile.node22base` for Node.js 22 base image with pre-installed packages
  - Installs Node.js 22 from NodeSource
  - Pre-installs ArangoDB packages: `@arangodb/node-foxx`, `@arangodb/node-foxx-launcher`, `@arangodb/arangodb`
  - Pre-installs standard packages with version pinning: `lodash`, `dayjs`, `uuid`, `dotenv`, `axios`, `joi`, `winston`, `async`, `jsonwebtoken`, `bcrypt`, `semver`
  - Creates base `node_modules` at `/home/user/node_modules` with checksums for dependency tracking
  - Base image is immutable and pre-scanned for security vulnerabilities
  - Added to `baseimages/imagelist.txt` as `node22base`

- **Node.js Dockerfile Template**: Created `Dockerfile.nodejs.template` for building Node.js/Foxx service images
  - Copies service directory directly to `/project/{service-name}/`
  - Configures working directory and user permissions
  - Sets NODE_PATH environment variable for module resolution
  - Executes `prepareproject-nodejs.sh` for dependency management

- **Dependency Management Script**: Added `scripts/prepareproject-nodejs.sh` and `scripts/check-base-dependencies.js`
  - Base `node_modules` at `/home/user/node_modules` is immutable and never copied
  - Pre-install check: `check-base-dependencies.js` analyzes project dependencies against base packages
  - Version compatibility: Uses `semver` to verify if base package versions satisfy project requirements
  - Avoids duplication: Only installs packages that are missing or have incompatible versions
  - Uses NODE_PATH for module resolution (project first, then base)
  - Verifies `node-foxx` binary accessibility from either location
  - Keeps base image immutable for security scanning
  - Results in smaller project `node_modules` and `project.tar.gz` files

- **Project Type Detection**: Extended `detect_project_type()` to support:
  - `python`: Projects with `pyproject.toml`
  - `express`: Express.js applications with `express` dependency, no `services.json` or `manifest.json`
  - `foxx`: Multi-service projects with `package.json` and `services.json` (both required)
  - `foxx-service`: Single service directory with `package.json` only (auto-generates `services.json`)

- **Service Structure Generation**: Simplified structure for single service directories
  - Copies service directory directly to `/project/{service-name}/` (no wrapper folder)
  - Generates `services.json` automatically with mount path "/" and basePath "." in the service directory
  - `package.json` and `services.json` are in the same directory where `node_modules` will be created

- **Services JSON Generation**: Added `generate_services_json()` function
  - Automatically generates `services.json` for single service directories (`foxx-service` type)
  - Configures mount path as "/" (routing handled by Helm chart at deployment)
  - Sets basePath to "." (relative to WORKDIR where `node-foxx` runs)

- **Package.json Support**: Added functions to read Node.js project metadata
  - `read_name_from_package_json()`: Extracts project name from `package.json`
  - `read_service_info_from_package_json()`: Extracts name and version for Helm charts

- **Entrypoint Enhancement**: Updated `baseimages/scripts/entrypoint.sh` to support Node.js/Foxx services
  - Detects Foxx services by checking for both `package.json` and `services.json`
  - Automatically runs `node-foxx` for Foxx services (checks project `node_modules` first, then base)
  - Only supports Foxx services (requires both `package.json` and `services.json`)
  - Maintains backward compatibility with Python services

- **Test Service**: Added `itzpapalotl-node` test service in `testprojects/`
  - Example Node.js/Foxx service for testing ServiceMaker functionality
  - Demonstrates service structure generation and dependency management

### Changed

- **Main Application Logic**: Extended `src/main.rs` to support Node.js/Foxx and Express projects
  - Added Express project type detection and handling
  - Added entrypoint auto-detection for Express apps from `package.json`
  - Added environment variable reading from `.env.example` for all project types
  - Added Dockerfile modification functions for Express apps (`modify_dockerfile_express`)
  - Added Express preparation script to embedded scripts list
  - Added project type detection requiring both `package.json` and `services.json` for `foxx` type
  - Simplified file copying: projects are copied as-is (no wrapper structure generation)
  - Base image default handling: Introduced compile-time constants (`DEFAULT_PYTHON_BASE_IMAGE`, `DEFAULT_NODEJS_BASE_IMAGE`)
  - Explicit user intent tracking: Changed `base_image` to `Option<String>` to detect explicit user choices
  - Smart defaults: Only sets project-type-specific defaults when user hasn't explicitly set base image
  - Modified Dockerfile generation to use appropriate template based on project type
  - Updated Helm chart generation to support all project types (Python, Express, Foxx)
  - Added `prepareproject-express.sh` to embedded scripts list
  - No entrypoint required for Foxx services (uses `node-foxx` from base image)

- **Entrypoint Script**: Enhanced `baseimages/scripts/entrypoint.sh` to support Node.js/Foxx services
  - Added service type detection based on project files
  - Maintains backward compatibility with Python services

- **Base Image List**: Updated `baseimages/imagelist.txt` to include `node22base`

- **File Copying Logic**: Updated `copy_dir_recursive()` to skip `node_modules` directories
  - Prevents copying local `node_modules` which should be installed in Docker build
  - Ensures only project source code is copied, dependencies are installed fresh in Docker

### Fixed

- **Windows Compatibility**: Fixed Windows build issues in `src/main.rs`
  - Added conditional compilation for Unix-specific file permissions (`#[cfg(unix)]`)
  - Windows builds now skip `set_mode()` calls that are Unix-only

### Technical Details

For detailed information about:
- **Node.js/Foxx Services**: Base image structure, service architecture, and module resolution - see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)
- **Express Applications**: Express app architecture, detection, and deployment - see [docs/ARCHITECTURE_EXPRESS.md](docs/ARCHITECTURE_EXPRESS.md)
- **Service Comparison**: Differences between Node-Foxx and Express+Arangojs services - see [docs/SERVICE_COMPARISON.md](docs/SERVICE_COMPARISON.md)

## [0.9.2] - Previous Release

### Existing Features
- Python service support with `pyproject.toml`
- Base image management for Python 3.13
- Docker image building and pushing
- Helm chart generation
- Project tar.gz creation
- Virtual environment management with `uv`

---

## Version History

- **0.9.2**: Initial release with Python support
- **Unreleased**: Added Node.js/Foxx service support and Express application support

---

## Notes

- All changes maintain backward compatibility with existing Python projects
- Node.js support (both Foxx and Express) is additive and does not affect Python service functionality
- Express applications are detected automatically and require no special configuration files
- Environment variables from `.env.example` are automatically injected into Docker images
- Base images must be built separately using `baseimages/build.sh`
- Windows users should use WSL or Linux environment for building base images

