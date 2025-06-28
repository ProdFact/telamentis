# TelaMentis Development Makefile

.PHONY: help dev-up dev-down build test lint fmt check clean docs

# Default target
help:
	@echo "TelaMentis Development Commands:"
	@echo "  dev-up      - Start development environment (Neo4j + Core + FastAPI)"
	@echo "  dev-down    - Stop development environment"
	@echo "  build       - Build all Rust components"
	@echo "  test        - Run all tests"
	@echo "  lint        - Run clippy linter"
	@echo "  fmt         - Format code"
	@echo "  check       - Run all checks (fmt, lint, test)"
	@echo "  clean       - Clean build artifacts"
	@echo "  docs        - Generate documentation"

# Development environment
dev-up:
	@echo "Starting TelaMentis development environment..."
	docker-compose up -d
	@echo "Services starting up. Access:"
	@echo "  - Neo4j Browser: http://localhost:7474"
	@echo "  - FastAPI Docs:  http://localhost:8000/docs"
	@echo "  - Core Service:  http://localhost:3000"

dev-down:
	@echo "Stopping TelaMentis development environment..."
	docker-compose down

# Build commands
build:
	@echo "Building TelaMentis components..."
	cargo build --all-features

build-release:
	@echo "Building TelaMentis components (release)..."
	cargo build --release --all-features

# Test commands
test:
	@echo "Running tests..."
	cargo test --all-features

test-integration:
	@echo "Running integration tests..."
	cargo test --all-features integration

# Code quality
lint:
	@echo "Running clippy..."
	cargo clippy --all-targets --all-features -- -D warnings

fmt:
	@echo "Formatting code..."
	cargo fmt --all

fmt-check:
	@echo "Checking code formatting..."
	cargo fmt --all -- --check

# Combined checks
check: fmt-check lint test
	@echo "All checks passed!"

# Documentation
docs:
	@echo "Generating documentation..."
	cargo doc --all-features --no-deps

docs-open:
	@echo "Opening documentation..."
	cargo doc --all-features --no-deps --open

# Cleanup
clean:
	@echo "Cleaning build artifacts..."
	cargo clean
	docker-compose down -v

# CLI tools
install-kgctl:
	@echo "Installing kgctl..."
	cargo install --path kgctl

# Database operations
db-reset:
	@echo "Resetting Neo4j database..."
	docker-compose stop neo4j
	docker volume rm telamentis_neo4j_data || true
	docker-compose up -d neo4j

# Logs
logs:
	docker-compose logs -f

logs-core:
	docker-compose logs -f telamentis-core

logs-neo4j:
	docker-compose logs -f neo4j

logs-fastapi:
	docker-compose logs -f fastapi

# Quick development workflow
quick-test: fmt lint test
	@echo "Quick test cycle completed!"

# Full CI-like check
ci-check: fmt-check lint test docs
	@echo "CI checks completed!"