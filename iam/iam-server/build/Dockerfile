FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin iam-server

# We do not need the Rust toolchain to run the binary!
FROM debian:buster-slim AS runtime
WORKDIR app
COPY --from=builder /app/target/release/iam-server /usr/local/bin
COPY --from=builder /app/.env  .
COPY --from=builder /app/iam/iam-server/build/entrypoint.sh /usr/local/bin/
# 赋予执行权限
RUN chmod +x /usr/local/bin/apiserver /usr/local/bin/entrypoint.sh

EXPOSE 30050
STOPSIGNAL 2

ENTRYPOINT ["entrypoint.sh"]
CMD ["iam-server"]
