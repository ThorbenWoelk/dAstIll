import { devices, expect, test } from "@playwright/test";

import { resetClientState } from "./test-helpers";
import {
  buildMockWorkspaceBootstrap,
  installMockWorkspaceApi,
  navigateViaInjectedLink,
} from "./workspace-mock-api";

test.use({ ...devices["iPhone 13"] });

test.beforeEach(async ({ page }) => {
  await resetClientState(page);
});

test("mobile header reserves the top safe area", async ({ page }) => {
  await page.goto("/");

  const viewportMeta = page.locator('meta[name="viewport"]');
  await expect(viewportMeta).toHaveAttribute("content", /viewport-fit=cover/);

  const header = page.getByRole("banner");
  const main = page.locator("#main-content");

  await expect(header).toBeVisible();
  await expect(main).toBeVisible();

  const before = await page.evaluate(() => {
    const headerEl = document.querySelector("header");
    const mainEl = document.getElementById("main-content");

    if (
      !(headerEl instanceof HTMLElement) ||
      !(mainEl instanceof HTMLElement)
    ) {
      throw new Error("Workspace shell layout is missing");
    }

    return {
      headerHeight: Math.round(headerEl.getBoundingClientRect().height),
      mainTop: Math.round(mainEl.getBoundingClientRect().top),
    };
  });

  await page.evaluate(() => {
    document.documentElement.style.setProperty("--safe-area-inset-top", "24px");
  });

  await expect
    .poll(async () =>
      page.evaluate(() => {
        const headerEl = document.querySelector("header");
        if (!(headerEl instanceof HTMLElement)) {
          throw new Error("Workspace shell header is missing");
        }

        return Math.round(headerEl.getBoundingClientRect().height);
      }),
    )
    .toBe(before.headerHeight + 24);

  await expect
    .poll(async () =>
      page.evaluate(() => {
        const mainEl = document.getElementById("main-content");
        if (!(mainEl instanceof HTMLElement)) {
          throw new Error("Workspace shell main content is missing");
        }

        return Math.round(mainEl.getBoundingClientRect().top);
      }),
    )
    .toBe(before.mainTop + 24);
});

test("mobile filter button opens the filter menu above the browse overlay", async ({
  page,
}) => {
  const selectedChannelId = "channel-filter";
  const selectedPath = `/?source=${selectedChannelId}`;
  const bootstrap = buildMockWorkspaceBootstrap({
    channelId: selectedChannelId,
    channelName: "Filter test channel",
    channelHandle: "@filter-test",
    containerId: "container-filter",
    videoId: "video-filter",
    videoTitle: "Filter fixture video",
    qualityScore: 7,
  });

  await installMockWorkspaceApi(page, { bootstrap });

  await page.goto("/");
  await expect(page.getByRole("banner")).toBeVisible();
  await navigateViaInjectedLink(page, selectedPath);

  const filterButton = page
    .getByRole("banner")
    .getByRole("button", { name: "Video filters" })
    .first();

  await expect
    .poll(() => new URL(page.url()).search)
    .toContain(`source=${selectedChannelId}`);
  await expect(filterButton).toBeVisible();
  await expect(filterButton).toBeEnabled();
  await filterButton.click();

  const filterMenu = page.getByRole("menu", { name: "Video filters" });
  await expect(filterButton).toHaveAttribute("aria-expanded", "true");
  await expect(filterMenu).toBeVisible();
  await expect(
    filterMenu.getByRole("menuitemradio", { name: "All" }).first(),
  ).toBeVisible();
});

