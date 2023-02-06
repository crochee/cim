.PHONY: all
all: fmt ci run

.PHONY: fmt
fmt:
	@cargo fmt
.PHONY: ci
ci:
	@cargo clippy

.PHONY: server
run:
	@cargo run --release --bin iam-server

.PHONY: migrate
migrate:
	@sqlx database create && sqlx migrate run

.PHONY: database
database:
	@cargo install sqlx-cli --no-default-features --features rustls,mysql