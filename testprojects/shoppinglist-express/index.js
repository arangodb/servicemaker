require('dotenv').config();

const express = require('express');
const { Database } = require('arangojs');
const swaggerUi = require('swagger-ui-express');
const swaggerSpec = require('./swagger');
const itemSchema = require('./schemas');
const fs = require('fs');

const app = express();
const PORT = process.env.PORT || 3000;
const DOCUMENT_NOT_FOUND = 1202;
const collectionName = 'shoppinglist';

// Global database connection
let db = null;
let systemDb = null; // For listing databases

// Helper function to get token from various sources
// Based on Python ServiceTokenManager pattern:
// 1. ARANGO_TOKEN env var contains a FILE PATH (not the token itself)
// 2. The actual JWT token is stored in that file
// 3. File is auto-rotated by operator, so we re-read on each call
// Throws error if token is not available
function getToken(providedToken) {
    // Priority: 1. Provided token (from request body/query), 2. ARANGO_TOKEN file path
    
    // 1. Use explicitly provided token (from request body/query)
    if (providedToken) {
        return providedToken;
    }
    
    // 2. Read token from ARANGO_TOKEN file path (mounted by operator)
    // ARANGO_TOKEN env var contains the file path, not the token itself
    // Example: ARANGO_TOKEN=/var/run/secrets/arango/token/token
    const tokenPath = process.env.ARANGO_TOKEN;
    if (tokenPath) {
        // Check if it's a file path (exists as a file)
        if (fs.existsSync(tokenPath)) {
            try {
                // Re-read file on each call to pick up operator-rotated tokens
                const token = fs.readFileSync(tokenPath, 'utf8').trim();
                if (token) {
                    return token;
                } else {
                    throw new Error(`ARANGO_TOKEN file is empty: ${tokenPath}`);
                }
            } catch (err) {
                if (err.code === 'ENOENT') {
                    throw new Error(`ARANGO_TOKEN file not found: ${tokenPath}. Sidecar may still be initializing.`);
                } else if (err.code === 'EACCES') {
                    throw new Error(`Permission denied reading ARANGO_TOKEN file: ${tokenPath}`);
                } else if (err.message && err.message.includes('ARANGO_TOKEN file is empty')) {
                    throw err;
                } else {
                    throw new Error(`Error reading ARANGO_TOKEN file ${tokenPath}: ${err.message}`);
                }
            }
        } else {
            // ARANGO_TOKEN is set but file doesn't exist yet (sidecar may still be starting up)
            throw new Error(`ARANGO_TOKEN=${tokenPath} but file does not exist yet. Sidecar may still be initializing.`);
        }
    }
    
    // No token available
    throw new Error('No service token available. ARANGO_TOKEN env var is not set. Ensure ArangoPermissionToken CR is configured in the Helm chart and the permissions.arangodb.com/token label is on the pod.');
}

// Middleware
app.use(express.json());

// Swagger UI
app.use('/docs', swaggerUi.serve, swaggerUi.setup(swaggerSpec, {
    customCss: '.swagger-ui .topbar { display: none }',
    customSiteTitle: 'Shopping List API',
}));

// Health check
app.get('/health', (req, res) => {
    res.json({ status: 'OK' });
});

// Middleware to check if database is initialized
const checkDbInitialized = (req, res, next) => {
    if (!db) {
        return res.status(400).json({ 
            error: 'Database not initialized', 
            message: 'Please call /api/init first to initialize the database connection' 
        });
    }
    next();
};

/**
 * @swagger
 * /api/databases:
 *   get:
 *     summary: Get list of databases
 *     tags: [Database]
 *     parameters:
 *       - in: query
 *         name: url
 *         schema:
 *           type: string
 *         description: ArangoDB URL (optional, defaults to env or http://127.0.0.1:8529)
 *       - in: query
 *         name: token
 *         schema:
 *           type: string
 *         description: JWT token for authentication (optional, can also use ARANGO_TOKEN env var)
 *     responses:
 *       200:
 *         description: List of databases
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 databases:
 *                   type: array
 *                   items:
 *                     type: string
 *       500:
 *         description: Error connecting to ArangoDB
 */
