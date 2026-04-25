import { expect, test, devices } from "@playwright/test";
import { PLAYWRIGHT_MAINTENANCE_MODE } from "./runtime-mode";
import { resetClientState } from "./test-helpers";

test.use({ ...devices["iPhone 13"] });

test.skip(
  PLAYWRIGHT_MAINTENANCE_MODE,
  "Mobile workspace text-selection assertions do not apply when the home route is in maintenance mode.",
);

test.beforeEach(async ({ page }) => {
  await resetClientState(page);
});

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
      await page
        .getByRole("banner")
        .getByRole("link", { name: "Go to dAstIll home" })
        .click();
    }
  }
  await expect(browseRegion).toBeVisible({ timeout: 15000 });

  const sourcesPane = browseRegion.getByRole("complementary").first();
  const channelButtons = sourcesPane
    .getByRole("button")
    .filter({ hasNotText: "Add source" });
  const browseState = async () => {
    if ((await channelButtons.count()) > 0) return "channels";
    return "empty";
  };

  await page.waitForTimeout(1500);

  test.skip(
    (await browseState()) !== "channels",
    "Mobile browse view has no channels; run against a seeded backend",
  );

  const channelButton = channelButtons.first();
  await expect(channelButton).toBeVisible();
  await channelButton.click();

  const videoList = browseRegion.getByRole("complementary").first();
  const videoButtons = videoList
    .getByRole("button")
    .filter({ hasNotText: "Adjust sync date" });
  const videoState = async () => {
    if ((await videoButtons.count()) > 0) return "videos";
    return "empty";
  };

  await page.waitForTimeout(1500);

  test.skip(
    (await videoState()) !== "videos",
    "Mobile browse view has no videos; run against a seeded backend",
  );

  const videoButton = videoButtons.first();
  await expect(videoButton).toBeVisible();
  await videoButton.click();

  // 3. Switch to Transcript tab if not already there
  await page.getByRole("button", { name: "Transcript", exact: true }).click();

  const article = page.locator("#content-view article");
  await expect(article).toBeVisible();

  // 4. Build a real DOM selection directly. This is more stable than mouse drag
  // on mobile emulation while still exercising the toolbar's selectionchange path.
  const selectionLength = await page.evaluate(() => {
    const article = document.querySelector("#content-view article");
    if (!article) return 0;

    const walker = document.createTreeWalker(article, NodeFilter.SHOW_TEXT);
    let textNode: Text | null = null;
    let currentNode = walker.nextNode();
    while (currentNode) {
      if (
        currentNode instanceof Text &&
        currentNode.textContent &&
        currentNode.textContent.trim().length > 0
      ) {
        textNode = currentNode;
        break;
      }
      currentNode = walker.nextNode();
    }

    if (!textNode?.textContent) return 0;

    const text = textNode.textContent;
    const start = text.search(/\S/);
    if (start < 0) return 0;

    const end = Math.min(text.length, start + 24);
    const range = document.createRange();
    range.setStart(textNode, start);
    range.setEnd(textNode, end);

    const selection = window.getSelection();
    if (!selection) return 0;
    selection.removeAllRanges();
    selection.addRange(range);
    document.dispatchEvent(new Event("selectionchange"));
    document.dispatchEvent(new PointerEvent("pointerup", { bubbles: true }));
    return selection.toString().length;
  });
  expect(selectionLength).toBeGreaterThan(0);

  // 5. Verify the custom toolbar appears at the bottom
  const toolbar = page.locator(".text-action-toolbar");
  await expect(toolbar).toBeVisible();

  // 6. Verify buttons are present
  await expect(
    toolbar.locator('button[aria-label="Save selected text as a highlight"]'),
  ).toBeVisible();
  await expect(
    toolbar.getByRole("button", { name: "Correct spelling" }),
  ).toBeVisible();

  // 7. Verify swipe doesn't trigger if we are selecting
  // (In reality, the above mouse move already verifies that preventDefault didn't kill selection)
  const selection = await page.evaluate(() =>
    window.getSelection()?.toString(),
  );
  expect(selection?.length).toBeGreaterThan(0);
});
