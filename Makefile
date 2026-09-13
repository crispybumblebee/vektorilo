# Vektorilo Makefile
# Tauri relies on the native OS toolchains, so cross-compiling from a single machine
# for all three operating systems (Windows, macOS, Linux) is highly complex.
# The standard and easiest way is to use GitHub Actions for cross-platform builds.
# See .github/workflows/release.yml for the CI/CD pipeline.

.PHONY: dev build check setup install

# Start the development server
dev:
	npm run tauri dev

# Build the application for your CURRENT operating system
build:
	npm run tauri build

# Check Rust code for errors
check:
	cd src-tauri && cargo check

# Install Node.js dependencies
install:
	npm install

# Full local setup
setup: install
	@echo "Dependencies installed. Run 'make dev' to start developing."
