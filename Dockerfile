# Start from the latest official Rust image to easily compile the project
FROM rust:alpine as builder

RUN apk update

RUN apk add binutils build-base g++ gcc libressl-dev

# Set the working directory
WORKDIR /glint

# Copy the current directory content into the container
COPY . .

# Build the project with release profile to optimize for runtime performance
RUN cargo build --release

# Create a smaller container for runtime, using Alpine Linux
FROM alpine as runtime

# Set the working directory for Glint files
WORKDIR /glint

# Copy the built Glint binary from the builder stage
COPY --from=builder /glint/target/release/glint /usr/local/bin

# Allow users to provide a configuration directory to mount
VOLUME /glint

# Define entrypoint to run Glint
ENTRYPOINT ["/usr/local/bin/glint"]

# Default command to show help if no arguments are provided
CMD ["--help"]
