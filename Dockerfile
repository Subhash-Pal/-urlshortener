# Use the official slim Rust image for a smaller base
FROM rust:1.70.0-slim-bullseye

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and your project source code
COPY Cargo.toml ./
COPY . .

# Install dependencies
RUN apt-get update && apt-get install -y pkg-config libssl-dev

# Compile the project in release mode
RUN cargo build --release

# Set the entry point command to run your application
CMD ["target/release/urlshortner"]
