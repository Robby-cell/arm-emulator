# ARM Emulator

The goal of the project is to create a user-friendly, modern GUI tool that allows students to visualize how a CPU works, alongside peripherals (i.e. LEDs).

Existing projects do not currently cover the requirements that this project aims to solve.

# Architecture

The project has two main components:

- [Emulator](docs/architecture/Emulator.md)

- [GUI](docs/architecture/GUI.md)

# Building and Using

To build the python bindings:

Tooling requirements:
- cargo (can be installed [here](https://rustup.rs/))
- python ([uv](https://docs.astral.sh/uv/getting-started/installation/) package/project manager is nice, but not necessary)

No other requirements. Cargo and python are all that are required.

Initialize a virtual environment:

```bash
uv venv
# python3 -m venv .venv && source .venv/bin/activate
```

Download maturin:

```bash
uv pip install maturin
# python3 -m pip install maturin
# cargo install maturin
```

Build the python bindings and install it to the environment:

```bash
uv run maturin develop
# python3 -m maturin develop
# For release:
# uv run maturin develop --release
```

If rebuilding, the cache can be cleared, so it won't use the old build:

```bash
uv cache clean
# python3 -m cache purge
```

Run a python script that is using the bindings, using the environment that has them installed:

```bash
uv run sample-usage.py
# python3 sample-usage.py
```
