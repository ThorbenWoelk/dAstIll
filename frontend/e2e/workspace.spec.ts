import { expect, test, type Locator, type Page } from "@playwright/test";
import { openFreshGuestPage } from "./test-helpers";
import {
  buildMockWorkspaceBootstrap,
  installMockWorkspaceApi,
  navigateViaInjectedLink,
} from "./workspace-mock-api";

const READY_MS = 120_000;

function workspaceSidebar(page: Page) {
  // Two aside#workspace nodes can exist (desktop rail + mobile browse dialog). Exclude the dialog copy.
  return page
    .locator('xpath=//aside[@id="workspace"][not(ancestor::*[@role="dialog"])]')
    .first();
}

function workspaceDesktopTabs(page: Page) {
  return page.locator("#workspace-tabs-desktop").first();
}

async function installSeededWorkspaceApi(page: Page) {
  const primary = buildMockWorkspaceBootstrap({
    channelId: "channel-alpha",
    channelName: "Alpha workspace channel",
    channelHandle: "@alpha-workspace",
    containerId: "container-alpha",
    videoId: "video-alpha",
    videoTitle: "Alpha workspace fixture video",
    qualityScore: 8,
  });
  const secondary = buildMockWorkspaceBootstrap({
    channelId: "channel-beta",
    channelName: "Beta workspace channel",
    channelHandle: "@beta-workspace",
    containerId: "container-beta",
    videoId: "video-beta",
    videoTitle: "Beta workspace fixture video",
    qualityScore: 6,
  });
  const bootstrap = {
    ...primary,
    containers: [...primary.containers, ...secondary.containers],
    sources: [...primary.sources, ...secondary.sources],
    channels: [...primary.channels, ...secondary.channels],
  };

  await installMockWorkspaceApi(page, {
    bootstrap,
    snapshots: {
      [primary.selected_channel_id]: primary.snapshot,
      [secondary.selected_channel_id]: secondary.snapshot,
    },
    transcripts: {
      "video-alpha": {
        video_id: "video-alpha",
        raw_text:
          "Alpha transcript fixture. The first channel keeps its own transcript.",
        formatted_markdown: null,
        render_mode: "plain_text",
      },
      "video-beta": {
        video_id: "video-beta",
        raw_text:
          "Beta transcript fixture. The second channel proves content changes.",
        formatted_markdown: null,
        render_mode: "plain_text",
      },
    },
    summaries: {
      "video-alpha": {
        video_id: "video-alpha",
        content:
          "Alpha summary fixture. The first channel keeps its own summary.",
        model_used: "glm-5.1:cloud",
        quality_score: 8,
        quality_note: "Clear alpha fixture.",
        quality_model_used: "gemma4:31b-cloud",
        summary_tags: ["alpha"],
        summary_tags_evaluated: true,
      },
      "video-beta": {
        video_id: "video-beta",
        content:
          "Beta summary fixture. The second channel proves summary content changes.",
        model_used: "glm-5.1:cloud",
        quality_score: 6,
        quality_note: "Clear beta fixture.",
        quality_model_used: "gemma4:31b-cloud",
        summary_tags: ["beta"],
        summary_tags_evaluated: true,
      },
    },
    videoInfos: {
      "video-alpha": {
        video_id: "video-alpha",
        watch_url: "https://www.youtube.com/watch?v=video-alpha",
        title: "Alpha workspace fixture video",
        description: "Fixture info for the alpha workspace video.",
        thumbnail_url: null,
        channel_name: "Alpha workspace channel",
        channel_id: "channel-alpha",
        published_at: "2026-04-11T18:30:00.000Z",
        duration_iso8601: "PT8M12S",
        duration_seconds: 492,
        view_count: 1280,
      },
      "video-beta": {
        video_id: "video-beta",
        watch_url: "https://www.youtube.com/watch?v=video-beta",
        title: "Beta workspace fixture video",
        description: "Fixture info for the beta workspace video.",
        thumbnail_url: null,
        channel_name: "Beta workspace channel",
        channel_id: "channel-beta",
        published_at: "2026-04-12T18:30:00.000Z",
        duration_iso8601: "PT9M24S",
        duration_seconds: 564,
        view_count: 980,
      },
    },
  });
}

async function openSeededWorkspace(page: Page, path = "/") {
  await installSeededWorkspaceApi(page);
  await page.goto(path);
  const sidebar = workspaceSidebar(page);
  await expect(sidebar).toBeVisible();
  await expect(sidebar.locator("[data-channel-id]").first()).toBeVisible({
    timeout: READY_MS,
  });
  return sidebar;
}

async function dispatchVisibleClick(locator: Locator): Promise<void> {
  await expect(locator).toBeVisible({ timeout: READY_MS });
  await locator.dispatchEvent("click");
}

