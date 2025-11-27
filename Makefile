.PHONY: release clean help

# Default target
help:
	@echo "Available targets:"
	@echo "  release - Build static x86_64 binary and prepare for GitHub release"
	@echo "  clean   - Clean build artifacts"

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

