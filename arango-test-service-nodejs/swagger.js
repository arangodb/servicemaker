/**
 * OpenAPI specification for ArangoDB test service (served at /docs).
 */

const port = process.env.PORT || 8000;

module.exports = {
  openapi: '3.0.0',
  info: {
    title: 'ArangoDB Service',
    version: '1.0.0',
    description:
      'Small service for ArangoDB operations. All endpoints except /health require a JWT in the Authorization header.',
  },
  servers: [
    {
      url: `http://localhost:${port}`,
      description: 'Local development',
    },
  ],
  components: {
    securitySchemes: {
      bearerAuth: {
        type: 'http',
        scheme: 'bearer',
        bearerFormat: 'JWT',
        description: 'JWT token (e.g. `Bearer <your-jwt-token>`)',
      },
    },
    schemas: {
      Error: {
        type: 'object',
        properties: {
          detail: { type: 'string', example: 'Authorization header required' },
        },
      },
      WriteSuccess: {
        type: 'object',
        properties: {
          status: { type: 'string', example: 'success' },
          message: { type: 'string', example: 'Document written successfully' },
          document_key: { type: 'string', example: '12345' },
        },
      },
      ReadSuccess: {
        type: 'object',
        properties: {
          status: { type: 'string', example: 'success' },
          message: { type: 'string', example: 'Retrieved 1 document(s)' },
          documents: {
            type: 'array',
            items: { type: 'object', additionalProperties: true },
          },
        },
      },
      DeleteSuccess: {
        type: 'object',
        properties: {
          status: { type: 'string', example: 'success' },
          message: {
            type: 'string',
            example: "Database 'test-service' deleted successfully",
          },
        },
      },
      Health: {
        type: 'object',
        properties: {
          status: { type: 'string', example: 'healthy' },
        },
      },
    },
  },
  paths: {
    '/health': {
      get: {
        summary: 'Health check',
        tags: ['Health'],
        responses: {
          200: {
            description: 'Service is healthy',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Health' },
              },
            },
          },
        },
      },
    },
    '/write': {
      post: {
        summary: 'Write a document',
        description:
          "Creates the `test-service` database and `test` collection if they do not exist, then inserts the request body as a document.",
        tags: ['ArangoDB'],
        security: [{ bearerAuth: [] }],
        requestBody: {
          required: true,
          content: {
            'application/json': {
              schema: {
                type: 'object',
                additionalProperties: true,
                example: { name: 'example', value: 42 },
              },
            },
          },
        },
        responses: {
          200: {
            description: 'Document written',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/WriteSuccess' },
              },
            },
          },
          400: {
            description: 'Insert returned no result',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Error' },
              },
            },
          },
          401: {
            description: 'Missing Authorization header',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Error' },
              },
            },
          },
          500: {
            description: 'Server or ArangoDB error',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Error' },
              },
            },
          },
        },
      },
    },
    '/read': {
      get: {
        summary: 'Read all documents',
        description:
          "Returns all documents from the `test` collection in the `test-service` database.",
        tags: ['ArangoDB'],
        security: [{ bearerAuth: [] }],
        responses: {
          200: {
            description: 'Documents retrieved',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/ReadSuccess' },
              },
            },
          },
          401: {
            description: 'Missing Authorization header',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Error' },
              },
            },
          },
          500: {
            description: 'Server or ArangoDB error',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Error' },
              },
            },
          },
        },
      },
    },
    '/delete': {
      delete: {
        summary: 'Delete test database',
        description: "Drops the entire `test-service` database.",
        tags: ['ArangoDB'],
        security: [{ bearerAuth: [] }],
        responses: {
          200: {
            description: 'Database deleted',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/DeleteSuccess' },
              },
            },
          },
          401: {
            description: 'Missing Authorization header',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Error' },
              },
            },
          },
          404: {
            description: 'Database does not exist',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Error' },
              },
            },
          },
          500: {
            description: 'Server or ArangoDB error',
            content: {
              'application/json': {
                schema: { $ref: '#/components/schemas/Error' },
              },
            },
          },
        },
      },
    },
  },
};
