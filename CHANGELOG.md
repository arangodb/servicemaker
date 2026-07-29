# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- CI: Bump the `trivy-scan` orb 0.0.3 → 1.1.2, pinned exactly (a float is resolved at config-compile time, so an orb publish would change what runs on the publish path with no repo diff). The vulnerability DB is now cached under a daily-rotating key (the old static key never refreshed, because CircleCI caches are immutable) and pulled via the mirror.gcr.io → ECR → ghcr.io registry chain. New `dependency-cve-scan` filesystem scan runs on every pipeline with `fail-on-findings: true`, gating on fixable CRITICAL/HIGH CVEs, and names its three dependency manifests with `expect-targets` so a manifest that moves or disappears fails by name instead of shrinking the gate silently.
- CI: **The image scans no longer depend on the `security_scan` pipeline parameter.** They used to live only in a workflow conditioned on that parameter, which defaults to false, so nothing scanned the published images on a push, a PR, an API trigger or any branch the "Daily" schedule does not name. All six image gates (`py12base`, `py12cugraph`, `py12torch`, `node22base`, `test-service`, `test-service-nodejs`) now also run in the default workflow, so a Dockerfile change is scanned before it merges.
- CI: Each image gate is now a single scan with a narrow gate band and a wide report band (`report-severity` CRITICAL→UNKNOWN), replacing the previous second report-only pass. The second pass re-scanned the same image and overwrote the gate's results at the orb's fixed output paths, and it could not simply be reordered: it ran after the gate, so on a red gate the job ended first and the report never ran at all. Each gate pins `scanners: vuln`, fails on an end-of-life base OS (`exit-on-eol`), stores a CycloneDX SBOM and correlates its findings against the CISA KEV catalogue and EPSS (report-only). Secret detection over the image filesystem and the image config (`--image-config-scanners secret`) runs as a separate report-only pass in the same job, ordered before the gate. Trivy's default scanner set for an image includes secrets, and the first run showed what leaving that inside a blocking gate costs: `py12cugraph` ships tornado's own test fixture key in the uv archive cache, which failed a vulnerability gate on a third-party fixture. That path now carries a dated allow-rule.
- CI: `rebuild-base-images-manual` now scans each base image with the same CRITICAL/HIGH gate between `make build` and `make push`, and the push job requires `dependency-cve-scan`, `misconfig-scan` and `disposition-check` to pass first, so nothing reaches Docker Hub from a tree with an expired waiver or a failing gate.
- CI: New gates in the default workflow: `misconfig-scan` (Trivy misconfiguration over every Dockerfile and both service templates, plus the final-stage build-credential guard), `sast-scan` and `sast-scan-diff` (Semgrep CE, `p/default` + `p/rust`, ERROR full-repo and WARNING on newly introduced findings), `disposition-check` (waiver and disposition integrity, blocking), `secret-scan` (report-only) and `required-checks-drift` (warn-only). The nightly adds a full-severity dependency report including dev dependencies, a MEDIUM misconfiguration report, and `nightly-self-test`, which scans a digest-pinned known-vulnerable image and fails if detection has stopped working.
- CI: `misconfig-scan` passes `file-patterns` for `Dockerfile.nodejs.template`; Trivy's dockerfile analyzer does not recognise that filename, so the Node.js service template was silently unscanned.
- CI: `dependency-cve-scan` now sets `ignore-unfixed: true` explicitly (previously relied on the orb's default), matching the explicit setting already used by every image-scan job.
- `baseimages/Dockerfile.py12torch` restates the `USER user` it already inherits from `arangodb/py12base`. No build-time change; Trivy cannot see a parent image's USER, so it reported DS-0002 as a HIGH against this file.

### Added

- `SECURITY.md`: reporting channel and response times, the two-tier gate policy, per-severity remediation windows including the KEV row, the required-check list and why the strings are unstable, the honest SLSA Build L0 statement for the published images, and a Known gaps table with the price of each paid alternative.
- `.circleci/security-waivers.yaml`: structured accepted-risk waivers, subtracted at gate time only so the published report and SBOM keep every finding. Validated on every pipeline: schema, scope, severity window, expiry, and an `approved-by` that must resolve to `.github/CODEOWNERS`.
- `.circleci/trivy-secret.yaml`: disables Trivy's built-in `tests` allow-rule, which otherwise excludes every test and fixture tree from secret detection entirely.
- `.semgrepignore` plus a scope-shrinkage guard in both directions, so a directory cannot be removed from SAST scope without a reviewed change.
- `.github/CODEOWNERS`, `.github/required-checks.txt` and `.security/baseline.yml` (per-control posture, with the unproven and blocked controls named).

### Security

- Patched npm's bundled `sigstore` to 4.1.1 in the Node.js 22 base image (CVE-2026-48815)
- Added `sigstore` override to the Node.js test service `package.json`
- Upgraded `axios` to 1.18.0 in the Node.js base image (GHSA-gcfj-64vw-6mp9)
- Bumped `tar` override to 7.5.19 (CVE-2026-59873, CVE-2026-59874)
- Added `brace-expansion` override to 2.1.2 (CVE-2026-13149)
- Patched npm's bundled `tar` and `brace-expansion` in the Node.js 22 base image (full-image Trivy scans)
- Dispositioned CVE-2026-14257 (`brace-expansion`, HIGH, denial of service) with a dated waiver expiring 2026-08-28. It appears twice in `node22base`, and therefore in `test-service-nodejs` too: inside npm's own bundled dependency tree in the `node:22` layer, and in `/home/user/node_modules` from the image's own global install, where `overrides.brace-expansion=2.1.2` pins it. GHSA-mh99-v99m-4gvg models one vulnerable range, `<= 5.0.7`, first fixed in 5.0.8, and npm's bundled `minimatch` declares `brace-expansion` `^2.0.1` (10.2.5 is the first release to accept `^5.0.5`), so no 2.x release clears either copy and the bundled one cannot move at all until upstream npm ships `minimatch` 10.2.5. Moving the override to 5.0.8 for the global install is the near-term action; it is a major bump against the image and needs a rebuild plus a smoke test. The 2026-07-29 nightly went red on this finding.
- Added `tar` and `brace-expansion` overrides to the Node.js test service `package.json`

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
