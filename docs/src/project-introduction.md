# Digital Twin Playground

This  is a project that provides a virtual environment for testing various motion control algorithms. It leverages the Bevy game engine and is implemented in the Rust programming language. 

> 🚧  This project is in alpha stage and there is still work in progress.

The idea is to create a digital twin of a physical system, such as a robot or a drone, and test different control algorithms in a virtual environment. This can be useful for testing and validating control algorithms before deploying them to a physical system.

In the current version, the project provides a furuta pendulum system which is "embedded" in the code. The pendulum can be controlled using the keyboard. 

The next steps are to import models from third-party software, such as Blender and control them by creating your own controller. The project will also support different types of joints and motors, as well as more complex interactions between joints. Which can be configured by the user and selecting the interaction between the joints and the motors in the user interface.

The user will be able to control the joints and motors in the scene, by creating a controller that can be attached to the joints and motors. The controller will be able to control the speed and precision of the joints and motors, as well as other settings. And the implmentation of this controller can be done in any programming language that can communicate with the project by Protocol Buffers.

The user will be able to interact with the scene using the keyboard or mouse. The user will also be able to control the camera position and projection, as well as other settings in the scene.
