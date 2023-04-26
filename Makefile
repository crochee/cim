.PHONY: all
all: fmt ci check

.PHONY: fmt
fmt:
	@cargo fmt
.PHONY: ci
ci:
	@cargo clippy
.PHONY: check
check:
	@cargo check

.PHONY: server
server:
	@cargo run --release --bin server

.PHONY: migrate
migrate:
	@sqlx database create && sqlx migrate run

.PHONY: database
database:
	@cargo install sqlx-cli --no-default-features --features rustls,mysql
