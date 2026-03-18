# ARM Emulator

The goal of the project is to create a user-friendly, modern GUI tool that allows students to visualize how a CPU works, alongside peripherals (i.e. LEDs).

Existing projects do not currently cover the requirements that this project aims to solve.

# Architecture

The project has two main components:

- [Emulator](docs/architecture/Emulator.md)

- [GUI](docs/architecture/GUI.md)

# Installation

Download the latest release from the [releases page](https://github.com/Robby-cell/arm-emulator/releases).

Extract the archive. The executable can be run as-is.
However, on MacOS, quarantine flags need to be stripped first,
as it will be built without paying Apple for an Apple Developer Certificate:

```bash
xattr -cr ./arm_emulator-macos/arm_emulator
```

Download a specific release from the tag:
https://github.com/Robby-cell/arm-emulator/releases/latest/

## Linux

How to download on Linux:

```bash
TAG="v0.2.4"
wget "https://github.com/Robby-cell/arm-emulator/releases/download/${TAG}/arm_emulator-linux.tar.gz"
tar -xzf "./arm_emulator-linux.tar.gz"
# ./arm_emulator-linux/arm_emulator
```

## Windows

How to download on Windows:

```ps1
$TAG="v0.2.4"
# Important: MUST be curl.exe, not curl
curl.exe -L -o ".\arm_emulator-windows.zip" "https://github.com/Robby-cell/arm-emulator/releases/download/$TAG/arm_emulator-windows.zip"
Expand-Archive ".\arm_emulator-windows.zip" -DestinationPath ".\arm_emulator-windows"
# .\arm_emulator-windows\arm_emulator.exe
```

## MacOS

How to download on MacOS:

```bash
TAG="v0.2.4"
curl -L -o "./arm_emulator-macos.tar.gz" "https://github.com/Robby-cell/arm-emulator/releases/download/${TAG}/arm_emulator-macos.tar.gz"
tar -xzf "./arm_emulator-macos.tar.gz"
xattr -cr "./arm_emulator-macos/arm_emulator"
# ./arm_emulator-macos/arm_emulator
```

# Building and Using

To build on Windows, you must use Git Bash.

To build the python bindings:

Tooling requirements:
- cargo (can be installed [here](https://rustup.rs/))
- python ([uv](https://docs.astral.sh/uv/getting-started/installation/) package/project manager is nice, but not necessary)

No other requirements. Cargo and python are all that are required.

Initialize a virtual environment:

```bash
uv venv
# python3 -m venv .venv

# Activate environment
source .venv/bin/activate

# On Windows:
# .\.venv\Scripts\activate
```

Download maturin:

```bash
uv pip install maturin
# python3 -m pip install maturin
# cargo install maturin
```

Build the python bindings and install it to the environment:

```bash
# Debug:
# uvx maturin develop
# python3 -m maturin develop
# For release:
uvx maturin develop --release
# python3 -m maturin develop --release
```

If rebuilding, the cache can be cleared, so it won't use the old build:

```bash
uv cache clean
# python3 -m cache purge
```

Run a python script that is using the bindings, using the environment that has them installed:

```bash
uv run gui_main.py
# python3 gui_main.py
```

# Run the Tests

To run the Rust tests:

```bash
cargo test --all
```

# Packaging the GUI

To build and package the python GUI:

```bash
# Build the project first. And then:
./package.sh

# On Windows, in Git Bash:
# ./package-windows.sh
```
