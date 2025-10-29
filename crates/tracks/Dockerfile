# Build stage
FROM rust:1.90 as builder

WORKDIR /app

# Copy cargo files
COPY Cargo.toml Cargo.lock ./

# Create src directory and dummy main.rs to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this will be cached)
RUN cargo build --release --bin tracks

# Remove dummy main.rs and copy actual source
RUN rm -rf src

# Copy source code
COPY src ./src
COPY migrations ./migrations
COPY .sqlx ./sqlx

# Build the actual application
RUN touch src/main.rs && cargo build --release --bin tracks

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
