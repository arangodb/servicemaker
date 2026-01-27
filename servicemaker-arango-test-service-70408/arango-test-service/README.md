# ArangoDB Service

A small Python FastAPI service for basic ArangoDB operations.

We publish this as the Docker image `arangodb/test-service` on Docker hub.
It is regularly scanned for security vulnerabilities.

## Requirements

- Python 3.13+
- ArangoDB instance with JWT authentication

## Installation

```bash
pip install -e .
```

## Configuration

Set the following environment variable:

```bash
export ARANGO_DEPLOYMENT_ENDPOINT="http://localhost:8529"
```

## Running the Service

```bash
python main.py
```

Or with uvicorn directly:

```bash
uvicorn main:app --host 0.0.0.0 --port 8000
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
  "message": "Collection 'test' deleted successfully"
}
```

## Error Handling

All endpoints return HTTP 2xx status codes on success. Error responses include:

- **401**: Missing Authorization header
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
