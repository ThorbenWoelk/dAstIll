# Build stage
FROM oven/bun:1 AS builder

WORKDIR /app

# Build arguments
ARG VITE_API_BASE
# Ensure the variable is available during 'bun run build'
ENV VITE_API_BASE=$VITE_API_BASE

# Install dependencies
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Copy source
COPY . .

# Run sync before build to ensure .svelte-kit directory is populated
RUN bun x svelte-kit sync

# Build with the environment variable explicitly passed just in case
RUN VITE_API_BASE=$VITE_API_BASE bun run build

# Runtime stage
FROM node:20-slim

WORKDIR /app
ENV NODE_ENV=production

# Copy build output
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json

# Expose the frontend port
EXPOSE 3000

# Start SvelteKit app
CMD ["node", "build/index.js"]
