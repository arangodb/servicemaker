# Node-Foxx Services vs Express + Arangojs Services

This document explains the key differences between Node-Foxx services (using `@arangodb/node-foxx` libraries) and simple Express.js applications using Arangojs driver.

## Overview

Both approaches can build REST APIs that interact with ArangoDB, but they use different architectures and have different trade-offs.

## Architecture Comparison

### Node-Foxx Services

**Components:**
- `@arangodb/node-foxx`: Framework providing router, context, and Foxx-specific features
- `@arangodb/node-foxx-launcher`: Orchestrator that manages services, worker threads, and HTTP server
- `@arangodb/arangodb`: Database client (used internally)

**Architecture:**
```
┌─────────────────────────────────────┐
│   node-foxx-launcher (Main Thread) │
│   - HTTP Server (Port 3000)        │
│   - Service Discovery               │
│   - Request Routing                 │
│   - Token Management                │
└──────────────┬──────────────────────┘
               │
       ┌───────┴────────┐
       │                │
┌──────▼──────┐  ┌──────▼──────┐
│ Worker 1    │  │ Worker 2     │
│ (Service A) │  │ (Service B)  │
│             │  │              │
│ - node-foxx │  │ - node-foxx  │
│ - Router    │  │ - Router     │
│ - Context   │  │ - Context    │
└─────────────┘  └──────────────┘
```

**Key Features:**
- Multi-service orchestration (multiple services on one port)
- Worker thread isolation (each service in separate thread)
- Automatic API documentation (Swagger)
- Service context and dependency injection
- Configuration management via `manifest.json`

### Express + Arangojs Services

**Components:**
- `express`: Web framework
- `arangojs`: Direct ArangoDB driver

**Architecture:**
```
┌─────────────────────────────┐
│   Express Application       │
│   - HTTP Server             │
│   - Routes                  │
│   - Middleware              │
│   - Direct Arangojs Access  │
└─────────────────────────────┘
```

**Key Features:**
- Standalone application (one service per process)
- Standard Express.js patterns
- Direct database access
- Full control over application structure
- Manual API documentation (if needed)

## Detailed Comparison

### 1. Service Configuration

**Node-Foxx:**
```json
// manifest.json
{
  "name": "my-service",
  "version": "1.0.0",
  "main": "index.js",
  "engines": { "arangodb": "^3.0" }
}

// services.json
[
  {
    "mount": "/myservice",
    "basePath": "."
  }
]
```

**Express:**
```json
// package.json
{
  "name": "my-service",
  "version": "1.0.0",
  "main": "index.js",
  "dependencies": {
    "express": "^4.18.2",
    "arangojs": "^10.1.2"
  }
}
```

**Difference:** Express requires only `package.json`, no additional configuration files.

### 2. Routing

**Node-Foxx:**
```javascript
const createRouter = require('@arangodb/node-foxx/router');
const { context } = require('@arangodb/node-foxx/locals');

const router = createRouter();
context.use(router);

router.get('/items', async (req, res) => {
  const collection = context.collection('items');
  const items = await collection.all();
  res.send(items);
})
  .response('array', joi.array().items(joi.object()), 'List of items')
  .summary('Get all items');
```

**Express:**
```javascript
const express = require('express');
const { Database } = require('arangojs');

const app = express();
const db = new Database({ url: '...', auth: { token: '...' } });

app.get('/items', async (req, res) => {
  const collection = db.collection('items');
  const cursor = await collection.all();
  const items = await cursor.all();
  res.json(items);
});
```

**Difference:** 
- Foxx: Uses Foxx router with built-in Swagger generation
- Express: Uses standard Express router, Swagger must be added manually

### 3. Database Access

**Node-Foxx:**
```javascript
const { context } = require('@arangodb/node-foxx/locals');

// Access via context (database connection managed by launcher)
const collection = context.collection('items');
const db = context.database('mydb');
```

**Express:**
```javascript
const { Database } = require('arangojs');

// Direct database connection (must manage yourself)
const db = new Database({
  url: process.env.ARANGO_DB_ENDPOINT,
  auth: { token: process.env.ARANGODB_JWT_TOKEN },
  databaseName: process.env.ARANGO_DB_NAME
});

const collection = db.collection('items');
```

**Difference:**
- Foxx: Database connection managed by launcher, accessed via context
- Express: Must create and manage database connections manually

### 4. Multi-Service Support

