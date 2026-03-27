import { expect, test, type Page } from "@playwright/test";

const READY_MS = 120_000;

function workspaceSidebar(page: Page) {
  // Two aside#workspace nodes can exist (desktop rail + mobile browse dialog). Exclude the dialog copy.
  return page
    .locator('xpath=//aside[@id="workspace"][not(ancestor::*[@role="dialog"])]')
    .first();
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

test.beforeEach(async ({ context }) => {
  await context.addInitScript(() => {
    try {
      localStorage.clear();
    } catch {
      /* ignore */
    }
  });
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

  await page.getByRole("button", { name: "Transcript", exact: true }).click();
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  await expect(page.locator("#content-view article")).not.toBeEmpty();

  await page.getByRole("button", { name: "Info", exact: true }).click();
  await expect(page.getByText("Published").first()).toBeVisible({
    timeout: READY_MS,
  });
  await expect(page.locator("#content-view article")).toHaveCount(0);

  await page.getByRole("button", { name: "Summary", exact: true }).click();
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
  await page.getByRole("button", { name: "Transcript", exact: true }).click();
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  const transcriptA = (
    await page.locator("#content-view article").innerText()
  ).trim();

  await selectChannelAndFirstVideo(1);
  await page.getByRole("button", { name: "Transcript", exact: true }).click();
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  const transcriptB = (
    await page.locator("#content-view article").innerText()
  ).trim();
  expect(transcriptB.length).toBeGreaterThan(0);
  expect(transcriptB).not.toBe(transcriptA);

  await page.getByRole("button", { name: "Summary", exact: true }).click();
  await expect(page.locator("#content-view article")).toBeVisible({
    timeout: READY_MS,
  });
  const summaryB = (
    await page.locator("#content-view article").innerText()
  ).trim();
  expect(summaryB.length).toBeGreaterThan(0);
  expect(summaryB).not.toBe(transcriptA);
});

test("G then W navigates from queue to workspace without full reload hang", async ({
  page,
}) => {
  await page.goto("/download-queue");
  await expect
    .poll(() => new URL(page.url()).pathname)
    .toContain("download-queue");

  await page.keyboard.press("g");
  await page.keyboard.press("w");

  await expect.poll(() => new URL(page.url()).pathname).toBe("/");
  await expect(page.locator("#workspace")).toBeVisible({ timeout: READY_MS });
});

test("mark read toggle flips aria-pressed on desktop", async ({ page }) => {
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
  await expect(toggle).toHaveAttribute(
    "aria-pressed",
    before === "true" ? "false" : "true",
  );
});

test("unread filter keeps unread videos and removes them after marking read", async ({
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
  await expect(toggle).toHaveAttribute("aria-label", "Mark as unread");
  await expect(
    sidebar.locator("#videos").getByText(targetTitle, { exact: true }),
  ).toHaveCount(0);
});
