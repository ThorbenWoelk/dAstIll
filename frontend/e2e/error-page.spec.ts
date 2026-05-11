import { expect, test } from "@playwright/test";

import { openFreshGuestPage } from "./test-helpers";

test("root error page gives clear recovery actions", async ({ page }) => {
  await openFreshGuestPage(page, "/missing-error-page-fixture");

  await expect(
    page.getByRole("heading", { name: "This page is out of reach." }),
  ).toBeVisible();
  await expect(page.getByRole("link", { name: "Go home" })).toHaveAttribute(
    "href",
    "/",
  );
  await expect(page.getByRole("button", { name: "Reload" })).toBeVisible();
  await expect(
    page.getByRole("link", { name: /contact support/i }),
  ).toBeVisible();
});
