FROM rust:latest AS builder

WORKDIR /usr/src/chimera

# Install build dependencies
RUN apt-get update && apt-get install -y libopus-dev pkg-config

# Copy the project files
COPY . .

# Build the project
RUN cargo build --release

FROM debian:bookworm-slim

WORKDIR /usr/src/chimera

# Install runtime dependencies
RUN apt-get update && apt-get install -y libopus0 openssl ca-certificates && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage
COPY --from=builder /usr/src/chimera/target/release/chimera .

# Copy Lavalink server if you have it locally, or download it
# For this example, we'll assume you have a Lavalink.jar in your project root
# COPY Lavalink.jar .

# Set the entrypoint
CMD ["./chimera"]
