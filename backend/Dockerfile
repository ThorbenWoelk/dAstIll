# Build stage for the Rust backend
FROM rust:1.93.1-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

COPY . .

# Build the application
RUN cargo build --release

# Runtime stage
FROM node:22.20.0-bookworm-slim

WORKDIR /usr/local/bin

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    openssl \
    ca-certificates \
    curl \
    python3 \
    ffmpeg \
    && rm -rf /var/lib/apt/lists/*

# Install summarize CLI and media helpers used for transcript extraction
RUN npm install -g @steipete/summarize@0.12.0 && \
    npm cache clean --force

# Install yt-dlp as a dependency for transcript extraction fallbacks
RUN curl -L https://github.com/yt-dlp/yt-dlp/releases/download/2026.03.17/yt-dlp -o /usr/local/bin/yt-dlp && \
    chmod a+rx /usr/local/bin/yt-dlp

# Copy the backend binary
COPY --from=builder /usr/src/app/target/release/dastill .

# Expose the backend port
EXPOSE 3001

# Set the startup command
CMD ["./dastill"]
