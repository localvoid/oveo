export PATH := $(shell pwd)/node_modules/.bin:$(PATH)
SHELL := bash
.ONESHELL:

.DEFAULT_GOAL := help

help:
	@printf 'Available targets:\n\n'
	@grep -E '^[a-zA-Z0-9_-]+:' Makefile | grep -v '.DEFAULT' | sort | awk -F':' '{print "  make " $$1}'

# ──────────────────────────────────────────────
# Root package targets
# ──────────────────────────────────────────────

init:
	bun install

tsc:
	tsc -b $(FLAGS)

publish:
	./scripts/publish-package.sh ./packages/@oveo/rolldown/ $(NPM_FLAGS)
	./scripts/publish-package.sh ./packages/@oveo/rollup/ $(NPM_FLAGS)
	./scripts/publish-package.sh ./packages/@oveo/vite/ $(NPM_FLAGS)
	./scripts/publish-package.sh ./packages/oveo/ $(NPM_FLAGS)

# ──────────────────────────────────────────────
# NAPI (@oveo/optimizer) targets
# ──────────────────────────────────────────────

NAPI_PKG_DIR := ./packages/@oveo/optimizer

napi-build:
	napi build --platform --esm --manifest-path $(NAPI_PKG_DIR)/Cargo.toml --package-json-path $(NAPI_PKG_DIR)/package.json --output-dir $(NAPI_PKG_DIR) $(FLAGS)

napi-test:
	bun test ./tests/optimizer/

napi-create-npm-dirs:
	napi create-npm-dirs --npm-dir $(NAPI_PKG_DIR)/packages --package-json-path $(NAPI_PKG_DIR)/package.json

napi-artifacts:
	napi artifacts --npm-dir $(NAPI_PKG_DIR)/packages --package-json-path $(NAPI_PKG_DIR)/package.json --output-dir ./napi-artifacts

napi-update-versions:
	./scripts/set-package-version.sh $(NAPI_PKG_DIR)/packages/darwin-arm64 $(VERSION)
	./scripts/set-package-version.sh $(NAPI_PKG_DIR)/packages/darwin-x64 $(VERSION)
	./scripts/set-package-version.sh $(NAPI_PKG_DIR)/packages/linux-arm64-gnu $(VERSION)
	./scripts/set-package-version.sh $(NAPI_PKG_DIR)/packages/linux-x64-gnu $(VERSION)
	./scripts/set-package-version.sh $(NAPI_PKG_DIR)/packages/win32-arm64-msvc $(VERSION)
	./scripts/set-package-version.sh $(NAPI_PKG_DIR)/packages/win32-x64-msvc $(VERSION)

napi-increment-versions:
	./scripts/napi-increment-versions.sh $(INCREMENT)

napi-publish:
	./scripts/publish-package.sh $(NAPI_PKG_DIR)/packages/darwin-arm64 $(NPM_FLAGS)
	./scripts/publish-package.sh $(NAPI_PKG_DIR)/packages/darwin-x64 $(NPM_FLAGS)
	./scripts/publish-package.sh $(NAPI_PKG_DIR)/packages/linux-arm64-gnu $(NPM_FLAGS)
	./scripts/publish-package.sh $(NAPI_PKG_DIR)/packages/linux-x64-gnu $(NPM_FLAGS)
	./scripts/publish-package.sh $(NAPI_PKG_DIR)/packages/win32-arm64-msvc $(NPM_FLAGS)
	./scripts/publish-package.sh $(NAPI_PKG_DIR)/packages/win32-x64-msvc $(NPM_FLAGS)
	./scripts/publish-package.sh $(NAPI_PKG_DIR)/ $(NPM_FLAGS)
