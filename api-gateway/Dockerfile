# Build stage
FROM rust:1.75-slim as builder

WORKDIR /app
COPY . .

# Install dependencies for building
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Build release binary
RUN cargo build --release -p api-gateway

# Runtime stage
FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary from builder stage
COPY --from=builder /app/target/release/api-gateway /usr/local/bin/api-gateway

# Create non-root user
RUN useradd -r -s /bin/false appuser
USER appuser

EXPOSE 3001

CMD ["api-gateway"]