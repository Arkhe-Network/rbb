# Makefile — Cathedral ARKHE Substrato 8000
# Selo: CATHEDRAL-ARKHE-8000-MAKEFILE-v1.0.0-2026-06-18

.PHONY: all build test test-unit test-integration test-e2e deploy deploy-local clean lint fmt check docs

# Variables
RUST_VERSION ?= 1.75
FEATURES ?= mcp-server,proxy-mode,cli,metrics,ema,cache-aligner,cross-agent-memory,zvec-bridge,tensorzkp-bridge
DOCKER_COMPOSE := docker-compose -f docker-compose.yml -p cathedral-headroom

# Default target
all: check build test

# ============================================================
# BUILD
# ============================================================

build:
	@echo "🔨 Building Cathedral Headroom..."
	cargo build --release --features "$(FEATURES)"

build-debug:
	cargo build --features "$(FEATURES)"

build-mcp:
	cargo build --release --bin cathedral-headroom-mcp --features "mcp-server,cli,metrics,ema"

build-proxy:
	cargo build --release --bin cathedral-headroom-proxy --features "proxy-mode,cli,metrics,ema"

# ============================================================
# TEST
# ============================================================

test: test-unit test-integration

test-unit:
	@echo "🧪 Running unit tests..."
	cargo test --lib --features "$(FEATURES)"

test-integration:
	@echo "🧪 Running integration tests..."
	cargo test --test '*' --features "$(FEATURES),test-integration"

test-e2e:
	@echo "🧪 Running E2E tests..."
	cargo test --test e2e_idt_audit_b20 --features "$(FEATURES),test-e2e"
	cargo test --test e2e_headroom_proxy --features "$(FEATURES),test-e2e"

test-coverage:
	@echo "📊 Running tests with coverage..."
	cargo tarpaulin --out Html --features "$(FEATURES)"

# ============================================================
# DOCKER
# ============================================================

docker-build:
	@echo "🐳 Building Docker images..."
	$(DOCKER_COMPOSE) build

docker-up:
	@echo "🚀 Starting services..."
	$(DOCKER_COMPOSE) up -d --wait

docker-down:
	@echo "🛑 Stopping services..."
	$(DOCKER_COMPOSE) down

docker-logs:
	$(DOCKER_COMPOSE) logs -f --tail=100

docker-clean:
	$(DOCKER_COMPOSE) down -v
	docker system prune -f

# ============================================================
# DEPLOY
# ============================================================

deploy: deploy-local

deploy-local:
	@echo "🚀 Deploying locally..."
	./deploy.sh deploy

deploy-staging:
	@echo "🚀 Deploying to staging..."
	ENV=staging ./deploy.sh deploy

deploy-production:
	@echo "🚀 Deploying to production..."
	ENV=production ./deploy.sh deploy

# ============================================================
# QUALITY
# ============================================================

check:
	@echo "🔍 Running checks..."
	cargo check --features "$(FEATURES)"
	cargo clippy --features "$(FEATURES)" -- -D warnings
	cargo fmt --check

lint:
	cargo clippy --features "$(FEATURES)" -- -D warnings

fmt:
	cargo fmt

fix:
	cargo fix --allow-dirty --features "$(FEATURES)"

audit:
	cargo audit

# ============================================================
# DOCUMENTATION
# ============================================================

docs:
	cargo doc --no-deps --features "$(FEATURES)"

docs-serve: docs
	cd target/doc && python3 -m http.server 8080

# ============================================================
# CLEAN
# ============================================================

clean:
	cargo clean
	rm -rf target/
	rm -rf data/
	rm -rf logs/

# ============================================================
# UTILITIES
# ============================================================

bench:
	cargo bench --features "$(FEATURES)"

run-mcp:
	cargo run --bin cathedral-headroom-mcp --features "mcp-server,cli"

run-proxy:
	cargo run --bin cathedral-headroom-proxy --features "proxy-mode,cli"

shell-mcp:
	$(DOCKER_COMPOSE) exec cathedral-headroom-mcp /bin/sh

shell-proxy:
	$(DOCKER_COMPOSE) exec cathedral-headroom-proxy /bin/sh

prometheus-query:
	@echo "Available metrics:"
	@curl -s http://localhost:9090/api/v1/label/__name__/values | jq '.data[]' | grep headroom

grafana-password:
	@echo "Grafana admin password:"
	@cat secrets/grafana-admin-password.txt

# ============================================================
# HELP
# ============================================================

help:
	@echo "Cathedral ARKHE Substrato 8000 — Available targets:"
	@echo ""
	@echo "Build:"
	@echo "  build              Build release binary"
	@echo "  build-debug        Build debug binary"
	@echo "  build-mcp          Build MCP server only"
	@echo "  build-proxy        Build Proxy server only"
	@echo ""
	@echo "Test:"
	@echo "  test               Run all tests"
	@echo "  test-unit          Run unit tests"
	@echo "  test-integration   Run integration tests"
	@echo "  test-e2e           Run E2E tests"
	@echo "  test-coverage      Run tests with coverage"
	@echo ""
	@echo "Docker:"
	@echo "  docker-build       Build Docker images"
	@echo "  docker-up          Start services"
	@echo "  docker-down        Stop services"
	@echo "  docker-logs        View logs"
	@echo "  docker-clean       Clean Docker resources"
	@echo ""
	@echo "Deploy:"
	@echo "  deploy             Deploy locally"
	@echo "  deploy-staging     Deploy to staging"
	@echo "  deploy-production  Deploy to production"
	@echo ""
	@echo "Quality:"
	@echo "  check              Run all checks"
	@echo "  lint               Run clippy"
	@echo "  fmt                Format code"
	@echo "  fix                Auto-fix issues"
	@echo "  audit              Run security audit"
	@echo ""
	@echo "Documentation:"
	@echo "  docs               Generate documentation"
	@echo "  docs-serve         Serve documentation"
	@echo ""
	@echo "Utilities:"
	@echo "  bench              Run benchmarks"
	@echo "  run-mcp            Run MCP server"
	@echo "  run-proxy          Run Proxy server"
	@echo "  clean              Clean build artifacts"
	@echo "  help               Show this help"
