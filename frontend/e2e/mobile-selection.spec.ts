import { expect, test, devices } from "@playwright/test";

test.use({ ...devices["iPhone 13"] });

test("mobile text selection shows the custom toolbar at the bottom", async ({
  page,
}) => {
  // 1. Navigate to home
  await page.goto("/");

  // 2. Select a channel and video (reuse logic from workspace.spec.ts if needed, but keep it simple)
  // 2. Select a channel and video from the mobile overlay
  // The overlay is identified by the "Channels" aria-label in MobileChannelGallery
  const channelScroller = page.locator('div[aria-label="Channels"]').first();
  await expect(channelScroller).toBeVisible({ timeout: 15000 });

  const channelRow = channelScroller.locator("button").first();
  await expect(channelRow).toBeVisible();
  await channelRow.click();

  // Wait for the visible mobile browse sheet to render video rows.
  const videoButton = page
    .locator('section[aria-label="Browse"] aside#workspace button')
    .filter({ has: page.locator("p.line-clamp-2") })
    .first();
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
