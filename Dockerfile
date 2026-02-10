# ---------- Build stage ----------
FROM rust:latest as builder

# Set working directory
WORKDIR /app

# Copy Cargo files first (for caching)
COPY Cargo.toml Cargo.lock ./

# Create dummy src to cache dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies only
RUN cargo build --release
RUN rm -rf src

# Copy actual source code
COPY src ./src

# Build the real app
RUN cargo build --release


# ---------- Runtime stage ----------
FROM debian:bookworm-slim

# Install required system libs (important for TLS, Postgres, etc.)
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user (best practice)
RUN useradd -m appuser

WORKDIR /app

# Copy compiled binary from builder
COPY --from=builder /app/target/release/blog-backend /app/app

# Change ownership
RUN chown -R appuser:appuser /app

USER appuser

# Expose your backend port
EXPOSE 8000

# Run the app
CMD ["sh", "-c", "echo STARTING && ./app || echo APP_CRASHED"]
