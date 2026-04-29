# dAstIll Docs Frontend

This folder contains the VitePress documentation site.

## Start Locally

From this folder:

```bash
bun install --frozen-lockfile
bun run dev
```

Default URL:

```text
http://localhost:4173
```

## Build

```bash
bun run build
```

The static output is written to:

```text
docs/.vitepress/dist
```

## Preview The Build

```bash
bun run preview
```

## From The Repo Root

The canonical full-stack local entrypoint starts the product frontend, backend, and docs together:

```bash
./start_app.sh
```

Stop everything:

```bash
./end_app.sh
```
