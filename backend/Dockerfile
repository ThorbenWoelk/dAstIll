# Build stage for the Rust backend
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libsqlite3-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    libsqlite3-0 \
    curl \
    python3 \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Install yt-dlp as a dependency for transcript extraction
RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/latest/download/yt-dlp -o /usr/local/bin/yt-dlp && \
    chmod a+rx /usr/local/bin/yt-dlp

# Copy the backend binary
COPY --from=builder /usr/src/app/target/release/dastill .

# Expose the backend port
EXPOSE 3001

# Set the startup command
CMD ["./dastill"]
