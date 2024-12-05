FROM debian:bookworm

RUN apt-get update && apt-get install -y cmake sudo clang curl zsh git pkg-config libssl-dev

RUN mkdir -p /usr/local/cargo \
    && useradd -m taxer \
    && chown -R taxer:taxer /usr/local/cargo \
    && echo 'taxer ALL=(ALL) NOPASSWD:ALL' >> /etc/sudoers

USER taxer

RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y && \
    sh -c "$(curl -fsSL https://raw.githubusercontent.com/ohmyzsh/ohmyzsh/master/tools/install.sh)" "" --unattended
