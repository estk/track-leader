# Build stage - multi-arch support for Apple Silicon and x86_64
FROM rust:1.83-bookworm as builder

WORKDIR /app

# Copy workspace manifest and crate manifest
COPY Cargo.toml ./
COPY crates/tracks/Cargo.toml crates/tracks/Cargo.lock ./crates/tracks/

# Create src directory and dummy main.rs to cache dependencies
RUN mkdir -p crates/tracks/src && echo "fn main() {}" > crates/tracks/src/main.rs

# Build dependencies (this will be cached)
RUN cargo build --release -p tracks

# Remove dummy main.rs and copy actual source
RUN rm -rf crates/tracks/src

# Copy source code
COPY crates/tracks/src ./crates/tracks/src
COPY crates/tracks/migrations ./crates/tracks/migrations
COPY crates/tracks/.sqlx ./crates/tracks/.sqlx

# Build the actual application
RUN touch crates/tracks/src/main.rs && cargo build --release -p tracks

# Runtime stage
FROM debian:bookworm-slim

# Install required runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/tracks /usr/local/bin/tracks

# Create uploads directory
RUN mkdir -p /app/uploads

# Set environment variables
ENV RUST_LOG=info
ENV DATABASE_URL=postgres://tracks_user:tracks_password@postgres:5432/tracks_db
ENV OBJECT_STORE_PATH=/app/uploads
ENV PORT=3000

# Expose port
EXPOSE 3000

# Run the application
CMD ["tracks"]
