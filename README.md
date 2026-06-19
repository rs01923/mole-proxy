# Mole Proxy

A Mullvad to Minecraft proxy interface.

<img width="915" height="637" alt="image" src="https://github.com/user-attachments/assets/beafc3c4-c6e8-49ee-8e76-cf750328f58d" />

> **Note:** This works on both Linux and Windows.

## Prerequisites

Make sure you have the following installed on your system:
* **Docker and Docker Compose** (e.g., Docker Desktop on Windows)
* **Cargo** (install via [rustup.rs](https://rustup.rs/))
* A **Mullvad account** with at least 1 available user slot

## Setup & Usage

1. Open a terminal in the root of the project.
2. Run the startup script for your platform:

### On Linux or Git Bash / MSYS:
```bash
chmod +x ./start.sh
./start.sh
```

### On Windows (PowerShell):
```powershell
Set-ExecutionPolicy -Scope Process -ExecutionPolicy Bypass
.\start.ps1
```
