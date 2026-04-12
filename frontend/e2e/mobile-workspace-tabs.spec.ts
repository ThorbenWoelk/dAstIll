import { devices, expect, test } from "@playwright/test";

import { resetClientState } from "./test-helpers";

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

  const bootstrap = {
    ai_available: true,
    ai_status: "cloud",
    containers: [
      {
        id: "container-eval",
        kind: "series",
        title: "Quality test container",
        provider: "you_tube",
        backing_kind: "feed",
        user_editable: true,
        source_ids: [selectedChannelId],
      },
    ],
    sources: [
      {
        id: selectedChannelId,
        provider: "you_tube",
        source_kind: "you_tube_channel",
        container_id: "container-eval",
        container_kind: "series",
        backing_kind: "feed",
        title: "Quality test channel",
        subtitle: "@quality-test",
        handle: "@quality-test",
        thumbnail_url: null,
        requires_auth: false,
        public_content_available: true,
        entitled_content_available: true,
        external_ids: [
          { provider: "you_tube", external_id: selectedChannelId },
        ],
      },
    ],
    channels: [
      {
        id: selectedChannelId,
        handle: "@quality-test",
        name: "Quality test channel",
        thumbnail_url: null,
        added_at: "2026-04-12T09:00:00.000Z",
        earliest_sync_date: "2026-04-01T00:00:00.000Z",
        earliest_sync_date_user_set: false,
      },
    ],
    selected_source_id: selectedChannelId,
    selected_channel_id: selectedChannelId,
    selected_item_id: selectedVideoId,
    snapshot: {
      channel_id: selectedChannelId,
      source_id: selectedChannelId,
      container: {
        id: "container-eval",
        kind: "series",
        title: "Quality test container",
        provider: "you_tube",
        backing_kind: "feed",
        user_editable: true,
        source_ids: [selectedChannelId],
      },
      source: {
        id: selectedChannelId,
        provider: "you_tube",
        source_kind: "you_tube_channel",
        container_id: "container-eval",
        container_kind: "series",
        backing_kind: "feed",
        title: "Quality test channel",
        subtitle: "@quality-test",
        handle: "@quality-test",
        thumbnail_url: null,
        requires_auth: false,
        public_content_available: true,
        entitled_content_available: true,
        external_ids: [
          { provider: "you_tube", external_id: selectedChannelId },
        ],
      },
      sync_depth: {
        earliest_sync_date: "2026-04-01T00:00:00.000Z",
        earliest_sync_date_user_set: false,
        derived_earliest_ready_date: "2026-04-10T00:00:00.000Z",
      },
      channel_video_count: 1,
      has_more: false,
      next_offset: null,
      videos: [
        {
          id: selectedVideoId,
          channel_id: selectedChannelId,
          title: "Mobile eval regression fixture",
          thumbnail_url: null,
          published_at: "2026-04-11T18:30:00.000Z",
          is_short: false,
          transcript_status: "ready",
          summary_status: "ready",
          acknowledged: false,
          retry_count: 0,
          quality_score: 8,
        },
      ],
      items: [],
      parts: [],
    },
    search_status: {
      available: true,
      model: "embeddinggemma",
      dimensions: 768,
      pending: 0,
      indexing: 0,
      ready: 1,
      failed: 0,
      total_sources: 1,
      total_chunk_count: 8,
      embedded_chunk_count: 8,
      vector_index_ready: true,
      retrieval_mode: "hybrid_ann",
    },
  };

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

  await page.route("**/api/**", async (route) => {
    const url = new URL(route.request().url());

    if (url.pathname === "/api/workspace/bootstrap") {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(bootstrap),
      });
      return;
    }

    if (url.pathname === `/api/channels/${selectedChannelId}/snapshot`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(bootstrap.snapshot),
      });
      return;
    }

    if (url.pathname === `/api/channels/${selectedChannelId}/backfill`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          videos_added: 0,
          fetched_count: 0,
          exhausted: true,
        }),
      });
      return;
    }

    if (url.pathname === `/api/channels/${selectedChannelId}/videos`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          source_id: selectedChannelId,
          videos: [],
          items: [],
          parts: [],
          has_more: false,
          next_offset: null,
        }),
      });
      return;
    }

    if (url.pathname === `/api/channels/${selectedChannelId}/sync-depth`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(bootstrap.snapshot.sync_depth),
      });
      return;
    }

    if (url.pathname === `/api/videos/${selectedVideoId}/summary`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(summary),
      });
      return;
    }

    if (url.pathname === `/api/videos/${selectedVideoId}/summary/audio/debug`) {
      await route.fulfill({
        status: 404,
        contentType: "text/plain",
        body: "audio not generated",
      });
      return;
    }

    if (url.pathname === `/api/videos/${selectedVideoId}/info/ensure`) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(videoInfo),
      });
      return;
    }

    await route.continue();
  });

  await page.goto("/");
  await expect(page.getByRole("banner")).toBeVisible();

  await page.evaluate(
    ({ href }) => {
      const existing = document.getElementById("__test-route");
      existing?.remove();
      const link = document.createElement("a");
      link.id = "__test-route";
      link.href = href;
      link.textContent = "route";
      document.body.appendChild(link);
    },
    { href: selectedPath },
  );

  await page.evaluate(() => {
    const link = document.getElementById("__test-route");
    if (!(link instanceof HTMLAnchorElement)) {
      throw new Error("Client navigation link was not mounted");
    }
    link.click();
  });
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
    ".summary-embed-strip-mobile-eval button[aria-controls='summary-quality-note']",
  );

  await expect(evalTrigger.first()).toBeVisible();
  await evalTrigger.first().click();

  const evalDrawer = page.locator("#summary-quality-note");
  await expect(evalDrawer).toBeVisible();
  await expect(evalTrigger.first()).toHaveAttribute("aria-expanded", "true");
  await expect(evalDrawer.locator(".eval-note-markdown")).toContainText(/\S+/);
});
