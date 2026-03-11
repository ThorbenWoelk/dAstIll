# Shared base with Bun available for deterministic installs.
FROM node:20-slim AS base

WORKDIR /app

# Install build tooling and Bun.
RUN apt-get update && apt-get install -y \
    python3 \
    make \
    g++ \
    && rm -rf /var/lib/apt/lists/*
RUN npm install -g bun

FROM base AS prod-deps

COPY package.json bun.lock .npmrc ./
RUN bun install --frozen-lockfile --production --ignore-scripts

# Build stage
FROM base AS builder

# Copy configuration files
COPY package.json bun.lock .npmrc ./

# Install dependencies
RUN bun install --frozen-lockfile

# Copy source
COPY . .

# Run build
ARG VITE_API_BASE
ENV VITE_API_BASE=$VITE_API_BASE
RUN bun x svelte-kit sync
RUN VITE_API_BASE=$VITE_API_BASE bun run build

# Runtime stage
FROM node:20-slim AS runtime

WORKDIR /app
ENV NODE_ENV=production

# Copy build output
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json
COPY --from=prod-deps /app/node_modules ./node_modules

# Expose the frontend port
EXPOSE 3000

# Start SvelteKit app
CMD ["node", "build/index.js"]
