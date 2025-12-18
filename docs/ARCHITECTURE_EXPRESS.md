# Express Application Architecture

This document describes the technical architecture and design decisions for Express.js applications in ServiceMaker.

## Overview

Express applications are standalone Node.js web services that use Express.js framework and Arangojs driver directly, without the Foxx framework. They are simpler and more flexible than Foxx services, requiring no `services.json` or `manifest.json` files.

## Project Detection

ServiceMaker automatically detects Express applications by checking:

1. **Presence of `package.json`**: Required for all Node.js projects
2. **Express dependency**: Checks if `express` is listed in `dependencies` or `devDependencies`
3. **Absence of Foxx files**: No `services.json` or `manifest.json` present

**Detection Logic:**
```rust
if package_json.exists() {
    if !services_json.exists() && !manifest_json.exists() {
        if has_express_dependency(package_json) {
            return "express"
        }
    }
}
```

## Project Structure

Express applications have a simple, standard Node.js structure:

```
project-root/
├── package.json          # Dependencies and metadata
├── index.js              # Main entry point (or custom)
├── .env.example          # Optional: Environment variable template
├── routes/               # Optional: Route handlers
├── middleware/           # Optional: Express middleware
├── config/               # Optional: Configuration files
└── ...                   # Other application files
```

**Key Differences from Foxx Services:**
- No `services.json` required
- No `manifest.json` required
- No `node-foxx` or `node-foxx-launcher` dependencies needed
- Standard Express.js application structure

## Base Image

Express applications use the same Node.js base image as Foxx services:

- **Base Image**: `arangodb/node22base:latest`
- **Node.js Version**: 22.x
- **Pre-installed Packages**: Standard Node.js packages (lodash, axios, joi, etc.)
- **Module Resolution**: Uses NODE_PATH for efficient dependency management

## Entrypoint Detection

ServiceMaker automatically detects the entrypoint for Express applications:

**Priority Order:**
1. `package.json` `main` field (e.g., `"main": "index.js"`)
2. `package.json` `scripts.start` field (extracts from `"node index.js"` → `"index.js"`)
3. Default: `"index.js"`

**Example:**
```json
{
  "name": "my-express-app",
  "main": "server.js",
  "scripts": {
    "start": "node server.js"
  }
}
```
→ Entrypoint: `server.js`

## Dockerfile Structure

Express applications use `Dockerfile.express.template`:

```dockerfile
FROM arangodb/node22base:latest

USER root
COPY ./scripts /scripts
COPY {PROJECT_DIR} /project/{PROJECT_DIR}
RUN chown -R user:user /project/{PROJECT_DIR}

USER user
WORKDIR /project/{WORKDIR}

ENV NODE_PATH=/project/{PROJECT_DIR}/node_modules:/home/user/node_modules

RUN /scripts/prepareproject-express.sh

EXPOSE {PORT}

CMD ["node", "{ENTRYPOINT}"]
```

**Key Features:**
- Runs directly with `node` (not `node-foxx`)
- Uses same dependency management as Foxx services
- Environment variables from `.env.example` are injected as `ENV` directives

## Dependency Management

Express applications use the same efficient dependency management as Foxx services:

**Process:**
1. **Pre-install Analysis**: `check-base-dependencies.js` analyzes project dependencies
2. **Version Compatibility**: Checks if base packages satisfy project requirements
3. **Selective Installation**: Only installs missing or incompatible packages
4. **Module Resolution**: Uses NODE_PATH to resolve from both locations

**Benefits:**
- Smaller `node_modules` (only project-specific packages)
- Faster builds (fewer packages to install)
- Base packages pre-scanned for security

## Environment Variables

Express applications support environment variables from `.env.example`:

**File Format:**
```
# Database configuration
ARANGO_DB_ENDPOINT=http://127.0.0.1:8529
ARANGO_DB_NAME=_system

# Application settings
PORT=3000
NODE_ENV=production

# Optional: Values with spaces
APP_NAME="My Express App"
```

**Processing:**
- Comments (lines starting with `#`) are ignored
- Empty lines are skipped
- Quoted values (single or double quotes) are unquoted
- Values with spaces or special characters are auto-quoted for Docker

**Dockerfile Injection:**
```dockerfile
ENV NODE_PATH=/project/{PROJECT_DIR}/node_modules:/home/user/node_modules
ENV ARANGO_DB_ENDPOINT=http://127.0.0.1:8529
ENV ARANGO_DB_NAME=_system
ENV PORT=3000
ENV NODE_ENV=production
ENV APP_NAME="My Express App"
```

## Runtime Execution

**Container Startup:**
1. **Entrypoint**: `node {ENTRYPOINT}` (e.g., `node index.js`)
2. **Working Directory**: `/project/{project-dir}/`
3. **Module Resolution**: Uses NODE_PATH to resolve dependencies
4. **Environment Variables**: Available from Docker ENV directives

**Example:**
```bash
# Container runs:
cd /project/my-express-app
node index.js
```

## Comparison with Foxx Services

| Feature | Express Apps | Foxx Services |
|---------|-------------|---------------|
| **Framework** | Express.js | Foxx (node-foxx) |
| **Orchestration** | None (standalone) | node-foxx-launcher |
| **Configuration** | Standard Express | manifest.json + services.json |
| **Entrypoint** | `node {file}` | `node-foxx` |
| **Multi-service** | No | Yes (via services.json) |
| **Worker Threads** | No | Yes (per service) |
| **Routing** | Express router | Foxx router |
| **API Docs** | Manual (Swagger) | Auto-generated (Swagger) |
| **Dependencies** | Standard npm | Foxx + npm |

## Use Cases

**Choose Express when:**
- You want standard Express.js patterns
- You don't need Foxx-specific features (manifest, services.json)
- You prefer simpler, more flexible architecture
- You want full control over application structure
- You're migrating from existing Express applications

**Choose Foxx when:**
- You need multi-service orchestration
- You want automatic API documentation
- You need Foxx-specific features (context, dependencies)
- You're building ArangoDB-native services
- You want worker thread isolation

## Best Practices

1. **Project Structure**: Follow Express.js best practices
   - Separate routes, middleware, and controllers
   - Use environment variables for configuration
   - Implement proper error handling

2. **Dependencies**: Minimize project-specific packages
   - Use base image packages when possible
   - Check base packages before adding new dependencies

3. **Environment Variables**: Use `.env.example` for documentation
   - Document all required environment variables
   - Provide default values where appropriate
   - Keep sensitive values out of version control

4. **Entrypoint**: Use `package.json` `main` field
   - Makes entrypoint explicit and discoverable
   - Works with standard Node.js tooling

5. **Error Handling**: Implement comprehensive error handling
   - Use Express error handling middleware
   - Return consistent error responses
   - Log errors appropriately

## Migration from Foxx

If you have an existing Foxx service and want to migrate to Express:

1. **Remove Foxx Dependencies**: Remove `@arangodb/node-foxx` and related packages
2. **Replace Router**: Convert Foxx router to Express router
3. **Update Middleware**: Convert Foxx middleware to Express middleware
4. **Remove Configuration**: Remove `manifest.json` and `services.json`
5. **Update Database Access**: Use Arangojs directly instead of Foxx context
6. **Add Express**: Add `express` and `arangojs` to dependencies
7. **Update Entrypoint**: Change from `node-foxx` to `node index.js`

See [SERVICE_COMPARISON.md](SERVICE_COMPARISON.md) for detailed differences.