app.get('/api/databases', async (req, res, next) => {
    try {
        const url = req.query.url || process.env.ARANGODB_ENDPOINT || 'http://127.0.0.1:8529';
        let token;
        try {
            token = getToken(req.query.token);
        } catch (tokenError) {
            return res.status(401).json({ 
                error: 'Authentication required', 
                message: tokenError.message 
            });
        }
        console.log('dbConfig', {
            url: url,
            bearerAuth: token,
            databaseName: '_system'
        });
        // Connect to _system database to list databases
        const systemDbConn = new Database({
            url,
            databaseName: '_system',
            agentOptions: {
                rejectUnauthorized: false,
            },
        });
        systemDbConn.useBearerAuth(token);

        const databases = await systemDbConn.listDatabases();
        res.json({ databases });
    } catch (err) {
        console.error('Error listing databases:', err.message);
        return res.status(500).json({ 
            error: 'Failed to list databases', 
            message: err.message 
        });
    }
});

/**
 * @swagger
 * /api/init:
 *   post:
 *     summary: Initialize database connection
 *     tags: [Database]
 *     requestBody:
 *       required: true
 *       content:
 *         application/json:
 *           schema:
 *             type: object
 *             required:
 *               - databaseName
 *             properties:
 *               url:
 *                 type: string
 *                 description: ArangoDB URL (optional)
 *                 example: http://127.0.0.1:8529
 *               token:
 *                 type: string
 *                 description: JWT token for authentication (optional)
 *               databaseName:
 *                 type: string
 *                 description: Name of the database to create/use (required)
 *                 example: shoppinglist_db
 *     responses:
 *       200:
 *         description: Database initialized successfully
 *         content:
 *           application/json:
 *             schema:
 *               type: object
 *               properties:
 *                 message:
 *                   type: string
 *                 databaseName:
 *                   type: string
 *                 url:
 *                   type: string
 *       400:
 *         description: Validation error
 *       500:
 *         description: Database connection or creation error
 */
app.post('/api/init', async (req, res, next) => {
    try {
        const { url, token, databaseName } = req.body;

        if (!databaseName || !databaseName.trim()) {
            return res.status(400).json({ error: 'databaseName is required' });
        }

        const dbUrl = url || process.env.ARANGODB_ENDPOINT || 'http://127.0.0.1:8529';
        let dbToken;
        try {
            dbToken = getToken(token);
        } catch (tokenError) {
            return res.status(401).json({ 
                error: 'Authentication required', 
                message: tokenError.message 
            });
        }

        // Connect to _system database first to create the database
        systemDb = new Database({
            url: dbUrl,
            databaseName: '_system',
            agentOptions: {
                rejectUnauthorized: false,
            },
        });
        systemDb.useBearerAuth(dbToken);

        // Check if database exists, create if it doesn't
        const databases = await systemDb.listDatabases();
        if (!databases.includes(databaseName)) {
            await systemDb.createDatabase(databaseName);
            console.log(`Database '${databaseName}' created successfully`);
        } else {
            console.log(`Database '${databaseName}' already exists`);
        }

        console.log('dbConfig', {
            url: dbUrl,
            bearerAuth: dbToken,
            databaseName: databaseName
        });

        // Now connect to the created database
        db =  new Database({
            url: dbUrl,
            databaseName: databaseName,
            agentOptions: {
                rejectUnauthorized: false,
            },
        });
        db.useBearerAuth(dbToken);

        // Verify connection by checking if we can access the database
        await db.version();

        // Ensure the collection exists
        const collection = db.collection(collectionName);
        if (!(await collection.exists())) {
            await collection.create();
            console.log(`Collection '${collectionName}' created successfully`);
        }

        res.json({
            message: 'Database initialized successfully',
            databaseName: databaseName,
            url: dbUrl
        });
    } catch (err) {
        console.error('Database initialization error:', err.message);
        return res.status(500).json({ 
            error: 'Database initialization failed', 
            message: err.message 
        });
    }
});

