import { expect, test, type Page } from "@playwright/test";
import { openFreshGuestPage } from "./test-helpers";
import {
  buildMockWorkspaceBootstrap,
  installMockWorkspaceApi,
  navigateViaInjectedLink,
} from "./workspace-mock-api";

const READY_MS = 120_000;
// Chromium on macOS reserves Cmd+<number> for browser tab switching, so Playwright
// never delivers that chord to the page. Use Ctrl there to keep the app-level
// shortcut path covered in automation; the app still supports Cmd for humans.
const PRIMARY_MODIFIER = "Control";

function workspaceSidebar(page: Page) {
  // Two aside#workspace nodes can exist (desktop rail + mobile browse dialog). Exclude the dialog copy.
  return page
    .locator('xpath=//aside[@id="workspace"][not(ancestor::*[@role="dialog"])]')
    .first();
}

function workspaceDesktopTabs(page: Page) {
  return page.locator("#workspace-tabs-desktop").first();
}

async function workspaceHasSeedData(page: Page): Promise<boolean> {
  await page.goto("/");
  const sidebar = workspaceSidebar(page);
  await expect(sidebar).toBeVisible();

  // SSR can render before the client bootstrap finishes; poll until we either
  // have channel rows or a confirmed empty workspace (not the loading skeleton).
  await expect
    .poll(
      async () => {
        const count = await sidebar.locator("[data-channel-id]").count();
        if (count > 0) return "channels";
        const empty = sidebar
          .getByText("Start by following a channel.")
          .first();
        if (await empty.isVisible()) return "empty";
        return "loading";
      },
      {
        timeout: READY_MS,
        message:
          "Timed out waiting for channels or empty workspace (still loading?)",
      },
    )
    .not.toBe("loading");

  return (await sidebar.locator("[data-channel-id]").count()) > 0;
}

test.beforeEach(async ({ page }) => {
  await openFreshGuestPage(page, "/");
});

test("sidebar lists channels and each row shows video titles", async ({
  page,
}) => {
  const hasData = await workspaceHasSeedData(page);
  if (!hasData) {
    test.skip(true, "Workspace has no channels; run against a seeded backend");
  }

  const sidebar = workspaceSidebar(page);
  const channelRows = sidebar.locator("[data-channel-id]");
  await expect(channelRows.first()).toBeVisible();
  const n = await channelRows.count();
  expect(n).toBeGreaterThan(0);

  for (let i = 0; i < n; i++) {
    const titles = channelRows
      .nth(i)
      .locator("xpath=following-sibling::div[1]")
      .locator("p.line-clamp-2");
    await expect(titles.first()).toBeVisible({ timeout: READY_MS });
  }
});

test("switching content tabs shows different views", async ({ page }) => {
  const hasData = await workspaceHasSeedData(page);
  if (!hasData) {
    test.skip(true, "Workspace has no channels; run against a seeded backend");
  }

  const sidebar = workspaceSidebar(page);
  await sidebar
    .locator("[data-channel-id]")
    .first()
    .locator("button")
    .first()
    .click();
  await expect(
    sidebar.locator("#videos").getByRole("button").first(),
  ).toBeVisible({
    timeout: READY_MS,
  });
  await sidebar.locator("#videos").getByRole("button").first().click();

  await workspaceDesktopTabs(page)
    .getByRole("button", { name: "Transcript", exact: true })
    .click();
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  await expect(page.locator("#content-view article")).not.toBeEmpty();

  await workspaceDesktopTabs(page)
    .getByRole("button", { name: "Info", exact: true })
    .click();
  await expect(page.getByText("Published").first()).toBeVisible({
    timeout: READY_MS,
  });
  await expect(page.locator("#content-view article")).toHaveCount(0);

  await workspaceDesktopTabs(page)
    .getByRole("button", { name: "Summary", exact: true })
    .click();
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  await expect(page.locator("#content-view article")).not.toBeEmpty();
  await expect(page.locator("#workspace")).toBeVisible();
});