**Node-Foxx:**
```json
// services.json - Multiple services on one port
[
  { "mount": "/service1", "basePath": "./service1" },
  { "mount": "/service2", "basePath": "./service2" }
]
```
- All services run on same port (e.g., 3000)
- Each service in separate worker thread
- Launcher routes requests to correct service

**Express:**
- One service per application/process
- Each service needs its own port or reverse proxy
- No built-in multi-service orchestration

**Difference:** Foxx supports multiple services on one port; Express requires separate processes/ports.

### 5. API Documentation

**Node-Foxx:**
```javascript
router.get('/items/:id', handler)
  .pathParam('id', joi.string().required(), 'Item ID')
  .response(200, joi.object(), 'Item details')
  .summary('Get item by ID');
```
- Automatic Swagger generation from route definitions
- Built into router methods (`.body()`, `.response()`, `.pathParam()`)

**Express:**
```javascript
/**
 * @swagger
 * /items/{id}:
 *   get:
 *     summary: Get item by ID
 *     parameters:
 *       - in: path
 *         name: id
 *         required: true
 *         schema:
 *           type: string
 */
app.get('/items/:id', handler);
```
- Manual Swagger documentation using JSDoc comments
- Requires `swagger-jsdoc` and `swagger-ui-express` packages
- More verbose but more flexible

**Difference:** Foxx auto-generates docs; Express requires manual documentation.

### 6. Service Context

**Node-Foxx:**
```javascript
const { context } = require('@arangodb/node-foxx/locals');

// Access service metadata
context.mount          // "/myservice"
context.baseUrl        // "http://localhost:3000/myservice"
context.manifest       // manifest.json content
context.configuration  // Service configuration
context.dependencies   // Service dependencies
```

**Express:**
```javascript
// No built-in context
// Must manage configuration yourself
const config = require('./config');
```

**Difference:** Foxx provides rich service context; Express requires manual configuration management.

### 7. Setup/Teardown Scripts

**Node-Foxx:**
```json
// manifest.json
{
  "scripts": {
    "setup": "scripts/setup.js",
    "teardown": "scripts/teardown.js"
  }
}
```
- Automatically executed by launcher during service installation/removal
- Access to service context and database

**Express:**
```json
// package.json
{
  "scripts": {
    "setup": "node scripts/setup.js"
  }
}
```
- Must be run manually
- Must manage database connection yourself

**Difference:** Foxx scripts run automatically; Express scripts are manual.

### 8. Error Handling

**Node-Foxx:**
```javascript
router.get('/items/:id', async (req, res) => {
  try {
    const item = await collection.document(id);
    res.send(item);
  } catch (e) {
    if (e.isArangoError && e.errorNum === 1202) {
      res.throw(404, 'Item not found');
    }
    throw e;
  }
});
```
- Built-in error handling with `res.throw()`
- Automatic error response formatting

**Express:**
```javascript
app.get('/items/:id', async (req, res, next) => {
  try {
    const item = await collection.document(id);
    res.json(item);
  } catch (e) {
    if (e.isArangoError && e.errorNum === 1202) {
      return res.status(404).json({ error: 'Item not found' });
    }
    next(e);
  }
});

// Error handling middleware
app.use((err, req, res, next) => {
  res.status(err.status || 500).json({ error: err.message });
});
```
- Standard Express error handling patterns
- Requires manual error middleware

**Difference:** Foxx has built-in error handling; Express uses standard middleware patterns.

### 9. Request/Response Objects

**Node-Foxx:**
```javascript
// Synthetic request/response objects
router.get('/items', async (req, res) => {
  req.body          // Parsed request body
  req.pathParams    // Path parameters
  req.queryParams   // Query parameters
  req.headers       // Request headers
  
  res.send(data)    // Send response (auto-formats)
  res.json(data)    // Send JSON
  res.throw(404)    // Throw error
});
```

**Express:**
```javascript
// Standard Express request/response
app.get('/items', async (req, res) => {
  req.body          // Parsed request body
  req.params        // Path parameters
  req.query         // Query parameters
  req.headers       // Request headers
  
  res.json(data)    // Send JSON
  res.status(404)   // Set status
  res.send(data)    // Send response
});
```

**Difference:** Similar APIs, but Foxx uses synthetic objects; Express uses standard Express objects.

### 10. Validation

