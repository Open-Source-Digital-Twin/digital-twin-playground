# Linux

1. Install Rust by following the official [installation guide](https://www.rust-lang.org/tools/install).
2. Clone this repository.
3. Run `git submodule update --init` to update the submodules (which contains 3D models).
4. Install [just](https://github.com/casey/just) to facilitate command handling (e.g., `cargo install just`).
5. (Optional) Run `just setup-wasm` to setup your environment for building the WebAssembly version of the project.
6. Run `just run-linux` or `just run-wasm` to start the project.