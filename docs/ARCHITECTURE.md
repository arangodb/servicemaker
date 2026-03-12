# Architecture

This document describes the technical architecture and design decisions for ServiceMaker.

## Supported Project Types

ServiceMaker supports two project types:
- **Python**: Projects with `pyproject.toml`
- **Node.js/Express**: Projects with `package.json` (no `services.json` or `manifest.json`)

## Node.js/Express Services

### Base Image Structure

The base image (`arangodb/node22base:latest`) provides an immutable foundation for all Node.js/Express services:

**File System Layout:**
```
/home/user/
├── node_modules/          # Immutable base packages (pre-scanned for security)
│   ├── lodash@^4.17.21
│   ├── dayjs@^1.11.10
│   ├── axios@^1.7.2
│   ├── joi@^17.13.3
│   ├── winston@^3.15.0
│   ├── semver@^7.6.3      # Required for dependency checking
│   └── ... (other standard packages)
└── sums_sha256            # SHA256 checksums of all base node_modules files
```

**Key Properties:**
- **Immutability**: Base `node_modules` is never modified after image creation
- **Pre-scanning**: Base packages are security-scanned before deployment
- **Version Pinning**: All base packages use caret ranges (e.g., `^4.17.21`) for reproducible builds
- **Checksum Tracking**: `sums_sha256` file enables change detection and verification

### Service Structure

Each Node.js/Express service is deployed with the following structure:

```
/project/{project-name}/
├── package.json          # Project dependencies and metadata
├── node_modules/         # Project-specific packages ONLY
│                         # Contains only packages that are:
│                         # - Missing from base node_modules
│                         # - Have incompatible versions with base
└── ...                   # Service code files (index.js, routes, etc.)
```

### Dependency Resolution Algorithm

The dependency resolution process ensures no duplication while maintaining compatibility:

**Phase 1: Pre-Install Analysis (`check-base-dependencies.js`)**

1. **Read Project Dependencies**: Parse `package.json` to extract all `dependencies`
2. **Check Base Availability**: For each dependency:
   - Check if package exists at `/home/user/node_modules/{package-name}/`
   - Read version from base package's `package.json`
3. **Version Compatibility Check**: Use `semver.satisfies()` to verify:
   - Base version satisfies project's version range (e.g., `^4.17.21` satisfies `^4.17.0`)
   - If satisfied → skip installation (use base version)
   - If not satisfied → add to install list
4. **Output**: JSON array of packages to install (missing or incompatible)

**Phase 2: Selective Installation (`prepareproject-nodejs.sh`)**

1. **Parse Install List**: Extract package specifications from JSON output
2. **Install Missing/Incompatible**: Run `npm install --production --no-save` for each:
   ```bash
   npm install --production --no-save package-name@version-range
   ```
3. **Result**: Project `node_modules` contains only packages not available/compatible in base

**Example Scenario:**
```
Project requires: lodash@^4.17.0, axios@^1.7.0, custom-pkg@^1.0.0
Base has:        lodash@4.17.21, axios@1.7.2

Result:
- lodash: ✓ Base version 4.17.21 satisfies ^4.17.0 → Use base
- axios:  ✓ Base version 1.7.2 satisfies ^1.7.0 → Use base  
- custom-pkg: ✗ Not in base → Install to project/node_modules
```

### Module Resolution at Runtime

Node.js module resolution uses `NODE_PATH` environment variable:

**Configuration:**
```dockerfile
ENV NODE_PATH=/project/{project-name}/node_modules:/home/user/node_modules
```

**Resolution Order:**
1. **Project `node_modules`** (checked first)
   - Contains project-specific packages
   - Takes precedence for version conflicts
2. **Base `node_modules`** (checked second)
   - Contains standard packages
   - Used when package not found in project

**Runtime Behavior:**
- `require('lodash')` → Resolves from base (if compatible version exists)
- `require('custom-pkg')` → Resolves from project (not in base)
- `require('axios')` → Resolves from base (if compatible) or project (if incompatible version installed)

### Build Process Flow

**Dockerfile Build Steps:**

1. **Base Image**: `FROM arangodb/node22base:latest`
2. **Copy Scripts**: Embed `prepareproject-nodejs.sh` and `check-base-dependencies.js`
3. **Copy Project**: Copy project directory to `/project/{project-name}/`
   - Local `node_modules` are excluded (not copied)
4. **Set Working Directory**: `WORKDIR /project/{project-name}`
5. **Configure NODE_PATH**: Set environment variable for module resolution
6. **Run Preparation Script**: Execute `prepareproject-nodejs.sh`
   - Analyzes dependencies
   - Installs only missing/incompatible packages
