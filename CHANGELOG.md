# Changelog

All notable changes to ServiceMaker will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

#### Node.js/Foxx Service Support
- **Node.js Base Image**: Added `Dockerfile.node22base` for Node.js 22 base image with pre-installed ArangoDB packages
  - Installs Node.js 22 from NodeSource
  - Pre-installs `@arangodb/node-foxx`, `@arangodb/node-foxx-launcher`, and `@arangodb/arangodb` packages
  - Creates base `node_modules` with checksums for dependency tracking
  - Added to `baseimages/imagelist.txt` as `node22base`

- **Node.js Dockerfile Template**: Created `Dockerfile.nodejs.template` for building Node.js/Foxx service images
  - Supports wrapper structure for single service directories
  - Configures working directory and user permissions
  - Executes `prepareproject-nodejs.sh` for dependency management

- **Dependency Management Script**: Added `scripts/prepareproject-nodejs.sh`
  - Base `node_modules` at `/home/user/node_modules` is immutable and never copied
  - Installs only missing or incompatible packages to project's `node_modules`
  - Uses NODE_PATH for module resolution (project first, then base)
  - npm automatically handles version conflicts (project version takes precedence)
  - Verifies `node-foxx` binary accessibility from either location
  - Keeps base image immutable for security scanning

- **Project Type Detection**: Extended `detect_project_type()` to support:
  - `python`: Projects with `pyproject.toml`
  - `foxx`: Multi-service projects with `package.json` and `services.json`
  - `foxx-service`: Single service directory with `package.json` (creates wrapper structure)
  - `nodejs`: Generic Node.js projects

- **Wrapper Structure Generation**: Automatic wrapper creation for single service directories
  - Creates `wrapper/` directory structure
  - Copies service directory directly to `/project/{service-name}/`
  - Generates `services.json` automatically with mount path "/" in the service directory
  - `package.json` and `services.json` are in the same directory where `node_modules` will be created

- **CLI Arguments**:

- **Services JSON Generation**: Added `generate_services_json()` function
  - Automatically generates `services.json` for single service directories
  - Configures mount path as "/" and base path for Foxx services

- **Package.json Support**: Added functions to read Node.js project metadata
  - `read_name_from_package_json()`: Extracts project name from `package.json`
  - `read_service_info_from_package_json()`: Extracts name and version for Helm charts

- **Entrypoint Enhancement**: Updated `baseimages/scripts/entrypoint.sh` to support Node.js/Foxx services
  - Detects service type based on project files
  - Automatically runs `node-foxx` for Foxx services
  - Falls back to generic Node.js execution for non-Foxx services
  - Maintains backward compatibility with Python services

- **Test Service**: Added `itzpapalotl-node` test service in `testprojects/`
  - Example Node.js/Foxx service for testing ServiceMaker functionality
  - Demonstrates wrapper structure generation and dependency management

### Changed

- **Main Application Logic**: Extended `src/main.rs` to support Node.js projects
  - Added project type detection for Node.js/Foxx services
  - Updated file copying logic to handle wrapper structure
  - Modified Dockerfile generation to use appropriate template based on project type
  - Updated Helm chart generation to support Node.js projects
  - Added `prepareproject-nodejs.sh` to embedded scripts list

- **Entrypoint Script**: Enhanced `baseimages/scripts/entrypoint.sh` to support Node.js/Foxx services
  - Added service type detection based on project files
  - Maintains backward compatibility with Python services

- **Base Image List**: Updated `baseimages/imagelist.txt` to include `node22base`

- **File Copying Logic**: Updated `copy_dir_recursive()` to skip `node_modules` directories
  - Prevents copying local `node_modules` which should be installed in Docker build

### Fixed

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

