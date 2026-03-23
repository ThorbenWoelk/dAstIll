/**
 * Browser E2E only: Playwright hits `baseURL` (the Svelte app). It does not read
 * DB credentials. Data comes from whatever database the running backend is
 * configured to use (e.g. `backend/.env`). Start backend + frontend (e.g.
 * `./start_app.sh`) before `bun run test:e2e`.
 */
import { defineConfig, devices } from "@playwright/test";

const baseURL =
  process.env.PLAYWRIGHT_BASE_URL?.replace(/\/$/, "") ??
  "http://127.0.0.1:3543";

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 2 : 0,
  workers: process.env.CI ? 2 : undefined,
  reporter: [["list"], ["html", { open: "never" }]],
  timeout: 120_000,
  expect: { timeout: 30_000 },
  use: {
    baseURL,
    trace: "on-first-retry",
    video: "on-first-retry",
    ...devices["Desktop Chrome"],
    viewport: { width: 1280, height: 900 },
  },
  projects: [{ name: "chromium", use: { ...devices["Desktop Chrome"] } }],
});
