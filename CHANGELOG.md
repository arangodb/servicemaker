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
  - Copies base `node_modules` from base image
  - Installs project-specific dependencies from `package.json`
  - Ensures `node-foxx` binary is always available with multiple safety checks
  - Tracks new dependencies using SHA256 checksums
  - Separates base packages from project packages
  - Includes automatic recovery mechanism if base packages are removed during `npm install`
  - Handles `package.json` copying to wrapper root for proper dependency installation

- **Project Type Detection**: Extended `detect_project_type()` to support:
  - `python`: Projects with `pyproject.toml`
  - `foxx`: Multi-service projects with `package.json` and `services.json`
  - `foxx-service`: Single service directory with `package.json` (creates wrapper structure)
  - `nodejs`: Generic Node.js projects

- **Wrapper Structure Generation**: Automatic wrapper creation for single service directories
  - Creates `wrapper/` directory structure
  - Copies service directory into `wrapper/{service-name}/`
  - Generates `services.json` automatically with mount path configuration
  - Copies `package.json` to wrapper root for dependency installation

- **CLI Arguments**:
  - `--mount-path`: Required for `foxx-service` type, specifies the mount path for the Foxx service

- **Services JSON Generation**: Added `generate_services_json()` function
  - Automatically generates `services.json` for single service directories
  - Configures mount path and base path for Foxx services

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

- **Base Image Structure**:
  - Base `node_modules` located at `/home/user/base_node_modules/node_modules`
  - Checksums stored at `/home/user/sums_sha256` for dependency tracking
  - Base packages: `@arangodb/node-foxx@^0.0.1-alpha.0`, `@arangodb/node-foxx-launcher@^0.0.1-alpha.0`, `@arangodb/arangodb@^0.0.1-alpha.0`

- **Wrapper Structure**:
  ```
  wrapper/
  ├── package.json          # Copied from service for npm install
  ├── services.json         # Auto-generated with mount path
  ├── node_modules/         # Installed dependencies (base + project)
  └── {service-name}/       # Service directory
      ├── package.json
      └── ...
  ```

- **Dependency Tracking**:
  - Uses SHA256 checksums to identify new files vs. base files
  - New project dependencies copied to `/project/node_modules/` for tracking
  - Base packages remain in base image for efficiency

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

