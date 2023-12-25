default: run-wasm

# Recipe for building Cargo project for WebAssembly (wasm)
build-wasm:
    cargo build --target wasm32-unknown-unknown

# Recipe for running Cargo project on WebAssembly (wasm)
run-wasm: fmt clippy
    @if ! rustup target list | grep wasm32-unknown-unknown >/dev/null; then \
        echo "wasm32-unknown-unknown target not installed. Run 'just install-wasm' to install it."; \
        exit 1; \
    fi
    @if ! command -v wasm-server-runner >/dev/null; then \
        echo "wasm-server-runner not installed. Please install it and make sure it's in your PATH."; \
        exit 1; \
    fi
    cargo run --target wasm32-unknown-unknown

# Recipe for building Cargo project for Linux native
build-linux:
    cargo build --target x86_64-unknown-linux-gnu

# Recipe for running Cargo project on Linux native
run-linux: fmt clippy
    cargo run --target x86_64-unknown-linux-gnu

# Recipe for running clippy on the Cargo project
clippy:
    cargo clippy

# Recipe for formatting the Cargo project
fmt:
    cargo fmt

# Recipe for installing wasm target
setup-wasm:
    rustup target add wasm32-unknown-unknown
    cargo install wasm-server-runner
    @if ! grep -q 'target.wasm32-unknown-unknown' ~/.cargo/config.toml || ! grep -q 'runner = "wasm-server-runner"' ~/.cargo/config.toml; then \
        echo '[target.wasm32-unknown-unknown]' >> ~/.cargo/config.toml; \
        echo 'runner = "wasm-server-runner"' >> ~/.cargo/config.toml; \
    fi

# Recipe for displaying help information
help:
    @echo "Available recipes:"
    @echo "  build-wasm   - Build Cargo project for WebAssembly (wasm)"
    @echo "  run-wasm     - Run Cargo project on WebAssembly (wasm)"
    @echo "  build-linux  - Build Cargo project for Linux native"
    @echo "  run-linux    - Run Cargo project on Linux native"
    @echo "  clippy       - Run clippy on the Cargo project"
    @echo "  fmt          - Format the Cargo project"
    @echo "  setup-wasm   - Install wasm target and wasm-server-runner"
