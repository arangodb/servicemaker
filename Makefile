.PHONY: release clean help test-service test-service-nodejs

# ARCH (optional, amd64|arm64): build the test-service images for linux/$(ARCH)
# and tag them arangodb/<image>:latest-$(ARCH). Unset: host platform, arangodb/<image>.
# PUSH=0 builds without pushing (default: push, as before).
ARCH ?=
PUSH ?= 1
TEST_SERVICE_TAG = $(if $(ARCH),:latest-$(ARCH),)
PUSH_FLAG = $(if $(filter 1,$(PUSH)),--push,)
PLATFORM_ENV = $(if $(ARCH),DOCKER_DEFAULT_PLATFORM=linux/$(ARCH),)

# Default target
help:
	@echo "Available targets:"
	@echo "  release - Build static x86_64 binary and prepare for GitHub release"
	@echo "  clean   - Clean build artifacts"
	@echo "  test-servie - Build the Docker image `arangodb/test-service`
	@echo "  test-service-nodejs - Build the Docker image `arangodb/test-service-nodejs`

# Build static binary for GitHub release
release:
	@echo "=== Building static x86_64 binary for release ==="
	@echo ""
	@echo "Step 1: Adding x86_64-unknown-linux-musl target..."
	rustup target add x86_64-unknown-linux-musl
	@echo ""
	@echo "Step 2: Building release binary..."
	cargo build --release --target x86_64-unknown-linux-musl
	@echo ""
	@echo "Step 3: Copying binary to project root..."
	cp target/x86_64-unknown-linux-musl/release/servicemaker ./servicemaker
	@echo ""
	@echo "✓ Static binary built successfully!"
	@echo ""
	@echo "Binary details:"
	@file servicemaker
	@ls -lh servicemaker
	@echo ""
	@echo "=== Release Ready ==="
	@echo "The static binary is now in the project root:"
	@echo "  - servicemaker"
	@echo ""
	@echo "This binary is statically linked and can run on any x86_64 Linux system."
	@echo ""
	@echo "You can now create a binary release on GitHub:"
	@echo "  1. Go to: https://github.com/arangodb/servicemaker/releases/new"
	@echo "  2. Create a new tag (e.g., v0.9.3)"
	@echo "  3. Upload the 'servicemaker' binary"
	@echo "  4. Publish the release"
	@echo ""

# Clean build artifacts
clean:
	cargo clean
	rm -f servicemaker
	@echo "✓ Cleaned build artifacts"

test-service:
	$(PLATFORM_ENV) target/release/servicemaker --project-home arango-test-service --port 8000 --make-tar-gz $(PUSH_FLAG) --image-name arangodb/test-service$(TEST_SERVICE_TAG)

test-service-nodejs:
	$(PLATFORM_ENV) target/release/servicemaker --project-home arango-test-service-nodejs --port 8000 --make-tar-gz $(PUSH_FLAG) --image-name arangodb/test-service-nodejs$(TEST_SERVICE_TAG)
