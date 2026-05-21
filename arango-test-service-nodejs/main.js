#!/usr/bin/env node
/**
 * Small Express service for ArangoDB operations.
 * Provides endpoints to write, read, and delete data from ArangoDB.
 */

const express = require('express');
const { Database } = require('arangojs');
const { Agent, setGlobalDispatcher } = require('undici');
const swaggerUi = require('swagger-ui-express');
const swaggerSpec = require('./swagger');
const {
  logError,
  accessLogMiddleware,
  logServerStartup,
  registerShutdownHandlers,
} = require('./logger');

const app = express();
const PORT = process.env.PORT || 8000;

app.set('trust proxy', true);
app.set('etag', false);

// Disable SSL certificate verification for HTTPS endpoints (e.g., self-signed certificates)
setGlobalDispatcher(
  new Agent({
    connect: {
      rejectUnauthorized: false,
    },
  })
);

function startupCheck() {
  const endpoint = process.env.ARANGO_DEPLOYMENT_ENDPOINT;
  if (!endpoint) {
    logError('ARANGO_DEPLOYMENT_ENDPOINT environment variable not set');
    process.exit(1);
  }
}

function getArangoUrl() {
  const endpoint = process.env.ARANGO_DEPLOYMENT_ENDPOINT;
  if (!endpoint) {
    throw new Error('ARANGO_DEPLOYMENT_ENDPOINT environment variable not set');
  }
  return endpoint;
}

function createDatabaseConnection(databaseName, token) {
  const db = new Database({
    url: getArangoUrl(),
    databaseName,
  });
  db.useBearerAuth(token);
  return db;
}

async function getSystemDatabase(token) {
  try {
    return createDatabaseConnection('_system', token);
  } catch (e) {
    logError(`Caught exception: ${e}`);
    process.exit(1);
  }
}

async function getDatabase(token, dbName = 'test-service') {
  const sysDb = await getSystemDatabase(token);
  const databases = await sysDb.listDatabases();
  if (!databases.includes(dbName)) {
    await sysDb.createDatabase(dbName);
  }

  try {
    return createDatabaseConnection(dbName, token);
  } catch (e) {
    logError(`Caught exception: ${e}`);
    process.exit(1);
  }
}

async function getExistingDatabase(token, dbName = 'test-service') {
  const sysDb = await getSystemDatabase(token);
  const databases = await sysDb.listDatabases();
  if (!databases.includes(dbName)) {
    throw new Error(`Database '${dbName}' does not exist`);
  }

  try {
    return createDatabaseConnection(dbName, token);
  } catch (e) {
    logError(`Caught exception: ${e}`);
    process.exit(1);
  }
}

function extractToken(authorization) {
  if (!authorization) {
    const error = new Error('Authorization header required');
    error.status = 401;
    throw error;
  }
  return authorization.replace('bearer ', '').replace('Bearer ', '').trim();
}

function isArangoError(err) {
  return err && (err.isArangoError || err.name === 'ArangoError');
}

function arangoErrorMessage(err) {
  return err.message || String(err);
}

/** Map errors to HTTP responses (aligned with Python FastAPI handlers). */
function handleRouteError(res, err) {
  if (err && err.status === 401) {
    logError(err.message);
    return res.status(401).json({ detail: err.message });
  }
  if (err instanceof Error && err.message.startsWith('ARANGO_DEPLOYMENT_ENDPOINT')) {
    return res.status(500).json({ detail: err.message });
  }
  if (err instanceof Error && err.message.includes('does not exist')) {
    return res.status(500).json({ detail: err.message });
  }
  if (isArangoError(err)) {
    return res.status(500).json({ detail: `ArangoDB error: ${arangoErrorMessage(err)}` });
  }
  return res.status(500).json({ detail: `Unexpected error: ${arangoErrorMessage(err)}` });
}

app.use(accessLogMiddleware());
app.use(express.json());

app.use('/docs', swaggerUi.serve, swaggerUi.setup(swaggerSpec, {
  customCss: '.swagger-ui .topbar { display: none }',
  customSiteTitle: 'ArangoDB Service',
}));

app.post('/write', async (req, res) => {
  try {
    const authorization = req.get('authorization');
    const token = extractToken(authorization);
    const db = await getDatabase(token, 'test-service');
    const collection = db.collection('test');

    if (!(await collection.exists())) {
      await collection.create();
    }

    const result = await collection.save(req.body);
    if (result) {
      return res.json({
        status: 'success',
        message: 'Document written successfully',
        document_key: result._key,
      });
    }
    return res.status(400).json({ detail: 'ArangoDB error: got None result' });
  } catch (err) {
    return handleRouteError(res, err);
  }
});

app.get('/read', async (req, res) => {
  try {
    const authorization = req.get('authorization');
    const token = extractToken(authorization);
    const db = await getExistingDatabase(token, 'test-service');
    const collection = db.collection('test');

    if (!(await collection.exists())) {
      return res.json({
        status: 'success',
        message: "Collection 'test' does not exist",
        documents: [],
      });
    }

    const cursor = await db.query('FOR doc IN @@collection RETURN doc', {
      '@collection': 'test',
    });
    const documents = await cursor.all();

    return res.json({
      status: 'success',
      message: `Retrieved ${documents.length} document(s)`,
      documents,
    });
  } catch (err) {
    return handleRouteError(res, err);
  }
});

app.delete('/delete', async (req, res) => {
  try {
    const authorization = req.get('authorization');
    const token = extractToken(authorization);
    const sysDb = await getSystemDatabase(token);
    const databases = await sysDb.listDatabases();

    if (!databases.includes('test-service')) {
      return res.status(404).json({
        detail: "Database 'test-service' does not exist, nothing to delete",
      });
    }

    await sysDb.dropDatabase('test-service');

    return res.json({
      status: 'success',
      message: "Database 'test-service' deleted successfully",
    });
  } catch (err) {
    return handleRouteError(res, err);
  }
});

app.get('/health', (req, res) => {
  res.json({ status: 'healthy' });
});

if (require.main === module) {
  startupCheck();
  const server = app.listen(PORT, '0.0.0.0', () => {
    logServerStartup(PORT);
  });
  registerShutdownHandlers(server);
}

module.exports = app;
