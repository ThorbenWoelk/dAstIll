# Build stage
FROM oven/bun:1 AS builder

WORKDIR /app

# Build arguments
ARG VITE_API_BASE
ENV VITE_API_BASE=$VITE_API_BASE

# Install dependencies (all, including devDeps for build)
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Copy source
COPY . .

# Run sync and build
RUN bun x svelte-kit sync
RUN VITE_API_BASE=$VITE_API_BASE bun run build

# Remove dev dependencies to keep only production ones for copying
# Bun doesn't have a direct equivalent to 'npm prune --production' that works easily here,
# so we'll just use the builder's node_modules. SvelteKit adapter-node usually bundles 
# most things, but 'isomorphic-dompurify' seems to be an external dependency.

# Runtime stage
FROM node:20-slim

WORKDIR /app
ENV NODE_ENV=production

# Copy build output
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json
# Copy all node_modules just to be safe, since prune is tricky with bun in multi-stage
COPY --from=builder /app/node_modules ./node_modules

# Expose the frontend port
EXPOSE 3000

# Start SvelteKit app
CMD ["node", "build/index.js"]
