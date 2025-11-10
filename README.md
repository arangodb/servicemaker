# ServiceMaker

A tool to wrap Python projects as Docker services.

## Features

- Takes an existing Python project and creates a Docker image
- Interactive prompts for missing configuration
- Customizable base image and entrypoint
- Optional image push to Docker registry

## Installation

Build the project:

```bash
cargo build --release
```

The binary will be available at `target/release/servicemaker`.

## Usage

### With all options specified:

```bash
servicemaker \
  --name myproject \
  --project-home /path/to/python/project \
  --base-image python:3.14-slim-trixie \
  --port 8080 \
  --image-name myregistry/myproject:latest \
  --entrypoint main.py \
  --push
```

### Interactive mode (prompts for missing options):

```bash
servicemaker
```

### Command-line Options

- `--name` - Name of the project
- `--project-home` - Path to the folder containing the Python project
- `--base-image` - Base Docker image (default: `python:3.14-slim-trixie`)
- `--port` - Exposed port number
- `--image-name` - Docker image name to push
- `--push` - Whether to push the image (default: false)
- `--registry` - URL of the Docker registry (optional, default: Docker Hub)
- `--entrypoint` - Name of the Python script to run (relative to project home)

## How it Works

1. Reads command-line arguments or prompts for missing values
2. Validates that the project home directory exists
3. Creates a temporary directory (e.g., `/tmp/servicemaker-<projectname>`)
4. Modifies the Dockerfile template with:
   - Custom base image
   - EXPOSE directive for the specified port
   - Custom entrypoint script
5. Copies the Dockerfile to the temporary directory
6. Recursively copies the Python project to `project/` subdirectory
7. Runs `docker build` to create the image
8. Optionally runs `docker push` if `--push` is specified

## Dockerfile Template

The tool uses the Dockerfile in the project root as a template. The default template:

- Uses `uv` package manager for fast dependency installation
- Creates a non-root `python` user
- Copies the project to `/home/python/project`
- Runs `uv sync` to install dependencies
- Executes the specified entrypoint script

## Notes

- The temporary directory is left behind after execution for inspection
- The project directory must contain a valid Python project with a `pyproject.toml` or `requirements.txt` for uv to work properly
- Docker must be installed and accessible for this tool to work