**Node-Foxx:**
```javascript
router.post('/items', handler)
  .body(joi.object({
    name: joi.string().required(),
    quantity: joi.number().optional()
  }), 'Item to create')
  .response(201, joi.object(), 'Created item');
```
- Built-in Joi validation
- Automatic Swagger schema generation

**Express:**
```javascript
const itemSchema = Joi.object({
  name: Joi.string().required(),
  quantity: Joi.number().optional()
});

app.post('/items', async (req, res) => {
  const { error, value } = itemSchema.validate(req.body);
  if (error) {
    return res.status(400).json({ error: error.details[0].message });
  }
  // ... handle request
});
```
- Manual validation
- Must handle validation errors yourself

**Difference:** Foxx has built-in validation; Express requires manual validation.

## When to Use Each

### Use Node-Foxx When:

✅ **Multi-service architecture**: Need multiple services on one port  
✅ **Automatic API docs**: Want auto-generated Swagger documentation  
✅ **Service isolation**: Need worker thread isolation  
✅ **ArangoDB-native**: Building services specifically for ArangoDB  
✅ **Configuration management**: Need built-in configuration system  
✅ **Migration from ArangoDB Foxx**: Migrating existing Foxx services  

### Use Express + Arangojs When:

✅ **Standard Express patterns**: Want familiar Express.js structure  
✅ **Flexibility**: Need full control over application architecture  
✅ **Standalone service**: Single service per application  
✅ **Existing Express apps**: Migrating or extending Express applications  
✅ **Simpler setup**: Don't need Foxx-specific features  
✅ **Standard tooling**: Want to use standard Node.js/Express tooling  

## Migration Path

### From Foxx to Express:

1. **Remove Foxx dependencies**:
   ```bash
   npm uninstall @arangodb/node-foxx @arangodb/node-foxx-launcher
   ```

2. **Add Express dependencies**:
   ```bash
   npm install express arangojs
   ```

3. **Replace router**:
   ```javascript
   // Before (Foxx)
   const router = createRouter();
   context.use(router);
   
   // After (Express)
   const app = express();
   ```

4. **Update database access**:
   ```javascript
   // Before (Foxx)
   const collection = context.collection('items');
   
   // After (Express)
   const db = new Database({ url: '...', auth: { token: '...' } });
   const collection = db.collection('items');
   ```

5. **Remove configuration files**:
   - Delete `manifest.json`
   - Delete `services.json` (if single service)

6. **Update entrypoint**:
   ```json
   // package.json
   {
     "main": "index.js",
     "scripts": {
       "start": "node index.js"
     }
   }
   ```

### From Express to Foxx:

1. **Add Foxx dependencies**:
   ```bash
   npm install @arangodb/node-foxx @arangodb/node-foxx-launcher @arangodb/arangodb
   ```

2. **Create manifest.json**:
   ```json
   {
     "name": "my-service",
     "version": "1.0.0",
     "main": "index.js"
   }
   ```

3. **Create services.json**:
   ```json
   [
     {
       "mount": "/myservice",
       "basePath": "."
     }
   ]
   ```

4. **Replace Express router with Foxx router**:
   ```javascript
   // Before (Express)
   const app = express();
   app.get('/items', handler);
   
   // After (Foxx)
   const router = createRouter();
   context.use(router);
   router.get('/items', handler);
   ```

5. **Update database access**:
   ```javascript
   // Before (Express)
   const db = new Database({ ... });
   
   // After (Foxx)
   const collection = context.collection('items');
   ```

## Summary

| Aspect | Node-Foxx | Express + Arangojs |
|--------|-----------|-------------------|
| **Complexity** | Higher (orchestration, worker threads) | Lower (standard Express) |
| **Multi-service** | ✅ Built-in | ❌ Requires separate processes |
| **API Docs** | ✅ Auto-generated | ⚠️ Manual (Swagger) |
| **Configuration** | ✅ Built-in (manifest.json) | ⚠️ Manual |
| **Flexibility** | ⚠️ Foxx-specific patterns | ✅ Full control |
| **Setup Scripts** | ✅ Automatic | ⚠️ Manual |
| **Dependencies** | More (Foxx libraries) | Fewer (Express + Arangojs) |
| **Learning Curve** | Steeper (Foxx concepts) | Gentler (standard Express) |
| **Use Case** | ArangoDB-native services | Standard web services |

Both approaches are valid and serve different needs. Choose based on your requirements for multi-service support, API documentation, and architectural preferences.

