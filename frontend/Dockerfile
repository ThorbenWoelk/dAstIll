# Build stage
FROM node:20-slim AS builder

WORKDIR /app

# Install build dependencies
# (We need these for some native packages if any, and for the build process)
RUN apt-get update && apt-get install -y \
    python3 \
    make \
    g++ \
    && rm -rf /var/lib/apt/lists/*

# Install bun just for the build if we want to stay consistent, 
# or just use npm/pnpm. Let's use npm for maximum compatibility in Docker.
RUN npm install -g bun

# Copy configuration files
COPY package.json bun.lock ./

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
FROM node:20-slim

WORKDIR /app
ENV NODE_ENV=production

# Copy build output
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json

# Instead of copying node_modules from builder (which might have dev deps 
# or be incompatible), we install only production deps here.
RUN npm install --omit=dev

# Expose the frontend port
EXPOSE 3000

# Start SvelteKit app
CMD ["node", "build/index.js"]
