#!/bin/sh

DEVCONTAINER_DIR="/workspaces/taxer/.devcontainer"
MOLD_VERSION="2.34.1"
MOLD_MD5=08d7304ea9f5e232a5c46a45f230b5db

if [ ! -d $DEVCONTAINER_DIR/mold-$MOLD_VERSION-x86_64-linux ]; then
    cd $DEVCONTAINER_DIR
    curl -L -o mold-$MOLD_VERSION-x86_64-linux.tar.gz https://github.com/rui314/mold/releases/download/v$MOLD_VERSION/mold-$MOLD_VERSION-x86_64-linux.tar.gz
    if [ "$(md5sum mold-$MOLD_VERSION-x86_64-linux.tar.gz | awk '{print $1}')" = "$MOLD_MD5" ]; then
        tar -xvf mold-$MOLD_VERSION-x86_64-linux.tar.gz
        rm -f mold-$MOLD_VERSION-x86_64-linux.tar.gz
    else
        echo "mold-$MOLD_VERSION-x86_64-linux.tar.gz has been modified"
    fi
else
    echo "already downloaded mold-$MOLD_VERSION-x86_64-linux"
fi

rm -f /home/taxer/.cargo/config.toml && mkdir -p /home/taxer/.cargo && touch /home/taxer/.cargo/config.toml
printf "[target.x86_64-unknown-linux-gnu]\nlinker = \"clang\"\nrustflags = [\"-C\", \"link-arg=-fuse-ld=/workspaces/taxer/.devcontainer/mold-$MOLD_VERSION-x86_64-linux/bin/mold\"]\n" > /home/taxer/.cargo/config.toml
echo "cargo config created"