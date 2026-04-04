import { expect, test, devices } from "@playwright/test";

test.use({ ...devices["iPhone 13"] });

test("mobile text selection shows the custom toolbar at the bottom", async ({
  page,
}) => {
  // 1. Navigate to home
  await page.goto("/");

  // 2. Ensure the mobile browse region is visible even if the page restored into
  // a content-first state.
  const browseRegion = page.getByRole("region", { name: "Browse" });
  if ((await browseRegion.count()) === 0) {
    const backButton = page.getByRole("button", { name: "Back" });
    if (await backButton.count()) {
      await backButton.click();
    } else {
      await page.getByLabel("Go to dAstIll home").click();
    }
  }
  await expect(browseRegion).toBeVisible({ timeout: 15000 });

  const videoButtons = browseRegion.locator("aside button");
  const emptyState = browseRegion.getByText("No videos yet.");
  const browseState = async () => {
    if ((await videoButtons.count()) > 0) return "videos";
    if (await emptyState.isVisible()) return "empty";
    return "loading";
  };

  await expect
    .poll(browseState, {
      timeout: 15000,
      message: "Timed out waiting for mobile browse results to settle",
    })
    .not.toBe("loading");

  test.skip(
    (await browseState()) !== "videos",
    "Mobile browse view has no videos; run against a seeded backend",
  );

  const videoButton = videoButtons.first();
  await expect(videoButton).toBeVisible();
  await videoButton.click();

  // 3. Switch to Transcript tab if not already there
  await page.getByRole("button", { name: "Transcript", exact: true }).click();

  const article = page.locator("#content-view article");
  await expect(article).toBeVisible();

  // 4. Simulate text selection directly in the transcript article. The mobile
  // transcript view now renders raw article text, not paragraph nodes.
  const box = await article.boundingBox();
  if (!box) throw new Error("Could not find transcript bounding box");

  // Drag to select text
  await page.mouse.move(box.x + 10, box.y + 10);
  await page.mouse.down();
  await page.mouse.move(box.x + 100, box.y + 10);
  await page.mouse.up();

  // 5. Verify the custom toolbar appears at the bottom
  const toolbar = page.locator(".text-action-toolbar");
  await expect(toolbar).toBeVisible();

  // 6. Verify buttons are present
  await expect(
    toolbar.locator('button[aria-label="Save selected text as a highlight"]'),
  ).toBeVisible();
  await expect(toolbar.getByRole("button", { name: "Correct" })).toBeVisible();

  // 7. Verify swipe doesn't trigger if we are selecting
  // (In reality, the above mouse move already verifies that preventDefault didn't kill selection)
  const selection = await page.evaluate(() =>
    window.getSelection()?.toString(),
  );
  expect(selection?.length).toBeGreaterThan(0);
});
