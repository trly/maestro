.PHONY: dev build check test icon clean help sidecar

# Default target
.DEFAULT_GOAL := help

# Detect platform and architecture
UNAME_S := $(shell uname -s)
UNAME_M := $(shell uname -m)

ifeq ($(UNAME_S),Darwin)
  ifeq ($(UNAME_M),arm64)
    TARGET := aarch64-apple-darwin
  else
    TARGET := x86_64-apple-darwin
  endif
else ifeq ($(UNAME_S),Linux)
  TARGET := x86_64-unknown-linux-gnu
else
  $(error Unsupported platform: $(UNAME_S))
endif

BINARIES_DIR := src-tauri/binaries
SIDECAR_BIN := $(BINARIES_DIR)/amp-executor-$(TARGET)
RESOURCES_DIR := src-tauri/resources

## Development
dev: sidecar ## Start Tauri app in development mode
	bun install
	cargo tauri dev

sidecar: ## Build amp-executor sidecar binary
	@echo "Building amp-executor sidecar for $(TARGET)..."
	@mkdir -p $(BINARIES_DIR)
	@mkdir -p $(RESOURCES_DIR)/amp-executor
	@cd amp-executor && bun install && bun run build
	@cp amp-executor/dist/index.js $(RESOURCES_DIR)/amp-executor/index.js
	@cp -r amp-executor/node_modules $(RESOURCES_DIR)/amp-executor/
	@echo '#!/usr/bin/env bun' > $(SIDECAR_BIN)
	@cat $(RESOURCES_DIR)/amp-executor/index.js >> $(SIDECAR_BIN)
	@chmod +x $(SIDECAR_BIN)
	@echo "✓ Sidecar binary built: $(SIDECAR_BIN)"

build: sidecar ## Build production installer
	bun install
	cargo tauri build

## Quality checks
check: ## Run TypeScript and Rust checks
	bun run check
	cd src-tauri && cargo check
	cd src-tauri && cargo clippy -- -D warnings
	cd src-tauri && cargo fmt -- --check

tidy: ## Format frontend and backend code
	bun run format
	cd src-tauri && cargo fmt

test: ## Run Rust test suite
	cd src-tauri && cargo test

test-watch: ## Run Rust tests in watch mode
	cd src-tauri && cargo watch -x test

## Utilities
icon: ## Generate app icons from source image
	cargo tauri icon

clean: ## Clean build artifacts
	rm -rf node_modules bun.lock
	cd src-tauri && cargo clean
	rm -rf dist/
	rm -rf .svelte-kit/
	rm -rf $(BINARIES_DIR)
	rm -rf $(RESOURCES_DIR)/amp-executor
	cd amp-executor && rm -rf node_modules bun.lock dist/

## Help
help: ## Show this help message
	@echo "Maestro - Tauri Development Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'
