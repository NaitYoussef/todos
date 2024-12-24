ARG FDB_VERSION=7.3.43
ARG RUST_VERSION=1.83.0
# Build Stage
FROM rust:${RUST_VERSION}-bullseye as builder

ARG FDB_VERSION

RUN apt-get update; apt-get install -y --no-install-recommends libclang-dev

WORKDIR /app

# Copy only the necessary files for dependency resolution
RUN mkdir s3
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

# Copy the rest of the source code
# COPY src ./src

COPY src ./src
# Build the Rust project
RUN cargo build --release

RUN ls -l target/release
# Final Stage
FROM debian:bullseye
ARG FDB_VERSION

RUN apt update && apt install -y wget curl dnsutils libpq-dev

WORKDIR /tmp

WORKDIR /app

# Copy the built artifact from the build stage
COPY --from=builder /app/target/release/todos .
# ADD .github/docker/run.sh /app/docker_entrypoint.sh

EXPOSE 3000 
EXPOSE 4000

# Set the command to run on container start
ENTRYPOINT ["./todos"]
