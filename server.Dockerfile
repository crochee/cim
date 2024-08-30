FROM golang:1.21.5-alpine as builder
ARG GOSU_VERSION=1.17
WORKDIR /workspace
RUN go install github.com/tianon/gosu@${GOSU_VERSION} && cp ${GOPATH}/bin/gosu /workspace/gosu

FROM ubuntu
WORKDIR /app
# 资源拷贝
COPY --from=builder /workspace/gosu /usr/local/bin/
COPY ./target/release/server /usr/local/bin
COPY ./entrypoint.sh /usr/local/bin/
COPY server.toml /app/
# 赋予执行权限
RUN chmod +x /usr/local/bin/server /usr/local/bin/entrypoint.sh &&\
    groupadd -g 10000 cloud &&\
    useradd -u 5000 -g cloud -m dev

EXPOSE 30050
STOPSIGNAL 2

ENTRYPOINT ["entrypoint.sh"]
CMD ["-c", "/app/server.toml"]