7. **Set Entrypoint**: `CMD ["node", "{ENTRYPOINT}"]`

**Script Execution Flow:**

```
prepareproject-nodejs.sh
├── Verify base node_modules exists
├── Run check-base-dependencies.js
│   ├── Parse package.json
│   ├── Check each dependency against base
│   ├── Verify version compatibility (semver)
│   └── Output JSON: packages to install
├── Parse JSON output
└── Install missing/incompatible packages
```

### Runtime Execution

**Container Startup:**

1. **Entrypoint**: `node {ENTRYPOINT}` (e.g., `node index.js`)
2. **Working Directory**: `/project/{project-name}/` (where `package.json` is located)
3. **Module Resolution**: Uses `NODE_PATH` to resolve dependencies from both locations
4. **Service Launch**: Node.js/Express application starts with access to both base and project packages

### Security Considerations

**Base Image Scanning:**
- Base `node_modules` is pre-scanned for vulnerabilities before deployment
- Checksums (`sums_sha256`) enable change detection
- Immutability ensures base packages cannot be modified

**Project Package Scanning:**
- Only project-specific packages need scanning (smaller surface area)
- Project `node_modules` is mutable and can be scanned separately
- Version conflicts resolved by installing project version (explicit choice)

**Benefits:**
- Reduced scan time (only scan project packages)
- Base image can be pre-approved and reused
- Clear separation between base (trusted) and project (variable) packages

### Performance Implications

**Build Time:**
- Faster builds: Only install missing/incompatible packages
- Reduced network: Fewer packages to download
- Smaller layers: Project `node_modules` is minimal

**Runtime:**
- Faster startup: Fewer packages to load
- Smaller images: Reduced image size
- Efficient resolution: NODE_PATH lookup is fast (filesystem-based)

**Storage:**
- Smaller `project.tar.gz`: Only project-specific packages archived
- Base image reuse: Single base image shared across all services
- Layer caching: Base image layers cached, only project layer changes

### Technical Constraints

**Version Compatibility:**
- Uses semantic versioning (semver) for compatibility checks
- Caret ranges (^) in base allow patch/minor updates
- Project can override with specific versions if needed

**NODE_PATH Limitations:**
- Only affects `require()` resolution, not `npm install` behavior
- Requires explicit pre-install check to avoid duplication
- Binary resolution: `.bin` executables must be in accessible `node_modules`

**File System:**
- Base `node_modules` must be read-only (immutability requirement)
- Project `node_modules` must be writable (for installation)
- Both locations must be accessible via NODE_PATH

## Python Services

### Base Image Structure

The base image (e.g., `arangodb/py13base:latest`) provides an immutable foundation for all Python services:

**File System Layout:**
```
/home/user/
├── the_venv/              # Python virtual environment
│   └── lib/python3.13/site-packages/
│       ├── python_arango/
│       ├── phenolrs/
│       ├── networkx/
│       └── ... (other pre-installed packages)
└── sums_sha256            # SHA256 checksums of all base packages
```

**Key Properties:**
- **Immutability**: Base virtual environment is never modified after image creation
- **Pre-scanning**: Base packages are security-scanned before deployment
- **Checksum Tracking**: `sums_sha256` file enables change detection and verification

### Service Structure

Each Python service is deployed with the following structure:

```
/project/{project-name}/
├── pyproject.toml         # Project dependencies and metadata
├── the_venv/             # Virtual environment changes ONLY
│                         # Contains only packages that are:
│                         # - Missing from base virtual environment
│                         # - Have incompatible versions with base
└── ...                   # Service code files (main.py, etc.)
```

### Dependency Resolution

Python services use `uv` package manager:
- `uv sync --active` installs only missing/incompatible dependencies
- Base virtual environment is reused and extended
- Only new packages are added to the virtual environment

### Build Process Flow

**Dockerfile Build Steps:**

1. **Base Image**: `FROM arangodb/py13base:latest`
2. **Copy Project**: Copy project directory to `/home/user/project/{project-name}/`
3. **Set Working Directory**: `WORKDIR /home/user/project/{project-name}`
4. **Activate Virtual Environment**: Source the base virtual environment
5. **Install Dependencies**: Run `uv sync --active` to install only new dependencies
6. **Set Entrypoint**: `CMD ["python", "{ENTRYPOINT}"]`

### Runtime Execution

**Container Startup:**

1. **Entrypoint**: `python {ENTRYPOINT}` (e.g., `python main.py`)
2. **Working Directory**: `/home/user/project/{project-name}/`
3. **Virtual Environment**: Base virtual environment is activated
4. **Service Launch**: Python application starts with access to both base and project packages