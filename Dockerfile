FROM rust:latest

# Install required development tools and libraries
RUN apt-get update && apt-get install -y \
    libsqlite3-dev \        # For SQLite support in Rust
    wkhtmltopdf \           # For HTML to PDF export (optional, for reports)
    curl \                  # For network tests (optional)
    git \                   # For pulling dependencies if needed
    pkg-config \
    build-essential \
    && rm -rf /var/lib/apt/lists/* \
    && rustup component add rustfmt

# Create a new user to avoid running as root
RUN useradd -ms /bin/bash rustdev
USER rustdev
WORKDIR /home/rustdev/app

# Default command
CMD ["bash"]