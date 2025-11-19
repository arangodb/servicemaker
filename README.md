# ServiceMaker

A tool to wrap Python projects as Docker services.

## Features

- Takes an existing Python project and creates a Docker image
- Interactive prompts for missing configuration
- Uses pre-built base images with Python and common libraries pre-installed
- Customizable base image and entrypoint
- Optional image push to Docker registry
- Optional creation of tar.gz archive with project files and virtual environment changes

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
  --base-image neunhoef/py13base:latest \
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

- `--name` - Name of the project (optional, will prompt if not provided)
- `--project-home` - Path to the folder containing the Python project (optional, will prompt if not provided)
- `--base-image` - Base Docker image (default: `neunhoef/py13base:latest`)
- `--port` - Exposed port number (optional, will prompt if not provided)
- `--image-name` - Docker image name to push (optional, will prompt if not provided)
- `--push` - Whether to push the image (default: `false`)
- `--registry` - URL of the Docker registry (optional, no default)
- `--entrypoint` - Name of the Python script to run relative to project home (optional, will prompt if not provided)
- `--make-tar-gz` - Whether to create a tar.gz archive with project files and virtual environment changes (default: `false`)

## How it Works

1. Reads command-line arguments or prompts for missing values
2. Validates that the project home directory exists
3. Creates a temporary directory in the current directory (e.g., `./servicemaker-<projectname>-<pid>`)
4. Modifies the Dockerfile template with:
   - Custom base image
   - EXPOSE directive for the specified port
   - Custom entrypoint script
5. Copies the Dockerfile to the temporary directory
6. Recursively copies the Python project to `project/` subdirectory
7. Runs `docker build` to create the image
8. Optionally runs `docker push` if `--push` is specified
9. Optionally creates a tar.gz archive if `--make-tar-gz` is specified

## Base Images

ServiceMaker uses pre-built base images that provide a robust foundation for Python services. These base images:

- Start with `debian:trixie-slim` for a minimal, secure base
- Include a non-root `user` user with `uv` package manager pre-installed
- Have a specific Python version (e.g., Python 3.13) pre-installed via `uv`
- Create a virtual environment called `the_venv` in the user's home directory
- Can pre-install common libraries (like `networkx`) into the virtual environment
- Include a SHA256 checksum file of all files in the virtual environment for change tracking

The default base image is `neunhoef/py13base:latest`, which includes Python 3.13 and `networkx`.

### Building Base Images

Base images are defined in the `baseimages/` directory. To build a base image:

```bash
cd baseimages
make py13  # Builds neunhoef/py13base:latest
```

You can create additional base images for different Python versions or with different pre-installed libraries by creating new Dockerfiles in the `baseimages/` directory.

## Dockerfile Template

The tool uses the Dockerfile template in the project root. The template:

- Uses the specified base image (which already has `uv` and Python installed)
- Copies the project to `/home/user/project`
- Activates the existing virtual environment (`the_venv`) from the base image
- Runs `uv sync --active` to install only additional dependencies not already in the base image
- Executes the specified entrypoint script

This approach ensures that:
- Dependencies are installed efficiently (only new ones are added)
- The final image shares layers with the base image (faster builds and smaller images)
- Base images can be pre-scanned for security vulnerabilities
- The virtual environment works seamlessly with modern Python tooling

## Python Version

The Python version is determined by the base image you select. The default base image (`neunhoef/py13base:latest`) includes Python 3.13.

You should declare the Python version in your project's `pyproject.toml` file to match the base image's Python version. The `uv` package manager will use the Python version from the base image's virtual environment.

For example, to use Python 3.13 (matching the default base image):

```toml
requires-python = "==3.13.*"
```

When using `uv sync --active`, the tool installs dependencies into the existing virtual environment from the base image, ensuring compatibility with the pre-installed Python version.

## Notes

- The temporary directory is left behind after execution for inspection
- The project directory must contain a valid Python project with a `pyproject.toml` or `requirements.txt` for uv to work properly
- Docker must be installed and accessible for this tool to work
- Base images must be available locally or pulled from a registry before building
- The base image's Python version should match the Python version requirement in your `pyproject.toml`

