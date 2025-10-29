.PHONY: dev build check test icon clean help

# Default target
.DEFAULT_GOAL := help

## Development
dev: ## Start Tauri app in development mode
	cargo tauri dev

build: ## Build production installer
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
	cargo clean
	rm -rf dist/
	rm -rf .svelte-kit/

## Help
help: ## Show this help message
	@echo "Maestro - Tauri Development Commands"
	@echo ""
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' $(MAKEFILE_LIST) | \
		awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-15s\033[0m %s\n", $$1, $$2}'