// Apply database check middleware to all /api routes except /api/init and /api/databases
app.use('/api', (req, res, next) => {
    if (req.path === '/init' || req.path === '/databases') {
        return next();
    }
    checkDbInitialized(req, res, next);
});

/**
 * @swagger
 * /api/items:
 *   post:
 *     summary: Create a new shopping list item
 *     tags: [Items]
 *     requestBody:
 *       required: true
 *       content:
 *         application/json:
 *           schema:
 *             $ref: '#/components/schemas/Item'
 *     responses:
 *       201:
 *         description: Item created
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Item'
 *       400:
 *         description: Validation error
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Error'
 */
app.post('/api/items', async (req, res, next) => {
    try {
        const { error, value } = itemSchema.validate(req.body);
        if (error) {
            return res.status(400).json({ error: error.details[0].message });
        }

        const collection = db.collection(collectionName);
        const meta = await collection.save(value);
        const savedItem = await collection.document(meta._key);
        res.status(201).json(savedItem);
    } catch (err) {
        next(err);
    }
});

/**
 * @swagger
 * /api/items:
 *   get:
 *     summary: Get all items
 *     tags: [Items]
 *     responses:
 *       200:
 *         description: List of all items
 *         content:
 *           application/json:
 *             schema:
 *               type: array
 *               items:
 *                 $ref: '#/components/schemas/Item'
 */
app.get('/api/items', async (req, res, next) => {
    try {
        const cursor = await db.query('FOR item IN @@collection RETURN item', {
            '@collection': collectionName,
        });
        const items = await cursor.all();
        res.json(items);
    } catch (err) {
        next(err);
    }
});

/**
 * @swagger
 * /api/items/{key}:
 *   get:
 *     summary: Get item by key
 *     tags: [Items]
 *     parameters:
 *       - in: path
 *         name: key
 *         required: true
 *         schema:
 *           type: string
 *     responses:
 *       200:
 *         description: Item found
 *         content:
 *           application/json:
 *             schema:
 *               $ref: '#/components/schemas/Item'
 *       404:
 *         description: Item not found
 */
app.get('/api/items/:key', async (req, res, next) => {
    try {
        const { key } = req.params;
        if (!key?.trim()) {
            return res.status(400).json({ error: 'Invalid key' });
        }

        const collection = db.collection(collectionName);
        const item = await collection.document(key);
        res.json(item);
    } catch (err) {
        if (err.isArangoError && err.errorNum === DOCUMENT_NOT_FOUND) {
            return res.status(404).json({ error: 'Item not found' });
        }
        next(err);
    }
});

/**
 * @swagger
 * /api/items/{key}:
 *   delete:
 *     summary: Delete item by key
 *     tags: [Items]
 *     parameters:
 *       - in: path
 *         name: key
 *         required: true
 *         schema:
 *           type: string
 *     responses:
 *       204:
 *         description: Item deleted
 *       404:
 *         description: Item not found
 */
app.delete('/api/items/:key', async (req, res, next) => {
    try {
        const { key } = req.params;
        if (!key?.trim()) {
            return res.status(400).json({ error: 'Invalid key' });
        }

        const collection = db.collection(collectionName);
        await collection.remove(key);
        res.status(204).send();
    } catch (err) {
        if (err.isArangoError && err.errorNum === DOCUMENT_NOT_FOUND) {
            return res.status(404).json({ error: 'Item not found' });
        }
        next(err);
    }
});

// Error handling
app.use((err, req, res, next) => {
    console.error('Error:', process.env.NODE_ENV === 'development' ? err : err.message);
    res.status(err.status || 500).json({
        error: err.message || 'Internal server error',
    });
});

// Start server
const server = app.listen(PORT, () => {
    console.log(`Server running on port ${PORT}`);
});

// Graceful shutdown
process.on('SIGTERM', () => {
    console.log('SIGTERM received, shutting down gracefully...');
    server.close(() => process.exit(0));
});

module.exports = app;
