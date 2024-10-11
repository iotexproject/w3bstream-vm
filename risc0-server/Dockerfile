# Use the official Rust image as the base image
FROM rust:1.79.0 AS builder

# Set the working directory inside the container
WORKDIR /usr/src/app

# Copy the Cargo.toml and Cargo.lock files
COPY Cargo.toml Cargo.lock ./

# Copy the source code
COPY src ./src

# Build the project
RUN cargo build --release

# Use a minimal base image for the final stage
FROM debian:bullseye-slim


# Install ca-certificates without cache
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates && apt-get clean && rm -rf /var/lib/apt/lists/*


# Copy the compiled binary from the builder stage
COPY --from=builder /usr/src/app/target/release/risc0-server /usr/local/bin/risc0-server

# Expose the port that the server will run on
EXPOSE 4001

# Set the entrypoint to the compiled binary
CMD ["risc0-server"]
