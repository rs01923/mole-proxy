#!/bin/bash

EXE_SUFFIX=""
if [[ "$OSTYPE" == "msys"* || "$OSTYPE" == "cygwin"* || "$OSTYPE" == "win32"* ]]; then
    EXE_SUFFIX=".exe"
    if [[ -d "C:/Program Files/Docker/Docker/resources/bin" ]]; then
        export PATH="C:/Program Files/Docker/Docker/resources/bin:$PATH"
    fi
elif [[ "$OSTYPE" != "linux-gnu"* ]]; then
    echo "Error: This script strictly requires Linux or Windows."
    exit 1
fi

if ! command -v docker &> /dev/null && ! command -v docker-compose &> /dev/null; then
    echo "Error: 'docker' or 'docker-compose' could not be found."
    echo "Please install Docker Desktop and ensure it is running."
    exit 1
fi

# Check if Docker daemon is running
if ! docker ps &> /dev/null; then
    echo "Error: Docker daemon is not running."
    echo "Please open Docker Desktop and make sure the engine is fully started before running this script."
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
cp target/release/mole-proxy-gui${EXE_SUFFIX} ../gui${EXE_SUFFIX}
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
chmod +x ./gui${EXE_SUFFIX}
./gui${EXE_SUFFIX}
