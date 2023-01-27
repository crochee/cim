.PHONY: all
all: fmt ci run

.PHONY: fmt
fmt:
	cargo fmt
.PHONY: ci
ci:
	cargo clippy

.PHONY: run
run:
	cargo run --release