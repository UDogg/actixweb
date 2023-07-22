# Use a Rust image as the base image
FROM rust:latest as builder

# Set the working directory inside the container
WORKDIR /app

# Copy the Cargo.toml and Cargo.lock files to the container
COPY Cargo.toml Cargo.lock ./

# Build the dependencies as a separate layer to optimize caching
RUN cargo fetch

# Copy the rest of the source code to the container
COPY src ./src

# Build the Rust project with optimizations
RUN cargo build --release

# Create a new lightweight image to run the application
FROM debian:buster-slim

# Set the working directory inside the container
WORKDIR /app

# Copy the built binary from the builder stage
COPY --from=builder /app/target/release/your_app_name .

# Expose the port on which your Actix web server listens
EXPOSE 8080

# Set the command to run your application when the container starts
CMD ["./actixweb"]
