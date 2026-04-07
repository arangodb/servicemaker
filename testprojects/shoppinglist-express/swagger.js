const swaggerJsdoc = require('swagger-jsdoc');

const options = {
    definition: {
        openapi: '3.0.0',
        info: {
            title: 'Shopping List API',
            version: '1.0.0',
            description: 'A minimal shopping list service',
            contact: { name: 'API Support' },
        },
        servers: [
            {
                url: `http://localhost:${process.env.PORT || 3000}`,
                description: 'Local Development',
            },
        ],
        components: {
            schemas: {
                Item: {
                    type: 'object',
                    properties: {
                        _key: { type: 'string', example: '12345' },
                        _id: { type: 'string', example: 'shoppinglist/12345' },
                        _rev: { type: 'string', example: '_abc123' },
                        name: { type: 'string', example: 'Milk' },
                        quantity: { type: 'number', example: 2 },
                        completed: { type: 'boolean', example: false },
                    },
                },
                Error: {
                    type: 'object',
                    properties: {
                        error: { type: 'string' },
                        message: { type: 'string' },
                    },
                },
            },
        },
    },
    apis: ['./index.js'],
};

module.exports = swaggerJsdoc(options);