test.beforeEach(async ({ page }) => {
  await openFreshGuestPage(page, "/");
});

test("sidebar lists channels and each row shows video titles", async ({
  page,
}) => {
  const sidebar = await openSeededWorkspace(page);
  const channelRows = sidebar.locator("[data-channel-id]");
  await expect(channelRows.first()).toBeVisible();
  await expect(channelRows).toHaveCount(2);

  await expect(
    sidebar.locator("#videos").getByText("Alpha workspace fixture video"),
  ).toBeVisible({ timeout: READY_MS });

  await page.goto("/?source=channel-beta");
  await expect(
    sidebar.locator("#videos").getByText("Beta workspace fixture video"),
  ).toBeVisible({ timeout: READY_MS });
});

test("channel row chevron collapses the selected channel without reopening", async ({
  page,
}) => {
  const sidebar = await openSeededWorkspace(
    page,
    "/?source=channel-alpha&item=video-alpha&content=info",
  );
  const selectedChannelRow = sidebar.locator(
    '[data-channel-id="channel-alpha"]',
  );
  const selectedVideoList = sidebar.locator(
    '[data-channel-video-list="channel-alpha"]',
  );
  const selectedVideo = selectedVideoList.getByText(
    "Alpha workspace fixture video",
  );

  await expect(selectedVideo).toBeVisible({ timeout: READY_MS });
  await selectedChannelRow
    .getByRole("button", { name: "Collapse channel" })
    .click();

  await expect(
    selectedChannelRow.getByRole("button", { name: "Expand channel" }),
  ).toBeVisible();
  await expect(selectedVideo).toBeHidden();

  await page.waitForTimeout(250);
  await expect(
    selectedChannelRow.getByRole("button", { name: "Expand channel" }),
  ).toBeVisible();
  await expect(selectedVideo).toBeHidden();
});

test("channel overview stays loading instead of showing not found during auth handoff", async ({
  page,
}) => {
  const stale = buildMockWorkspaceBootstrap({
    channelId: "channel-stale",
    channelName: "Stale anonymous channel",
    channelHandle: "@stale-anon",
    containerId: "container-stale",
    videoId: "video-stale",
    videoTitle: "Stale anonymous fixture video",
    qualityScore: 4,
  });
  const authenticated = buildMockWorkspaceBootstrap({
    channelId: "channel-auth",
    channelName: "Authenticated channel",
    channelHandle: "@authenticated-channel",
    containerId: "container-auth",
    videoId: "video-auth",
    videoTitle: "Authenticated fixture video",
    qualityScore: 9,
  });
  let bootstrapCalls = 0;
  let secondBootstrapStarted = false;

  await page.addInitScript(() => {
    window.localStorage.setItem(
      "__dastill_e2e_auth",
      JSON.stringify({
        userId: "e2e-user",
        email: "e2e@example.com",
        token: "e2e-token",
      }),
    );
  });

  await installMockWorkspaceApi(page, {
    bootstrap: authenticated,
    snapshots: {
      "channel-stale": stale.snapshot,
      "channel-auth": authenticated.snapshot,
    },
  });
  await page.route("**/api/workspace/bootstrap**", async (route) => {
    bootstrapCalls += 1;
    if (bootstrapCalls === 1) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify(stale),
      });
      return;
    }

    secondBootstrapStarted = true;
    await new Promise((resolve) => setTimeout(resolve, 250));
    await route.fulfill({
      status: 200,
      contentType: "application/json",
      body: JSON.stringify(authenticated),
    });
  });

  await page.goto("/channels/channel-auth");
  await expect
    .poll(() => secondBootstrapStarted, { timeout: READY_MS })
    .toBe(true);
  await expect(page.getByText("Channel not found.")).toHaveCount(0);
  await expect(
    page.getByRole("heading", { level: 1, name: "Authenticated channel" }),
  ).toBeVisible({ timeout: READY_MS });
  await expect(page.getByText("Channel not found.")).toHaveCount(0);
});

test("desktop sidebar sits flush against the main content", async ({
  page,
}) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await openSeededWorkspace(page);
  const main = page.locator("#main-content");
  await expect(main).toBeVisible();

  const layout = await page.evaluate(() => {
    const sidebarNode = Array.from(
      document.querySelectorAll("aside#workspace"),
    ).find((node) => !node.closest('[role="dialog"]'));
    const mainNode = document.querySelector("#main-content");
    if (!(sidebarNode instanceof HTMLElement) || !mainNode) {
      return null;
    }

    const sidebarRect = sidebarNode.getBoundingClientRect();
    const mainRect = mainNode.getBoundingClientRect();
    return {
      gap: mainRect.left - sidebarRect.right,
    };
  });

  if (!layout) throw new Error("Workspace layout was not rendered");
  expect(Math.abs(layout.gap)).toBeLessThanOrEqual(1);
});

