# ArangoDB Service (Node.js)

A small Node.js Express service for basic ArangoDB operations. Node.js port of the Python `arango-test-service` in this repository.

We publish this as the Docker image `arangodb/test-service-nodejs` on Docker hub.
It is regularly scanned for security vulnerabilities.

## Requirements

- Node.js 22+
- ArangoDB instance with JWT authentication

## Installation

```bash
npm install
```

## Configuration

Set the following environment variable:

```bash
export ARANGO_DEPLOYMENT_ENDPOINT="http://localhost:8529"
```

## Running the Service

```bash
node main.js
```

Or with npm:

```bash
npm start
```

## API Documentation

Interactive Swagger UI (same path as the Python FastAPI service):

```bash
http://localhost:8000/docs
```

## API Endpoints

All endpoints require a JWT token in the `Authorization` header:

```
Authorization: Bearer <your-jwt-token>
```

### POST /write

Creates the `test-service` database and in there the `test` collection
(if these do not yet exist) and writes the request body as a document.

**Example:**
```bash
curl -X POST http://localhost:8000/write \
  -H "Authorization: Bearer YOUR_JWT_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "example", "value": 42}'
```

**Success Response (200):**
```json
{
  "status": "success",
  "message": "Document written successfully",
  "document_key": "_key_value"
}
```

### GET /read

Retrieves all documents from the `test` collection in the `test-service` 
database.

**Example:**
```bash
curl -X GET http://localhost:8000/read \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

**Success Response (200):**
```json
{
  "status": "success",
  "message": "Retrieved 1 document(s)",
  "documents": [
    {
      "_key": "12345",
      "_id": "test/12345",
      "_rev": "_abc123",
      "name": "example",
      "value": 42
    }
  ]
}
```

### DELETE /delete

Deletes the entire `test-service` database.

**Example:**
```bash
curl -X DELETE http://localhost:8000/delete \
  -H "Authorization: Bearer YOUR_JWT_TOKEN"
```

**Success Response (200):**
```json
{
  "status": "success",
  "message": "Database 'test-service' deleted successfully"
}
```

## Logging

Console output uses a uvicorn-style format (startup, per-request access lines, graceful shutdown). Example:

```
INFO:     Service running on http://0.0.0.0:8000 (Press CTRL+C to quit)
INFO:     172.17.0.1 - "GET /health HTTP/1.1" 200 OK
```

## Error Handling

All endpoints return HTTP 2xx status codes on success. Error responses include:

- **400**: Insert returned no result (POST /write only)
- **401**: Missing Authorization header
- **404**: Database does not exist (DELETE /delete only)
- **500**: Database errors or other server errors

**Error Response Format:**
```json
{
  "detail": "Descriptive error message"
}
```

## Health Check

```bash
curl http://localhost:8000/health
```

Returns:
```json
{
  "status": "healthy"
}
```

## Docker Build

Build with servicemaker (uses `arangodb/node22base` and installs only missing dependencies):

```bash
servicemaker --project-home arango-test-service-nodejs --port 8000 --entrypoint main.js --image-name arangodb/test-service-nodejs
```
