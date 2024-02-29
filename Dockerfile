# Use a Rust-based Docker image suitable for cross-compiling to Linux
FROM ekidd/rust-musl-builder AS builder

# Set the working directory inside the container
WORKDIR /usr/src/urlshortner

# Copy the project files into the container
COPY . .

# Build the Rust application
RUN cargo build --release

# Start a new stage without the build environment
FROM alpine:latest

# Set the working directory inside the container
WORKDIR /usr/src/urlshortner

# Copy the compiled binary from the previous stage
COPY --from=builder /usr/src/urlshortner/target/x86_64-unknown-linux-musl/release/urlshortner .

# Expose the port that the application listens on
EXPOSE 8080

# Run the application
CMD ["./urlshortner"]