test("podcast subscribe protected action shows sign-in modal above drawer", async ({
  page,
}) => {
  await page.addInitScript(() => {
    window.localStorage.setItem(
      "__dastill_e2e_auth",
      JSON.stringify({
        userId: "e2e-user",
        email: "e2e@example.com",
        token: "e2e-token",
      }),
    );
  });
  await installSeededWorkspaceApi(page);

  let submittedInput: string | null = null;
  await page.route("**/api/channels", async (route) => {
    if (route.request().method() !== "POST") {
      await route.fallback();
      return;
    }

    const payload = route.request().postDataJSON() as { input?: string };
    submittedInput = payload.input ?? null;
    await route.fulfill({
      status: 403,
      contentType: "text/plain",
      body: "Sign-in required",
    });
  });

  await page.goto("/");
  const sidebar = workspaceSidebar(page);
  await expect(sidebar.locator("[data-channel-id]").first()).toBeVisible({
    timeout: READY_MS,
  });

  await sidebar.getByLabel("Add source").click();
  await page.getByRole("button", { name: /Podcast RSS/ }).click();
  await page
    .getByLabel("Podcast RSS")
    .fill("https://feeds.simplecast.com/54nAGcIl");
  await page.getByRole("button", { name: "Subscribe to podcast feed" }).click();

  await expect
    .poll(() => submittedInput)
    .toBe("podcast: https://feeds.simplecast.com/54nAGcIl");

  const modal = page.getByRole("dialog", { name: "Sign in required" });
  await expect(modal).toBeVisible();
  await expect(
    page.getByRole("dialog", { name: "Pick a source type" }),
  ).toBeVisible();

  const modalOwnsTopElement = await modal.evaluate((node) => {
    const rect = node.getBoundingClientRect();
    const topElement = document.elementFromPoint(
      rect.left + rect.width / 2,
      rect.top + rect.height / 2,
    );
    return topElement !== null && node.contains(topElement);
  });
  expect(modalOwnsTopElement).toBe(true);
});

test("mobile workspace still uses mobile chrome", async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 812 });
  await installSeededWorkspaceApi(page);
  await page.goto("/");

  await expect(page.getByRole("button", { name: "Open menu" })).toBeVisible({
    timeout: READY_MS,
  });
  await expect(workspaceDesktopTabs(page)).toBeHidden();
});

test("desktop workspace keeps desktop chrome", async ({ page }) => {
  await page.setViewportSize({ width: 1280, height: 900 });
  await openSeededWorkspace(page);

  await expect(workspaceSidebar(page)).toBeVisible();
  await expect(workspaceDesktopTabs(page)).toBeVisible();
});

test("switching content tabs shows different views", async ({ page }) => {
  await openSeededWorkspace(
    page,
    "/?source=channel-alpha&item=video-alpha&content=summary",
  );

  await dispatchVisibleClick(
    workspaceDesktopTabs(page).getByRole("button", {
      name: "Transcript",
      exact: true,
    }),
  );
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  await expect(page.locator("#content-view article")).not.toBeEmpty();

  await dispatchVisibleClick(
    workspaceDesktopTabs(page).getByRole("button", {
      name: "Info",
      exact: true,
    }),
  );
  await expect(page.getByText("Published").first()).toBeVisible({
    timeout: READY_MS,
  });
  await expect(page.locator("#content-view article")).toHaveCount(0);

  await dispatchVisibleClick(
    workspaceDesktopTabs(page).getByRole("button", {
      name: "Summary",
      exact: true,
    }),
  );
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  await expect(page.locator("#content-view article")).not.toBeEmpty();
  await expect(page.locator("#workspace")).toBeVisible();
});

test("summary and transcript match the selected video after changing channel", async ({
  page,
}) => {
  await openSeededWorkspace(
    page,
    "/?source=channel-alpha&item=video-alpha&content=transcript",
  );
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  const transcriptA = (
    await page.locator("#content-view article").innerText()
  ).trim();

  await page.goto("/?source=channel-beta&item=video-beta&content=transcript");
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  const transcriptB = (
    await page.locator("#content-view article").innerText()
  ).trim();
  expect(transcriptB.length).toBeGreaterThan(0);
  expect(transcriptB).not.toBe(transcriptA);

  await dispatchVisibleClick(
    workspaceDesktopTabs(page).getByRole("button", {
      name: "Summary",
      exact: true,
    }),
  );
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  const summaryB = (
    await page.locator("#content-view article").innerText()
  ).trim();
  expect(summaryB.length).toBeGreaterThan(0);
  expect(summaryB).not.toBe(transcriptA);
});

