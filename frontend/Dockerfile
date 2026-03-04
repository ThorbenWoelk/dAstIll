# Build stage
FROM oven/bun:1 AS builder

WORKDIR /app

# Build arguments
ARG VITE_API_BASE
ENV VITE_API_BASE=$VITE_API_BASE

# Install dependencies
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Copy source
COPY . .

# Run sync and build
RUN bun x svelte-kit sync
RUN VITE_API_BASE=$VITE_API_BASE bun run build

# Install only production dependencies to a separate directory
# This ensures we have exactly what we need for runtime
RUN mkdir /temp/prod_deps
COPY package.json bun.lock /temp/prod_deps/
RUN cd /temp/prod_deps && bun install --frozen-lockfile --production

# Runtime stage
FROM node:20-slim

WORKDIR /app
ENV NODE_ENV=production

# Copy build output
COPY --from=builder /app/build ./build
COPY --from=builder /app/package.json ./package.json
# Copy production dependencies
COPY --from=builder /temp/prod_deps/node_modules ./node_modules

# Expose the frontend port
EXPOSE 3000

# Start SvelteKit app
CMD ["node", "build/index.js"]
