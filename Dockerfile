FROM rust:latest

# Install common development tools
RUN apt-get update && apt-get install -y \
    mpv \
    ffmpeg \
    pkg-config \
    libssl-dev \
    build-essential \
    libasound2-dev \
    && rm -rf /var/lib/apt/lists/* \
    rustup component add rustfmt

# Create a new user to avoid running as root
RUN useradd -ms /bin/bash rustdev
USER rustdev
WORKDIR /home/rustdev/app

# Default command
CMD ["bash"]