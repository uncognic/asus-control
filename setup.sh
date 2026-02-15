#!/bin/bash
set -euo pipefail

if ! getent group "asus-control" >/dev/null; then
    echo "Creating group asus-control..."
    sudo groupadd asus-control
fi

if ! id -nG "$USER" | grep -qw "asus-control"; then
    echo "Adding $USER to group asus-control..."
    sudo usermod -aG "asus-control" "$USER"
    echo "Running newgrp asus-control to apply group changes..."
    exec newgrp asus-control
fi

echo "Group setup compplete"