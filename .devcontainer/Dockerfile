FROM mcr.microsoft.com/devcontainers/rust:latest

# Avoid warnings by switching to noninteractive
ENV DEBIAN_FRONTEND=noninteractive

# Configure apt and install packages
RUN apt update \
    && apt -y install --no-install-recommends \
    g++ \
    pkg-config \
    libx11-dev \
    libasound2-dev \
    libudev-dev \
    libxkbcommon-x11-0 \
    libwayland-dev \
    libxkbcommon-dev 

# Set environment variables
ENV DISPLAY=:0
ENV XAUTHORITY=/tmp/.Xauthority
ENV WAYLAND_DISPLAY=wayland-0

# Copy the Xauthority file
# COPY --chown=root:root /tmp/.Xauthority /tmp/.Xauthority