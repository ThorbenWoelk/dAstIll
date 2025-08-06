FROM python:3.11-slim

WORKDIR /app

# Install system dependencies
RUN apt-get update && apt-get install -y \
    git \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install Node.js and npm (needed for Claude CLI to run)
RUN curl -fsSL https://deb.nodesource.com/setup_18.x | bash - \
    && apt-get install -y nodejs

# Install Claude Code CLI globally in container
# Auth will be mounted from host via volumes
RUN npm install -g @anthropic-ai/claude-code

# Install uv package manager
RUN curl -LsSf https://astral.sh/uv/install.sh | sh

# Add uv to PATH
ENV PATH="/root/.local/bin:$PATH"

# Verify uv installation
RUN uv --version

# Copy dependency files and README (needed for build)
COPY pyproject.toml uv.lock README.md ./

# Install dependencies
RUN uv sync --frozen

# Copy application code
COPY . .

# Create necessary directories
RUN mkdir -p /data/to_be_downloaded /data/downloaded /data/unknown /data/config

# Set environment variables
ENV DASTILL_BASE_PATH=/data
ENV DASTILL_CONFIG_DIR=/data/config
ENV PYTHONPATH=/app
# Ensure Python output is unbuffered for real-time logs
ENV PYTHONUNBUFFERED=1

# Default command starts monitoring service
CMD ["uv", "run", "python", "main.py"]