.PHONY: all
all: fmt check clippy test

.PHONY: fmt
fmt:
	# @cargo fmt -- --check
	@cargo fmt
.PHONY: clippy
clippy:
	# @cargo clippy --all-targets --all-features --tests --benches -- -D warnings
	@cargo clippy --all-targets --tests --benches
.PHONY: check
check:
	@cargo check --all
.PHONY: test
test:
	# @cargo test --all-features --all
	@cargo test --all

.PHONY: server
server:
	@cargo run --release --bin server

.PHONY: migrate
migrate:
	@sqlx database create && sqlx migrate run

.PHONY: database
database:
	@cargo install sqlx-cli --no-default-features --features rustls,mysql

.PHONY: rs
rs:
	sudo docker build -f ./server/build/Dockerfile -t server:latest . && \
	sudo docker run -itd -p 30050:30050 --restart=always --name server server:latest
