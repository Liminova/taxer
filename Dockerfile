FROM rust:alpine AS builder

RUN apk add --no-cache make cmake musl-dev libssl3 g++ gcc git libressl-dev clang libstdc++-dev

WORKDIR /app
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN cargo build --release

FROM alpine:latest

ENV YT_DLP_PATH=/usr/local/bin/yt-dlp
ENV FFMPEG_PATH=/usr/local/bin/ffmpeg

COPY --from=builder /app/target/release/taxer /usr/local/bin/taxer
ENTRYPOINT ["/usr/local/bin/taxer"]