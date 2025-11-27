# Docker Hub Setup for CircleCI

Note: Max Neunhöffer has done the following to setup image pushing
      using the `neunhoef` account to produce an access token.

## Overview

The `rebuild-base-images-manual` workflow in CircleCI requires Docker Hub credentials to push images to the `arangodb` organization.

## Setting Up Docker Hub Credentials

### Step 1: Create a Docker Hub Access Token

1. Go to [Docker Hub](https://hub.docker.com/) and log in with your account
2. Click on your username in the top right corner and select **Account Settings**
3. Navigate to **Security** in the left sidebar
4. Click on **New Access Token**
5. Give your token a description (e.g., "CircleCI ArangoDB Base Images")
6. Set the access permissions:
   - **Access permissions**: Read, Write, Delete (needed to push images)
   - **Scope**: Select the `arangodb` organization if you have access
7. Click **Generate**
8. **Important**: Copy the token immediately - you won't be able to see it again!

### Step 2: Create a Context in CircleCI

1. Go to your CircleCI project: https://app.circleci.com/
2. Click on **Organization Settings** in the left sidebar
3. Click on **Contexts**
4. Click **Create Context**
5. Name it exactly: `dockerhub-credentials` (this matches the context referenced in the workflow)
6. Click **Create Context**

### Step 3: Add Environment Variables to the Context

1. Click on the `dockerhub-credentials` context you just created
2. Click **Add Environment Variable**
3. Add the first variable:
   - **Name**: `DOCKERHUB_USERNAME`
   - **Value**: Your Docker Hub username (must have push access to the `arangodb` organization)
   - Click **Add Environment Variable**
4. Add the second variable:
   - **Name**: `DOCKERHUB_PASSWORD`
   - **Value**: The access token you created in Step 1 (paste the token here)
   - Click **Add Environment Variable**

### Step 4: Verify Permissions

Ensure that your Docker Hub account has the necessary permissions:
- You must be a member of the `arangodb` organization on Docker Hub
- Your role must have **push/write** permissions to publish images

Contact the organization administrator if you need to be added or granted the appropriate permissions.

## Triggering the Base Images Rebuild

### Method 1: Using the CircleCI API

You can trigger the workflow using the CircleCI API with curl:

```bash
curl -X POST \
  --header "Content-Type: application/json" \
  --header "Circle-Token: YOUR_CIRCLECI_API_TOKEN" \
  --data '{
    "parameters": {
      "rebuild_base_images": true
    }
  }' \
  https://circleci.com/api/v2/project/gh/YOUR_ORG/servicemaker/pipeline
```

Replace:
- `YOUR_CIRCLECI_API_TOKEN`: Your personal CircleCI API token (create one in CircleCI User Settings > Personal API Tokens)
- `YOUR_ORG`: Your GitHub organization or username

### Method 2: Using the CircleCI Web UI

1. Go to your project in CircleCI
2. Click on **Trigger Pipeline** (top right)
3. Add a parameter:
   - **Parameter Type**: `boolean`
   - **Name**: `rebuild_base_images`
   - **Value**: `true`
4. Click **Trigger Pipeline**

### Method 3: Using CircleCI CLI

Install the CircleCI CLI and run:

```bash
circleci trigger pipeline \
  --param rebuild_base_images=true \
  --org YOUR_ORG \
  --repo servicemaker
```

## What the Workflow Does

When triggered with `rebuild_base_images=true`, the workflow will:

1. Checkout the code
2. Set up Docker with layer caching
3. Log in to Docker Hub using the credentials from the context
4. Run `make build` in the `baseimages` directory to build all base images
5. Run `make push` in the `baseimages` directory to push all images to `arangodb` organization

The workflow has a 60-minute timeout for the build step to accommodate large image builds.

## Troubleshooting

### "denied: requested access to the resource is denied"

This means your Docker Hub account doesn't have push access to the `arangodb` organization. Contact the organization administrator.

### "Error saving credentials: error storing credentials"

This is usually a transient Docker login issue. Re-run the workflow.

### "DOCKERHUB_USERNAME or DOCKERHUB_PASSWORD not set"

Make sure:
1. The context name is exactly `dockerhub-credentials`
2. The environment variables are named exactly `DOCKERHUB_USERNAME` and `DOCKERHUB_PASSWORD`
3. Your CircleCI project has access to the context (check Context > Settings > Security)

## Security Notes

- **Never commit Docker Hub credentials to the repository**
- Access tokens are more secure than passwords and can be revoked individually
- Use tokens with the minimum required permissions
- Regularly rotate access tokens (every 6-12 months)
- Monitor the audit logs in Docker Hub for suspicious activity

