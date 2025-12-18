require('dotenv').config();

const express = require('express');
const { Database } = require('arangojs');
const swaggerUi = require('swagger-ui-express');
const swaggerSpec = require('./swagger');
const itemSchema = require('./schemas');

const app = express();
const PORT = process.env.PORT || 3000;
const DOCUMENT_NOT_FOUND = 1202;
const collectionName = 'shoppinglist';

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

// Database middleware (simulated - connect to ArangoDB)
const dbMiddleware = (req, res, next) => {
    const authHeader = req.headers.authorization || req.headers.Authorization;
    const dbEndpoint = process.env.ARANGO_DB_ENDPOINT || 'http://127.0.0.1:8529';
    const dbName = process.env.ARANGO_DB_NAME || '_system';
    let token = null;

    if (authHeader) {
        const parts = authHeader.split(' ');
        token = parts.length === 2 && parts[0] === 'Bearer' ? parts[1] : authHeader;
    }

    if (!token) {
        return res.status(401).json({ error: 'Authorization header with JWT token is required' });
    }

    try {
        const db = new Database({
            url: dbEndpoint,
            auth: { token },
            databaseName: dbName
        });

        req.db = db;
        next();
    } catch (error) {
        console.error('Database connection error:', error.message);
        return res.status(500).json({ error: 'Database connection error', message: error.message });
    }
};

app.use('/api', dbMiddleware);

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

        const collection = req.db.collection(collectionName);
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
        const cursor = await req.db.query('FOR item IN @@collection RETURN item', {
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

        const collection = req.db.collection(collectionName);
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

        const collection = req.db.collection(collectionName);
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