test("summary and transcript match the selected video after changing channel", async ({
  page,
}) => {
  const hasData = await workspaceHasSeedData(page);
  if (!hasData) {
    test.skip(true, "Workspace has no channels; run against a seeded backend");
  }

  const sidebar = workspaceSidebar(page);
  const channelRows = sidebar.locator("[data-channel-id]");
  if ((await channelRows.count()) < 2) {
    test.skip(
      true,
      "Need at least two channels to verify per-channel content switching",
    );
  }

  async function selectChannelAndFirstVideo(index: number) {
    await channelRows.nth(index).locator("button").first().click();
    await expect(
      sidebar.locator("#videos").getByRole("button").first(),
    ).toBeVisible({
      timeout: READY_MS,
    });
    await sidebar.locator("#videos").getByRole("button").first().click();
  }

  await selectChannelAndFirstVideo(0);
  await workspaceDesktopTabs(page)
    .getByRole("button", { name: "Transcript", exact: true })
    .click();
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  const transcriptA = (
    await page.locator("#content-view article").innerText()
  ).trim();

  await selectChannelAndFirstVideo(1);
  await workspaceDesktopTabs(page)
    .getByRole("button", { name: "Transcript", exact: true })
    .click();
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  const transcriptB = (
    await page.locator("#content-view article").innerText()
  ).trim();
  expect(transcriptB.length).toBeGreaterThan(0);
  expect(transcriptB).not.toBe(transcriptA);

  await workspaceDesktopTabs(page)
    .getByRole("button", { name: "Summary", exact: true })
    .click();
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  const summaryB = (
    await page.locator("#content-view article").innerText()
  ).trim();
  expect(summaryB.length).toBeGreaterThan(0);
  expect(summaryB).not.toBe(transcriptA);
});

test("workspace feature guide opens from guide URL param (same state as Guide control)", async ({
  page,
}) => {
  // Matches tour.restoreFromUrl() / ?guide=0; avoids flaky nav-click races with client URL sync.
  await page.goto("/?guide=0");
  await page.waitForLoadState("load");

  const dialog = page.getByRole("dialog", { name: "Feature guide" });
  await expect(dialog).toBeVisible({ timeout: READY_MS });
  await expect(dialog.getByText("Welcome to dAstIll")).toBeVisible();
});

test("Cmd/Ctrl+1 navigates from queue to workspace without full reload hang", async ({
  page,
}) => {
  await page.goto("/download-queue");
  await expect
    .poll(() => new URL(page.url()).pathname)
    .toContain("download-queue");
  await page.waitForTimeout(1500);
  await page.keyboard.press(`${PRIMARY_MODIFIER}+1`);

  await expect.poll(() => new URL(page.url()).pathname).toBe("/");
  await expect(workspaceSidebar(page)).toBeVisible({ timeout: READY_MS });
});

