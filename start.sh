#!/bin/bash

if [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "cygwin"* || "$OSTYPE" == "win32"* ]]; then
    echo "Windows support coming soon."
    exit 1
elif [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "Error: This script strictly requires Linux."
    exit 1
fi

if ! command -v cargo &> /dev/null; then
    echo "Error: 'cargo' could not be found."
    echo "Please install Rust and cargo (e.g., via rustup) before running this script."
    exit 1
fi

echo "Building GUI..."
cd mole-proxy-gui || exit 
cargo build --release
cp target/release/mole-proxy-gui ../gui
cd ../

if [ ! -d "mullvad-config" ]; then
    echo "Configuration folder not found. Running the initial build..."
    docker-compose up -d --build
else
    echo "Configuration found! Skipping the build and starting the proxy..."
    docker-compose up -d
fi

echo "All setup! Use the Mole Proxy GUI to interface with the setup."
echo "To stop the container use \"docker compose down\""
chmod +x ./gui
./gui
