FROM node:20.20.0-slim AS base

WORKDIR /app

RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    git \
    && rm -rf /var/lib/apt/lists/*
RUN npm install -g bun

FROM base AS builder

ARG VITE_API_BASE
ARG PUBLIC_DOCS_URL
ARG PUBLIC_CONTACT_EMAIL
ARG PUBLIC_FIREBASE_API_KEY
ARG PUBLIC_FIREBASE_AUTH_DOMAIN
ARG PUBLIC_FIREBASE_PROJECT_ID

ENV VITE_API_BASE=$VITE_API_BASE
ENV PUBLIC_DOCS_URL=$PUBLIC_DOCS_URL
ENV PUBLIC_CONTACT_EMAIL=$PUBLIC_CONTACT_EMAIL
ENV PUBLIC_FIREBASE_API_KEY=$PUBLIC_FIREBASE_API_KEY
ENV PUBLIC_FIREBASE_AUTH_DOMAIN=$PUBLIC_FIREBASE_AUTH_DOMAIN
ENV PUBLIC_FIREBASE_PROJECT_ID=$PUBLIC_FIREBASE_PROJECT_ID

COPY package.json bun.lock .npmrc ./
RUN bun install --frozen-lockfile

COPY . .
RUN bun x svelte-kit sync
RUN bun run build

FROM nginx:1.28.2-alpine AS runtime

COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY --from=builder /app/build /usr/share/nginx/html

EXPOSE 8080

CMD ["nginx", "-g", "daemon off;"]
