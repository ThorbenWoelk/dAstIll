import { devices, expect, test } from "@playwright/test";

import { resetClientState } from "./test-helpers";

test.use({ ...devices["iPhone 13"] });

test.beforeEach(async ({ page }) => {
  await resetClientState(page);
});

test("mobile tabs stay scrollable and shortcut hints stay hidden", async ({
  page,
}) => {
  await page.goto("/");

  await page.evaluate(() => {
    window.dispatchEvent(
      new KeyboardEvent("keydown", {
        key: "Meta",
        metaKey: true,
        bubbles: true,
      }),
    );
  });

  await expect(
    page.locator('[aria-label="Shortcut hints: press Cmd and a number"]'),
  ).toHaveCount(0);

  const browseRegion = page.getByRole("region", { name: "Browse" });
  if ((await browseRegion.count()) === 0) {
    const backButton = page.getByRole("button", { name: "Back" });
    if (await backButton.count()) {
      await backButton.click();
    } else {
      await page
        .getByRole("banner")
        .getByRole("link", { name: "Go to dAstIll home" })
        .click();
    }
  }

  await expect(browseRegion).toBeVisible({ timeout: 15_000 });

  const sourcesPane = browseRegion.getByRole("complementary").first();
  const channelButtons = sourcesPane
    .getByRole("button")
    .filter({ hasNotText: "Add source" });

  await page.waitForTimeout(1_500);

  test.skip(
    (await channelButtons.count()) === 0,
    "Mobile browse view has no channels; run against a seeded backend",
  );

  await channelButtons.first().click();

  const videoList = browseRegion.getByRole("complementary").first();
  const videoButtons = videoList
    .getByRole("button")
    .filter({ hasNotText: "Adjust sync date" });

  await page.waitForTimeout(1_500);

  test.skip(
    (await videoButtons.count()) === 0,
    "Mobile browse view has no videos; run against a seeded backend",
  );

  await videoButtons.first().click();

  const mobileTabs = page.locator("[data-mobile-content-tabs]");
  const mobileTabsCue = page.locator("[data-mobile-content-tabs-cue]");

  await expect(mobileTabs).toBeVisible();
  await expect(mobileTabsCue).toBeVisible();

  const tabMetrics = await mobileTabs.evaluate((node) => ({
    clientWidth: node.clientWidth,
    scrollWidth: node.scrollWidth,
  }));
  expect(tabMetrics.scrollWidth).toBeGreaterThan(tabMetrics.clientWidth);

  const tabsOverlap = await page
    .locator("#workspace-tabs-mobile [data-workspace-content-tab]")
    .evaluateAll((nodes) => {
      if (nodes.length < 2) return false;
      const first = nodes[0].getBoundingClientRect();
      const second = nodes[1].getBoundingClientRect();
      return second.left < first.right;
    });
  expect(tabsOverlap).toBe(true);
});
