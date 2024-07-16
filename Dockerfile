FROM rust:1.79.0-alpine3.20 AS builder

RUN apk add --no-cache make cmake musl-dev libssl3 g++ gcc git libressl-dev clang libstdc++-dev
WORKDIR /app

COPY ./src /app/src
COPY ./Cargo.toml /app/Cargo.toml
COPY ./Cargo.lock /app/Cargo.lock

RUN cargo build --release

FROM alpine:latest

COPY --from=builder /app/target/release/taxer /usr/local/bin/taxer

ENTRYPOINT ["/usr/local/bin/taxer"]