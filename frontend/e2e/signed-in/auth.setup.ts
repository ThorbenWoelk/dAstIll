import { test as setup } from "@playwright/test";
import { mkdir } from "node:fs/promises";

const storageStatePath = "playwright/.auth/user.json";

setup("prepare signed-in storage state scaffold", async ({ page }) => {
  await mkdir("playwright/.auth", { recursive: true });
  await page.context().storageState({ path: storageStatePath });
});
