# Architecture

This document describes the technical architecture and design decisions for ServiceMaker.

## Node.js/Foxx Services

### Base Image Structure

- Base `node_modules` located at `/home/user/node_modules` (immutable, pre-scanned)
- Checksums stored at `/home/user/sums_sha256` for dependency tracking
- Base packages: `@arangodb/node-foxx@^0.0.1-alpha.0`, `@arangodb/node-foxx-launcher@^0.0.1-alpha.0`, `@arangodb/arangodb@^0.0.1-alpha.0`
- Base image is never modified - projects install only missing/incompatible packages to their own `node_modules`

### Service Structure

```
/project/{service-name}/
├── services.json         # Auto-generated with mount path "/"
├── package.json          # Service package.json
├── node_modules/         # Project-specific packages ONLY (missing/incompatible)
└── ...                   # Service code files
```

### Module Resolution

- Base packages: `/home/user/node_modules` (immutable, pre-scanned)
- Project packages: `/project/{service-name}/node_modules` (mutable)
- NODE_PATH configured to resolve from project first, then base
- npm automatically installs only missing or incompatible packages
- Version conflicts handled automatically (project version takes precedence)

