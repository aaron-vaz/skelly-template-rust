# Project
NAME := {{.Inputs.project_name.Value}}

.PHONY: all clean build test lint fmt deps run{{if eq .Inputs.project_type.Value "cli"}} release{{else}} docker{{end}} help

# Build directory
BUILD_DIR := build
{{- if eq .Inputs.project_type.Value "cli"}}
BINARY_DIR := $(BUILD_DIR)/bin

# Release binaries are named <project>_<os>_<arch>, for the platform that runs make
OS := $(shell uname -s | tr '[:upper:]' '[:lower:]')
ARCH := $(patsubst x86_64,amd64,$(patsubst aarch64,arm64,$(shell uname -m)))
{{- else}}

# Version information
VERSION := $(shell git describe --tags --always --dirty 2>/dev/null || echo dev)
{{- end}}

# Default target
all: lint test build ## Lint, test and build

clean: ## Remove build artifacts
	@echo "Cleaning build artifacts..."
	@cargo clean
	@rm -rf $(BUILD_DIR)

build: ## Build an optimized binary for the current platform -> target/release/$(NAME)
	@echo "Building binary..."
	@cargo build --release

test: ## Run the unit, integration and doc tests
	@echo "Running tests..."
	@cargo test
{{- if eq .Inputs.project_type.Value "cli"}}

release: test ## Test, then build $(NAME)_$(OS)_$(ARCH) -> build/bin
	@echo "Building release binary for $(OS)/$(ARCH)..."
	@cargo build --release
	@mkdir -p $(BINARY_DIR)
	@cp target/release/$(NAME) $(BINARY_DIR)/$(NAME)_$(OS)_$(ARCH)
	@echo "Built release artifacts in $(BINARY_DIR)"

run: ## Run the app, pass arguments with ARGS="greet --name Ferris"
	@cargo run --quiet -- $(ARGS)
{{- else}}

run: ## Run the service locally
	@cargo run --quiet

docker: ## Build the container image
	@docker build -t $(NAME):$(VERSION) .
{{- end}}

lint: ## Check formatting, then run clippy with warnings as errors
	@echo "Running linter..."
	@cargo fmt --all -- --check
	@cargo clippy --all-targets -- -D warnings

fmt: ## Format the code
	@cargo fmt --all

deps: ## Install the pinned toolchain (rustfmt and clippy included) and fetch crates
	@echo "Installing dependencies..."
	@rustup toolchain install
	@cargo fetch

help: ## Show this help
	@grep -E '^[a-z_-]+:.*## ' $(MAKEFILE_LIST) | awk -F':.*## ' '{printf "  %-8s %s\n", $$1, $$2}'
