FROM lukemathwalker/cargo-chef:latest AS chef
WORKDIR app

FROM chef AS planner
# 代码拷贝
RUN git clone -b release-v1.0.0 https://github.com/crochee/cim.git

RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
RUN git clone -b release-v1.0.0 https://github.com/crochee/cim.git
RUN cargo build --release

# We do not need the Rust toolchain to run the binary!
FROM ubuntu AS runtime
WORKDIR app
COPY --from=builder /app/target/release/server /usr/local/bin
COPY --from=builder /app/.env  .
COPY --from=builder /app/server/entrypoint.sh /usr/local/bin/
# 赋予执行权限
RUN chmod +x /usr/local/bin/server /usr/local/bin/entrypoint.sh

EXPOSE 30050
STOPSIGNAL 2

ENTRYPOINT ["entrypoint.sh"]
CMD ["server"]
