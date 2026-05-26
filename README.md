# Mole Proxy

A Mullvad to Minecraft proxy interface.

<img width="915" height="637" alt="image" src="https://github.com/user-attachments/assets/beafc3c4-c6e8-49ee-8e76-cf750328f58d" />

> **Note:** This currently only works on Linux. If you want to try porting it to Windows, have fun - PRs are open!

## Prerequisites

Make sure you have the following installed on your system:
* **Docker and Docker Compose**
* **Cargo** (install via [rustup.rs](https://rustup.rs/))
* A **Mullvad account** with at least 1 available user slot

## Setup & Usage

1. Open a terminal in the root of the project.
2. Make the script executable and run it:

```bash
chmod +x ./start
./start
