# Digital twin playground

This project is a virtual environment for testing different motion control algorithms. It uses the Bevy game engine and is written in Rust.

> :warning: This project is in alpha stage and there is still work in progress.

[pendulum.webm](assets/pendulum.webm)

## Features
* Import 3D models from Blender
* Control the motion of the models using different controllers, such as PID, fuzzy, and adaptive control (or any other creative strategies)
* Visualize the results of the control algorithms

## Getting Started
1. Install Rust.
2. Clone this repository.
3. Run `git submodule update --init`
4. Install [just](https://github.com/casey/just) to facilitate command handling (e.g., `cargo install just`).
5. (Optional) Run `just setup-wasm` to setup your environment for building the WebAssembly version of the project.
6. Run `just run-linux` or `just run-wasm` to start the project.

## Controls
* Left mouse button - rotate camera
* Right mouse button - pan camera
* Mouse wheel - zoom camera
* B - enable/disable frames for elements
* L - start/stop animation
* U - enable/disable shadows

## Documentation
The documentation for the project can be found in the docs directory.

## Contributing
Contributions to this project are welcome! Please open an issue or a pull request if you have any ideas or improvements.

## License
This project is licensed under the MIT License.