test("mobile queue filter button keeps full tap target and opens its menu", async ({
  page,
}) => {
  const selectedChannelId = "queue-filter";
  const selectedPath = `/download-queue?source=${selectedChannelId}`;
  const bootstrap = buildMockWorkspaceBootstrap({
    channelId: selectedChannelId,
    channelName: "Queue filter channel",
    channelHandle: "@queue-filter",
    containerId: "container-queue-filter",
    videoId: "video-queue-filter",
    videoTitle: "Queue filter fixture video",
    qualityScore: 6,
  });

  await installMockWorkspaceApi(page, { bootstrap });

  await page.goto(selectedPath);
  await expect(page.getByRole("banner")).toBeVisible();

  const filterButton = page
    .getByRole("banner")
    .getByRole("button", { name: "Video filters" })
    .first();

  await expect
    .poll(() => new URL(page.url()).search)
    .toContain(`source=${selectedChannelId}`);
  await expect(filterButton).toBeVisible();
  await expect(filterButton).toBeEnabled();

  const filterButtonBounds = await filterButton.boundingBox();
  expect(filterButtonBounds?.width ?? 0).toBeGreaterThanOrEqual(36);
  expect(filterButtonBounds?.height ?? 0).toBeGreaterThanOrEqual(36);

  await filterButton.click();

  const filterMenu = page.getByRole("menu", { name: "Video filters" });
  await expect(filterButton).toHaveAttribute("aria-expanded", "true");
  await expect(filterMenu).toBeVisible();
  await expect(
    filterMenu.getByRole("menuitemradio", { name: "All" }).first(),
  ).toBeVisible();
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
  await expect(mobileTabsCue).toHaveCount(0);

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

test("mobile summary eval pill opens the quality drawer", async ({ page }) => {
  const selectedChannelId = "channel-eval";
  const selectedVideoId = "video-eval";
  const selectedPath = `/?source=${selectedChannelId}&item=${selectedVideoId}&content=summary`;
  const bootstrap = buildMockWorkspaceBootstrap({
    channelId: selectedChannelId,
    channelName: "Quality test channel",
    channelHandle: "@quality-test",
    containerId: "container-eval",
    videoId: selectedVideoId,
    videoTitle: "Mobile eval regression fixture",
    qualityScore: 8,
    selectedItemId: selectedVideoId,
    totalChunkCount: 8,
  });

  const summary = {
    video_id: selectedVideoId,
    content:
      "This mocked summary exists only to verify that the mobile evaluation drawer stays tappable.",
    model_used: "glm-5.1:cloud",
    quality_score: 8,
    quality_note:
      "Strong structure.\n\n- Keeps the central claim intact.\n- Leaves a clear next question for the reader.",
    quality_model_used: "gemma4:31b-cloud",
    summary_tags: ["clear", "structured"],
    summary_tags_evaluated: true,
  };

  const videoInfo = {
    video_id: selectedVideoId,
    watch_url: "https://www.youtube.com/watch?v=video-eval",
    title: "Mobile eval regression fixture",
    description: "Fixture video info for the mobile eval drawer regression.",
    thumbnail_url: null,
    channel_name: "Quality test channel",
    channel_id: selectedChannelId,
    published_at: "2026-04-11T18:30:00.000Z",
    duration_iso8601: "PT8M12S",
    duration_seconds: 492,
    view_count: 1280,
  };

  await installMockWorkspaceApi(page, { bootstrap, summary, videoInfo });

  await page.goto("/");
  await expect(page.getByRole("banner")).toBeVisible();
  await navigateViaInjectedLink(page, selectedPath);
  await expect
    .poll(() => new URL(page.url()).search)
    .toContain(`source=${selectedChannelId}`);
  await page
    .getByRole("region", { name: "Browse" })
    .getByRole("button", { name: /Mobile eval regression fixture/i })
    .first()
    .click();
  await page.getByRole("button", { name: "Summary", exact: true }).click();
  await expect(page.locator("#content-view article")).toContainText(
    "mobile evaluation drawer stays tappable",
  );

  const evalTrigger = page.locator(
    ".summary-embed-strip-eval button[aria-controls='summary-quality-note']",
  );

  await expect(evalTrigger.first()).toBeVisible();
  await evalTrigger.first().click();

  const evalDrawer = page.locator("#summary-quality-note");
  await expect(evalDrawer).toBeVisible();
  await expect(evalTrigger.first()).toHaveAttribute("aria-expanded", "true");
  await expect(evalDrawer.locator(".eval-note-markdown")).toContainText(/\S+/);
});
