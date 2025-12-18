# Shopping List Express - Mini Version

A minimal, straightforward shopping list API built with Express.js and ArangoDB.

## Project Structure

```
shoppinglist-express/
├── index.js         # Main app with all routes
├── schemas.js       # Joi validation schemas
├── swagger.js       # Swagger configuration
├── package.json     # Dependencies
├── .env             # Environment variables
└── README.md        # This file
```

## Setup

1. Install dependencies:
```bash
npm install
```

2. Update `.env` with your configuration:
```
PORT=3000
NODE_ENV=development
```

3. Connect to ArangoDB (in `index.js`, attach your db connection to `req.db`)

4. Start the server:
```bash
npm start
```

## API Endpoints

- `GET /health` - Health check
- `GET /docs` - Swagger documentation
- `POST /api/items` - Create item
- `GET /api/items` - Get all items
- `GET /api/items/:key` - Get item by key
- `DELETE /api/items/:key` - Delete item

## Features

✅ Simple and straightforward structure  
✅ Minimal folder organization  
✅ Swagger documentation included  
✅ Input validation with Joi  
✅ Error handling middleware  
✅ Graceful shutdown  
