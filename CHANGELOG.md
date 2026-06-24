# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [1.1.0] - 2026-06-24

### Added

- Node.js 22 project support with automatic detection from `package.json`
- Node.js base image (`arangodb/node22base:latest`) with pre-installed common packages
- Smart Node.js dependency resolution that installs only packages missing from or incompatible with the base image
- Automatic Node.js entrypoint detection from `package.json`
- Environment variable injection from `.env.example` files
- Python 3.12 base images (`py12base`, `py12cugraph`, `py12torch`)
- Node.js reference test service (`arango-test-service-nodejs`)
- Runtime project archive download for BYOC deployments via `ARCHIVE_FILE` or `projectURL`
- NVIDIA GPU library path setup for cuGraph and PyTorch services
- Nightly Trivy security scan with parallel jobs for all base and test-service images
- Manual base image rebuild workflow in CircleCI

### Changed

- Default Python base image changed from 3.13 to 3.12
- Base images migrated to `ubuntu:24.04` with layered builds
- Helm charts updated for Node.js services and auth labels
- Security scanning migrated from Grype to Trivy
- Entrypoint script supports both Python and Node.js services
- Archive creation (`--make-tar-gz`) supports Node.js projects

### Removed

- Python 3.13 base images (`py13base`, `py13cugraph`, `py13torch`)

### Fixed

- Integration test failures for base images
- CI `security-scan-notify` and `rebuild-base-images-manual` workflows
- In-container Trivy scan path visibility issues
- Integration test preparation script issues

### Security

- Run `apt-get upgrade` during base image builds to patch OS-level vulnerabilities
- Upgraded `axios` to 1.16.0 in the Node.js base image
- Patched vulnerabilities in test service dependencies and base images

## [1.0.0] - 2025-11-27

### Added

- Python reference test service (`arango-test-service`)
- CircleCI workflow for building and pushing Docker images to Docker Hub
- Makefile release targets and Docker Hub setup documentation

### Changed

- Security scan configuration updated for the test-service image

## [0.9.3] - 2025-11-27

### Added

- Automatic entrypoint detection when a project contains a single Python file
- Integration tests now pull base images explicitly and clean up old test directories

### Changed

- Helm route configuration prepends `/_services` to the Envoy mount path
- Upgraded Rust crate dependencies

## [0.9.1] - 2025-11-24

### Changed

- Entrypoint script no longer needs to be marked executable
- Updated Dockerfile template and entrypoint startup command handling

## [0.9.0] - 2025-11-21

### Added

- Python project support with Docker image building and optional registry push
- Python 3.13 base images with `uv` virtual environment management
- Helm chart generation for Kubernetes deployment
- Project archive creation with `--make-tar-gz`
- Nightly Grype security scan workflow with Slack notifications
- Integration test suite with Helm deployment validation
- CircleCI pipeline for building, testing, and scanning

[unreleased]: https://github.com/arangodb/servicemaker/compare/v1.1.0...HEAD
[1.1.0]: https://github.com/arangodb/servicemaker/compare/v1.0.0...v1.1.0
[1.0.0]: https://github.com/arangodb/servicemaker/compare/v0.9.3...v1.0.0
[0.9.3]: https://github.com/arangodb/servicemaker/compare/v0.9.1...v0.9.3
[0.9.1]: https://github.com/arangodb/servicemaker/compare/v0.9.0...v0.9.1
[0.9.0]: https://github.com/arangodb/servicemaker/releases/tag/v0.9.0
