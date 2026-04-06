# Build stage for the Rust backend (mirror.gcr.io reduces Docker Hub flakes on CI runners)
FROM mirror.gcr.io/library/rust:1.93.1-slim-bookworm AS builder

WORKDIR /usr/src/app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy dependency manifests first for layer caching
COPY Cargo.toml Cargo.lock ./

# Create dummy lib and main to establish dependency graph
# This allows cargo to fetch and cache dependencies before copying source
RUN mkdir -p src/bin && \
    echo "" > src/lib.rs && \
    echo "fn main() {}" > src/main.rs && \
    for f in chat_capability_eval migrate_user_highlights purge_bad_transcripts set_earliest_sync_dates; do \
        echo "fn main() {}" > "src/bin/$f.rs"; \
    done && \
    cargo fetch --locked

# Remove dummy source, copy actual source
RUN rm -rf src
COPY src ./src

# Build application (only recompiles changed source)
RUN cargo build --release

# Runtime stage
FROM mirror.gcr.io/library/node:22.20.0-bookworm-slim

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
