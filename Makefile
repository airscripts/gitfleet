CARGO ?= cargo
JOBS ?= 4

.DEFAULT_GOAL := help
.PHONY: help install fmt clippy check test coverage build release metrics verify

help:
	@printf '%s\n' 'Gitfleet development targets:'
	@printf '%s\n' '  install   Install the local gitfleet binaries'
	@printf '%s\n' '  fmt       Check Rust formatting'
	@printf '%s\n' '  clippy    Run clippy with warnings denied'
	@printf '%s\n' '  check     Type-check the workspace'
	@printf '%s\n' '  test      Run the workspace tests'
	@printf '%s\n' '  coverage  Run the 80% coverage gate'
	@printf '%s\n' '  build     Build the debug workspace'
	@printf '%s\n' '  release   Build optimized binaries'
	@printf '%s\n' '  metrics   Refresh LOC and test-count reports'
	@printf '%s\n' '  verify    Run the complete local quality workflow'

install:
	$(CARGO) install --path gitfleet --force

fmt:
	$(CARGO) fmt --check

clippy:
	CARGO_BUILD_JOBS=$(JOBS) $(CARGO) clippy -- -D warnings

check:
	CARGO_BUILD_JOBS=$(JOBS) $(CARGO) check --workspace

test:
	CARGO_BUILD_JOBS=$(JOBS) $(CARGO) test --workspace

coverage:
	CARGO_BUILD_JOBS=$(JOBS) $(CARGO) llvm-cov --fail-under-lines 80 --workspace

build:
	CARGO_BUILD_JOBS=$(JOBS) $(CARGO) build --workspace

release:
	CARGO_BUILD_JOBS=$(JOBS) $(CARGO) build --release

metrics:
	./gitfleet-scripts/loc.sh
	./gitfleet-scripts/tests.sh

verify: fmt clippy check test coverage release metrics
