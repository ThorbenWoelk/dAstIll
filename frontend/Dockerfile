# Build stage
FROM oven/bun:1 AS builder

WORKDIR /app

# Build arguments
ARG VITE_API_BASE
ENV VITE_API_BASE=$VITE_API_BASE

# Install dependencies
COPY package.json bun.lock ./
RUN bun install --frozen-lockfile

# Copy source and build
COPY . .
RUN bun run build

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
