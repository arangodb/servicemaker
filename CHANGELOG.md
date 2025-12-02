# Changelog

All notable changes to ServiceMaker will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

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
  - `foxx`: Multi-service projects with `package.json` and `services.json` (both required)
  - `foxx-service`: Single service directory with `package.json` only (auto-generates `services.json`)
  - Execution stops with error if `services.json` is missing for Node.js projects

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

- **Main Application Logic**: Extended `src/main.rs` to support Node.js/Foxx projects
  - Added project type detection requiring both `package.json` and `services.json` for `foxx` type
  - Error handling: execution stops if `services.json` is missing for Node.js projects
  - Simplified file copying: projects are copied as-is (no wrapper structure generation)
  - Base image default handling: Introduced compile-time constants (`DEFAULT_PYTHON_BASE_IMAGE`, `DEFAULT_NODEJS_BASE_IMAGE`)
  - Explicit user intent tracking: Changed `base_image` to `Option<String>` to detect explicit user choices
  - Smart defaults: Only sets project-type-specific defaults when user hasn't explicitly set base image
  - Modified Dockerfile generation to use Node.js template for Foxx projects
  - Updated Helm chart generation to support Node.js/Foxx projects
  - Added `prepareproject-nodejs.sh` and `check-base-dependencies.js` to embedded scripts list
  - No entrypoint required for Foxx services (uses `node-foxx` from base image)

- **Entrypoint Script**: Enhanced `baseimages/scripts/entrypoint.sh` to support Node.js/Foxx services
  - Added service type detection based on project files
  - Maintains backward compatibility with Python services

- **Base Image List**: Updated `baseimages/imagelist.txt` to include `node22base`

- **File Copying Logic**: Updated `copy_dir_recursive()` to skip `node_modules` directories
  - Prevents copying local `node_modules` which should be installed in Docker build
  - Ensures only project source code is copied, dependencies are installed fresh in Docker

### Fixed

- **services.json basePath**: Fixed incorrect basePath in generated `services.json`
  - Changed from service name to "." (current directory)
  - Fixes path resolution issue where `node-foxx` was looking in wrong location for `manifest.json`

- **Base Image Default Handling**: Improved robustness of base image default selection
  - Replaced magic string comparisons with compile-time constants
  - Added explicit tracking of user intent (whether base image was explicitly set)
  - Prevents breakage if default values change in the future

- **Windows Compatibility**: Fixed Windows build issues in `src/main.rs`
  - Added conditional compilation for Unix-specific file permissions (`#[cfg(unix)]`)
  - Windows builds now skip `set_mode()` calls that are Unix-only

### Technical Details

For detailed information about base image structure, service architecture, and module resolution, see [docs/ARCHITECTURE.md](docs/ARCHITECTURE.md).

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
- **Unreleased**: Added Node.js/Foxx service support

---

## Notes

- All changes maintain backward compatibility with existing Python projects
- Node.js support is additive and does not affect Python service functionality
- Base images must be built separately using `baseimages/build.sh`
- Windows users should use WSL or Linux environment for building base images

