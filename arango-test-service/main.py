#!/usr/bin/env python
"""
Small FastAPI service for ArangoDB operations.
Provides endpoints to write, read, and delete data from ArangoDB.
"""

import os
import sys
from typing import Any, Dict

from arango.client import ArangoClient
from arango.exceptions import ArangoError
from fastapi import FastAPI, Header, HTTPException, Request

app = FastAPI(title="ArangoDB Service")


def startup_check():
    """Check required environment variables at startup."""
    endpoint = os.getenv("ARANGO_DEPLOYMENT_ENDPOINT")
    if not endpoint:
        print("ERROR: ARANGO_DEPLOYMENT_ENDPOINT environment variable not set", file=sys.stderr)
        sys.exit(1)


def get_arango_client():
    """Create and return an ArangoDB client with JWT authentication."""
    endpoint = os.getenv("ARANGO_DEPLOYMENT_ENDPOINT")
    if not endpoint:
        raise ValueError("ARANGO_DEPLOYMENT_ENDPOINT environment variable not set")

    # Disable SSL certificate verification for HTTPS endpoints (e.g., self-signed certificates)
    client = ArangoClient(hosts=endpoint, verify_override=False)
    return client


def get_system_database(token: str):
    """Get authenticated system database connection."""
    client = get_arango_client()
    # Use JWT token for authentication
    try:
        db = client.db(
            name="_system",
            auth_method="jwt",
            user_token=token
        )
    except Exception as e:
        print("Caught exception:", e)
        sys.exit(1)
    return db


def get_database(token: str, db_name: str = "test-service"):
    """Get authenticated database connection, creating it if it doesn't exist."""
    client = get_arango_client()
    sys_db = get_system_database(token)
    
    # Create database if it doesn't exist
    if not sys_db.has_database(db_name):
        sys_db.create_database(db_name)
    
    # Use JWT token for authentication
    try:
        db = client.db(
            name=db_name,
            auth_method="jwt",
            user_token=token
        )
    except Exception as e:
        print("Caught exception:", e)
        sys.exit(1)
    return db


def get_existing_database(token: str, db_name: str = "test-service"):
    """Get authenticated database connection, failing if it doesn't exist."""
    client = get_arango_client()
    sys_db = get_system_database(token)
    
    # Check if database exists
    if not sys_db.has_database(db_name):
        raise ValueError(f"Database '{db_name}' does not exist")
    
    # Use JWT token for authentication
    try:
        db = client.db(
            name=db_name,
            auth_method="jwt",
            user_token=token
        )
    except Exception as e:
        print("Caught exception:", e)
        sys.exit(1)
    return db


@app.post("/write")
async def write_data(
    request: Request,
    authorization: str = Header(None)
) -> Dict[str, Any]:
    """
    Write data to ArangoDB.
    Creates 'test-service' database and 'test' collection if they don't exist,
    and inserts the request body as a document.
    Fails if the database already exists to prevent accidental data corruption.
    """
    if not authorization:
        raise HTTPException(status_code=401, detail="Authorization header required")

    # Extract token (assuming format "Bearer <token>" or just the token)
    token = authorization.replace("bearer ", "").replace("Bearer ", "").strip()
    
    try:
        # Parse request body
        body = await request.json()

        # Get database connection (creates database since we verified it doesn't exist)
        db = get_database(token, "test-service")

        # Create collection if it doesn't exist
        if not db.has_collection("test"):
            db.create_collection("test")

        # Get collection and insert document
        collection = db.collection("test")
        result = collection.insert(body)
        if result:
            return {
                "status": "success",
                "message": "Document written successfully",
                "document_key": result.get("_key")
            }
        raise HTTPException(
                status_code=400,
                detail="ArangoDB error: got None result"
            )
    except ValueError as e:
        raise HTTPException(status_code=500, detail=str(e)) from e
    except ArangoError as e:
        raise HTTPException(
            status_code=500,
            detail=f"ArangoDB error: {str(e)}"
        ) from e
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Unexpected error: {str(e)}"
        ) from e


@app.get("/read")
async def read_data(authorization: str = Header(None)) -> Dict[str, Any]:
    """
    Read all documents from the 'test' collection in the 'test-service' database.
    Fails if the database doesn't exist.
    """
    if not authorization:
        raise HTTPException(status_code=401, detail="Authorization header required")

    token = authorization.replace("bearer ", "").replace("Bearer ", "").strip()
    
    try:
        db = get_existing_database(token, "test-service")

        # Check if collection exists
        if not db.has_collection("test"):
            return {
                "status": "success",
                "message": "Collection 'test' does not exist",
                "documents": []
            }

        # Get all documents from collection
        collection = db.collection("test")
        documents = list(collection.all())

        return {
            "status": "success",
            "message": f"Retrieved {len(documents)} document(s)",
            "documents": documents
        }

    except ValueError as e:
        raise HTTPException(status_code=500, detail=str(e)) from e
    except ArangoError as e:
        raise HTTPException(
            status_code=500,
            detail=f"ArangoDB error: {str(e)}"
        ) from e
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Unexpected error: {str(e)}"
        ) from e


@app.delete("/delete")
async def delete_data(authorization: str = Header(None)) -> Dict[str, Any]:
    """
    Drop the 'test-service' database and all its contents.
    Fails if the database doesn't exist.
    """
    if not authorization:
        raise HTTPException(status_code=401, detail="Authorization header required")

    token = authorization.replace("bearer ", "").replace("Bearer ", "").strip()

    try:
        sys_db = get_system_database(token)

        # Check if database exists - fail if it doesn't
        if not sys_db.has_database("test-service"):
            raise HTTPException(
                status_code=404,
                detail="Database 'test-service' does not exist, nothing to delete"
            )

        # Drop the database
        sys_db.delete_database("test-service")

        return {
            "status": "success",
            "message": "Database 'test-service' deleted successfully"
        }

    except ValueError as e:
        raise HTTPException(status_code=500, detail=str(e)) from e
    except ArangoError as e:
        raise HTTPException(
            status_code=500,
            detail=f"ArangoDB error: {str(e)}"
        ) from e
    except Exception as e:
        raise HTTPException(
            status_code=500,
            detail=f"Unexpected error: {str(e)}"
        ) from e


@app.get("/health")
async def health_check() -> Dict[str, str]:
    """Health check endpoint."""
    return {"status": "healthy"}


if __name__ == "__main__":
    startup_check()
    import uvicorn
    uvicorn.run(app, host="0.0.0.0", port=8000)
