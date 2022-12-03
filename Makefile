.PHONY: all
all: fmt ci

.PHONY: fmt
fmt:
	cargo fmt
.PHONY: ci
ci:
	cargo clippy