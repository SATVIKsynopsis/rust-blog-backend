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

# Install only the necessary runtime libraries
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Create non-root user
RUN useradd -m appuser
WORKDIR /app

# Copy the binary
COPY --from=builder /app/target/release/blog-backend /app/app

# Set permissions
RUN chown -R appuser:appuser /app
USER appuser

# Render uses the PORT env var; EXPOSE is just documentation
EXPOSE 8000

CMD ["./app"]