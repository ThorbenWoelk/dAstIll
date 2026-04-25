import { expect, test } from "@playwright/test";

import { PLAYWRIGHT_MAINTENANCE_MODE } from "./runtime-mode";
import { openFreshGuestPage, resetClientState } from "./test-helpers";

test.skip(
  !PLAYWRIGHT_MAINTENANCE_MODE,
  "Maintenance home assertions only apply when the repo runtime mode is maintenance.",
);

test("home route shows the maintenance page for guests", async ({ page }) => {
  await openFreshGuestPage(page, "/");

  await expect(
    page.getByRole("heading", { name: "Sorry, we hit the budget cap :(" }),
  ).toBeVisible();
  await expect(
    page.getByRole("link", {
      name: "Sign in and continue to dastill-mini",
    }),
  ).toHaveAttribute("href", "/login?redirectTo=%2Fmini");
  await expect(
    page.getByRole("link", { name: "Browse the docs to find out more" }),
  ).toBeVisible();
  await expect(page.locator("#workspace")).toHaveCount(0);
});

test("authenticated visitors get the direct mini reader link", async ({
  page,
}) => {
  await resetClientState(page);
  await page.addInitScript(() => {
    window.localStorage.setItem(
      "__dastill_e2e_auth",
      JSON.stringify({
        userId: "maintenance-e2e-user",
        email: "maintenance-e2e@example.com",
        token: "maintenance-e2e-token",
      }),
    );
  });

  await page.goto("/");

  await expect(
    page.getByRole("link", { name: "Continue to dastill-mini" }),
  ).toHaveAttribute("href", "/mini");
});
