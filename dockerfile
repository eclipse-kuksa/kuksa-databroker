# Use the latest version of the Rust base image
FROM rust:latest

# Set the working directory in the container to /my
WORKDIR /usr/src/my-app

ENV VSS_FILE "vss_release_4.0.json"

# Copy the Rust project files to the working directory
COPY . .

# Build the Rust app
RUN cargo build

# Set the command to run the Rust app
CMD cargo run --bin databroker --features stats -- --metadata ./data/vss-core/${VSS_FILE} --insecure --enable-databroker-v1
