/**
 * Browser E2E. Specs mock the backend with `page.route`, so only the frontend
 * dev server is needed. It is started automatically unless one is already
 * running on the base URL (for example from `./start_app.sh`).
 */
import { defineConfig, devices } from "@playwright/test";

const baseURL =
  process.env.PLAYWRIGHT_BASE_URL?.replace(/\/$/, "") ??
  "http://127.0.0.1:3543";
const chromiumPath = process.env.PLAYWRIGHT_CHROMIUM_PATH;

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: true,
  forbidOnly: Boolean(process.env.CI),
  retries: process.env.CI ? 2 : 0,
  reporter: [["list"]],
  timeout: 60_000,
  expect: { timeout: 10_000 },
  use: {
    baseURL,
    trace: "on-first-retry",
    ...devices["Desktop Chrome"],
    viewport: { width: 1280, height: 900 },
    ...(chromiumPath
      ? { launchOptions: { executablePath: chromiumPath } }
      : {}),
  },
  webServer: {
    command: "bun run dev -- --host 127.0.0.1 --port 3543 --strictPort",
    url: baseURL,
    reuseExistingServer: true,
    timeout: 120_000,
  },
});