test("browser back restores workspace content from the URL", async ({
  page,
}) => {
  await openSeededWorkspace(
    page,
    "/?source=channel-alpha&item=video-alpha&content=transcript",
  );
  await expect(page.locator("#content-view article")).toContainText(
    "Alpha transcript fixture",
    { timeout: READY_MS },
  );

  await navigateViaInjectedLink(
    page,
    "/?source=channel-beta&item=video-beta&content=transcript",
  );
  await expect
    .poll(() => new URL(page.url()).searchParams.get("source"))
    .toBe("channel-beta");
  await expect(page.locator("#content-view article")).toContainText(
    "Beta transcript fixture",
    { timeout: READY_MS },
  );

  await page.goBack();
  await expect
    .poll(() => new URL(page.url()).searchParams.get("source"))
    .toBe("channel-alpha");
  await expect(page.locator("#content-view article")).toContainText(
    "Alpha transcript fixture",
    { timeout: READY_MS },
  );
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

  await dispatchVisibleClick(
    sidebar
      .locator("[data-channel-id]")
      .first()
      .getByRole("button", { name: "Overview test channel" }),
  );

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
  await openSeededWorkspace(
    page,
    "/?source=channel-alpha&item=video-alpha&content=summary",
  );

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
  const sidebar = await openSeededWorkspace(
    page,
    "/?source=channel-alpha&item=video-alpha&content=summary",
  );
  const videoButtons = sidebar.locator("#videos").getByRole("button");
  await expect(videoButtons.first()).toBeVisible({ timeout: READY_MS });
  const targetButton = videoButtons.first();
  const targetTitle = (
    await targetButton.locator("p.line-clamp-2").innerText()
  ).trim();

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

  const evalTrigger = page
    .locator("button[aria-controls='summary-quality-note']")
    .filter({ hasText: "Quality" });
  const evalDrawer = page.locator("#summary-quality-note");
  const tagList = evalDrawer.locator("[aria-label='Evaluation tags']");
  const note = evalDrawer.locator(".eval-note-markdown");

  await expect(evalTrigger).toBeVisible();
  await evalTrigger.click();

  await expect(evalDrawer).toBeVisible();
  await expect(evalTrigger).toHaveAttribute("aria-expanded", "true");
  await expect(note).toContainText(/\S+/);
  await expect(tagList).toBeVisible();
  const tagListTop = await tagList.evaluate(
    (node) => node.getBoundingClientRect().top,
  );
  const noteTop = await note.evaluate(
    (node) => node.getBoundingClientRect().top,
  );
  expect(tagListTop).toBeLessThan(noteTop);
});

test("desktop summary eval drawer still opens when only score and tags are present", async ({
  page,
}) => {
  const selectedChannelId = "channel-eval-tags";
  const selectedVideoId = "video-eval-tags";
  const selectedPath = `/?source=${selectedChannelId}&item=${selectedVideoId}&content=summary`;
  const bootstrap = buildMockWorkspaceBootstrap({
    channelId: selectedChannelId,
    channelName: "Tag-only eval channel",
    channelHandle: "@tag-only-eval",
    containerId: "container-eval-tags",
    videoId: selectedVideoId,
    videoTitle: "Desktop score-only eval fixture",
    qualityScore: 7,
    selectedItemId: selectedVideoId,
    totalChunkCount: 8,
  });

  const summary = {
    video_id: selectedVideoId,
    content:
      "This mocked summary exists only to verify that the eval drawer still opens without a note.",
    model_used: "glm-5.1:cloud",
    quality_score: 7,
    quality_note: null,
    quality_model_used: "gemma4:31b-cloud",
    summary_tags: ["AI Security", "Tech Knowledge"],
    summary_tags_evaluated: true,
  };

  const videoInfo = {
    video_id: selectedVideoId,
    watch_url: "https://www.youtube.com/watch?v=video-eval-tags",
    title: "Desktop score-only eval fixture",
    description:
      "Fixture video info for the score-only eval drawer regression.",
    thumbnail_url: null,
    channel_name: "Tag-only eval channel",
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
    "eval drawer still opens without a note",
  );

  const tagTrigger = page
    .locator("button[aria-controls='summary-quality-note']")
    .filter({ hasText: "2 tags" });
  const evalDrawer = page.locator("#summary-quality-note");

  await expect(page.locator("[aria-label='Summary tags']")).toHaveCount(0);
  await expect(tagTrigger).toBeVisible();
  await tagTrigger.click();

  await expect(evalDrawer).toBeVisible();
  await expect(tagTrigger).toHaveAttribute("aria-expanded", "true");
  await expect(evalDrawer.getByText("AI Security")).toBeVisible();
  await expect(evalDrawer.getByText("Tech Knowledge")).toBeVisible();
});
