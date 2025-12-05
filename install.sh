#!/bin/sh
# Usage: curl -sL https://raw.githubusercontent.com/USER/how/main/install.sh | bash

REPO="hansbala/how"
VERSION="v0.1.0"

OS=$(uname -s)
ARCH=$(uname -m)

if [ "$OS" = "Darwin" ] && [ "$ARCH" = "arm64" ]; then
    ASSET="how-darwin-arm64"
elif [ "$OS" = "Linux" ] && [ "$ARCH" = "x86_64" ]; then
    ASSET="how-linux-amd64"
else
    echo "Sorry, your platform ($OS $ARCH) is not supported yet."
    exit 1
fi

echo "Downloading $ASSET..."
curl -L -o how "https://github.com/$REPO/releases/download/$VERSION/$ASSET"

chmod +x how
sudo mv how /usr/local/bin/how

echo "Installed! Try: how 'list large files'"

