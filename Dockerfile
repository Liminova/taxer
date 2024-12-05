FROM rust:bookworm AS builder

RUN apt-get update && apt-get install -y libssl-dev pkg-config cmake clang

WORKDIR /app
COPY ./src ./src
COPY ./Cargo.toml ./Cargo.toml
COPY ./Cargo.lock ./Cargo.lock

RUN curl -L https://github.com/rui314/mold/releases/download/v2.34.1/mold-2.34.1-x86_64-linux.tar.gz | tar xz -C /tmp --strip-components=1

RUN /tmp/bin/mold -run cargo build --release

FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y libssl3 && rm apt-get clean

ENV YT_DLP_PATH=/usr/local/bin/yt-dlp
ENV FFMPEG_PATH=/usr/local/bin/ffmpeg

COPY --from=builder /app/target/release/taxer /taxer
ENTRYPOINT ["/taxer"]