test("channel row click opens the overview page and overview exposes delete", async ({
  page,
}) => {
  const selectedChannelId = "channel-overview";
  const bootstrap = buildMockWorkspaceBootstrap({
    channelId: selectedChannelId,
    channelName: "Overview test channel",
    channelHandle: "@overview-test",
    containerId: "container-overview",
    videoId: "video-overview",
    videoTitle: "Overview fixture video",
    qualityScore: 7,
  });

  await installMockWorkspaceApi(page, { bootstrap });
  await page.goto("/");

  const sidebar = workspaceSidebar(page);
  await expect(sidebar.locator("[data-channel-id]").first()).toBeVisible({
    timeout: READY_MS,
  });

  await sidebar.locator("[data-channel-id]").first().locator("button").click();

  await expect
    .poll(() => new URL(page.url()).pathname)
    .toBe(`/channels/${selectedChannelId}`);
  await expect(
    page.getByRole("button", { name: "Delete channel" }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Delete channel" }).click();
  await expect(
    page.getByRole("dialog", { name: "Sign in required" }),
  ).toBeVisible();
});

test("guest mark read toggle opens the sign-in prompt", async ({ page }) => {
  const hasData = await workspaceHasSeedData(page);
  if (!hasData) {
    test.skip(true, "Workspace has no channels; run against a seeded backend");
  }

  const sidebar = workspaceSidebar(page);
  await sidebar
    .locator("[data-channel-id]")
    .first()
    .locator("button")
    .first()
    .click();
  await expect(
    sidebar.locator("#videos").getByRole("button").first(),
  ).toBeVisible({
    timeout: READY_MS,
  });
  await sidebar.locator("#videos").getByRole("button").first().click();

  const toggle = page.locator("#mark-read-toggle");
  await expect(toggle).toBeVisible({ timeout: READY_MS });
  const before = await toggle.getAttribute("aria-pressed");
  await toggle.click();
  await expect(
    page.getByRole("dialog", { name: "Sign in required" }),
  ).toBeVisible();
  await expect(toggle).toHaveAttribute("aria-pressed", before ?? "false");
});

test("guest unread filter keeps videos visible when mark read requires sign-in", async ({
  page,
}) => {
  const hasData = await workspaceHasSeedData(page);
  if (!hasData) {
    test.skip(true, "Workspace has no channels; run against a seeded backend");
  }

  const sidebar = workspaceSidebar(page);
  await sidebar
    .locator("[data-channel-id]")
    .first()
    .locator("button")
    .first()
    .click();
  const videoButtons = sidebar.locator("#videos").getByRole("button");
  await expect(videoButtons.first()).toBeVisible({ timeout: READY_MS });
  const targetButton = videoButtons.first();
  const targetTitle = (
    await targetButton.locator("p.line-clamp-2").innerText()
  ).trim();
  await targetButton.click();

  const toggle = page.locator("#mark-read-toggle");
  await expect(toggle).toBeVisible({ timeout: READY_MS });
  if ((await toggle.getAttribute("aria-label")) === "Mark as unread") {
    await toggle.click();
    await expect(toggle).toHaveAttribute("aria-label", "Mark as read");
  }

  await page.getByRole("button", { name: "Video filters" }).click();
  await page.getByRole("menuitemradio", { name: "Unread" }).click();
  await expect
    .poll(() => new URL(page.url()).searchParams.get("ack"))
    .toBe("unack");
  await expect(sidebar.getByText("Unread", { exact: true })).toBeVisible();
  await expect(
    sidebar.locator("#videos").getByText(targetTitle, { exact: true }),
  ).toBeVisible();

  await toggle.click();
  await expect(
    page.getByRole("dialog", { name: "Sign in required" }),
  ).toBeVisible();
  await expect(
    sidebar.locator("#videos").getByText(targetTitle, { exact: true }),
  ).toBeVisible();
});

test("desktop summary eval score opens the quality drawer", async ({
  page,
}) => {
  const selectedChannelId = "channel-eval";
  const selectedVideoId = "video-eval";
  const selectedPath = `/?source=${selectedChannelId}&item=${selectedVideoId}&content=summary`;
  const bootstrap = buildMockWorkspaceBootstrap({
    channelId: selectedChannelId,
    channelName: "Quality test channel",
    channelHandle: "@quality-test",
    containerId: "container-eval",
    videoId: selectedVideoId,
    videoTitle: "Desktop eval regression fixture",
    qualityScore: 8,
    selectedItemId: selectedVideoId,
    totalChunkCount: 8,
  });

  const summary = {
    video_id: selectedVideoId,
    content:
      "This mocked summary exists only to verify that the desktop evaluation drawer opens.",
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
    title: "Desktop eval regression fixture",
    description: "Fixture video info for the desktop eval drawer regression.",
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
  await expect(workspaceSidebar(page)).toBeVisible();
  await navigateViaInjectedLink(page, selectedPath);
  await expect
    .poll(() => new URL(page.url()).search)
    .toContain(`source=${selectedChannelId}`);
  const sidebar = workspaceSidebar(page);
  await expect(
    sidebar.locator("#videos").getByRole("button").first(),
  ).toBeVisible({
    timeout: READY_MS,
  });
  await sidebar.locator("#videos").getByRole("button").first().click();
  await workspaceDesktopTabs(page)
    .getByRole("button", { name: "Summary", exact: true })
    .click();
  await expect(page.locator("#content-view article")).toContainText(
    "desktop evaluation drawer opens",
  );

  const evalTrigger = page.locator(
    ".summary-embed-strip-eval button[aria-controls='summary-quality-note']",
  );
  const evalDrawer = page.locator("#summary-quality-note");

  await expect(evalTrigger).toBeVisible();
  await evalTrigger.click();

  await expect(evalDrawer).toBeVisible();
  await expect(evalTrigger).toHaveAttribute("aria-expanded", "true");
  await expect(evalDrawer.locator(".eval-note-markdown")).toContainText(/\S+/);
});
