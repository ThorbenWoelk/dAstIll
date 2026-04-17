import { expect, test, type Page } from "@playwright/test";
import { resetClientState } from "./test-helpers";

const READY_MS = 120_000;

function miniSummary(index: number) {
  const id = `mini-video-${index}`;
  return {
    video_id: id,
    channel_id: "mini-channel",
    channel_name: "Mini Dispatch",
    title: `Mini reader fixture ${index}`,
    thumbnail_url: null,
    published_at: "2026-04-16T08:00:00.000Z",
    watch_url: `https://www.youtube.com/watch?v=${id}`,
    summary_content: [
      `# Mini reader fixture ${index}`,
      "",
      "This summary exists to prove that the mini reader keeps one component tree across screen sizes.",
      "",
      "## First section",
      "",
      "The desktop view should turn the summary rail into a sidebar while the mobile view keeps it above the article.",
      "",
      "## Second section",
      "",
      "The article pane owns vertical scrolling so the surrounding shell can keep its chrome steady.",
    ].join("\n"),
    read: false,
  };
}

async function installMiniApi(
  page: Page,
  options: { onMiniRequest?: () => void } = {},
) {
  await page.route("**/api/**", async (route) => {
    const url = new URL(route.request().url());

    if (url.pathname === "/api/mini") {
      options.onMiniRequest?.();
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          channels: [
            {
              id: "mini-channel",
              handle: "@mini",
              name: "Mini Dispatch",
              thumbnail_url: null,
              added_at: "2026-04-16T08:00:00.000Z",
              earliest_sync_date: null,
              earliest_sync_date_user_set: false,
            },
          ],
          selected_channel_id: "mini-channel",
          summaries: Array.from({ length: 24 }, (_, index) =>
            miniSummary(index + 1),
          ),
        }),
      });
      return;
    }

    if (/^\/api\/videos\/[^/]+\/highlights$/.test(url.pathname)) {
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: "[]",
      });
      return;
    }

    if (/^\/api\/mini\/videos\/[^/]+\/read$/.test(url.pathname)) {
      const videoId = url.pathname.split("/").at(-2) ?? "mini-video-1";
      await route.fulfill({
        status: 200,
        contentType: "application/json",
        body: JSON.stringify({
          video_id: videoId,
          read: true,
          updated_at: "2026-04-16T09:00:00.000Z",
        }),
      });
      return;
    }

    await route.fulfill({ status: 404, body: "Unhandled test API route" });
  });
}

async function openMini(
  page: Page,
  viewport: { width: number; height: number },
  apiOptions: { onMiniRequest?: () => void } = {},
) {
  await page.setViewportSize(viewport);
  await resetClientState(page);
  await page.addInitScript(() => {
    window.localStorage.setItem(
      "__dastill_e2e_auth",
      JSON.stringify({
        userId: "mini-e2e-user",
        email: "mini-e2e@example.com",
        token: "mini-e2e-token",
      }),
    );
  });
  await installMiniApi(page, apiOptions);
  await page.goto("/mini");
  await expect(
    page
      .locator(".reader-article > header")
      .getByRole("heading", { name: "Mini reader fixture 1" }),
  ).toBeVisible({ timeout: READY_MS });
}

test("mini reader keeps mobile and desktop layouts responsive", async ({
  page,
}) => {
  await openMini(page, { width: 375, height: 812 });

  const bottomBar = page.locator(".bottom-bar");
  const strip = page.locator(".strip");
  const articlePane = page.locator(".mini-article-pane");

  await expect(bottomBar).toBeVisible();
  await expect(strip).toHaveCSS("flex-direction", "row");

  await articlePane.evaluate((node) => {
    node.scrollTop = 180;
    node.dispatchEvent(new Event("scroll", { bubbles: true }));
  });
  await expect(
    bottomBar.getByRole("button", { name: "Mark read and advance" }),
  ).toBeVisible();

  await openMini(page, { width: 1280, height: 900 });

  await expect(bottomBar).toBeHidden();
  await expect(strip).toHaveCSS("flex-direction", "column");
  await expect(
    page.getByRole("button", { name: "Change channel" }),
  ).toBeVisible();

  const sidebarMetrics = await strip.evaluate((node) => ({
    clientHeight: node.clientHeight,
    scrollHeight: node.scrollHeight,
    overflowY: getComputedStyle(node).overflowY,
  }));
  expect(sidebarMetrics.scrollHeight).toBeGreaterThan(
    sidebarMetrics.clientHeight,
  );
  expect(sidebarMetrics.overflowY).toBe("auto");
});

test("mini reader reloads when pulled from the top", async ({ page }) => {
  let miniRequests = 0;
  await openMini(
    page,
    { width: 375, height: 812 },
    {
      onMiniRequest: () => {
        miniRequests += 1;
      },
    },
  );
  expect(miniRequests).toBe(1);

  await page.locator(".mini-article-pane").evaluate((node) => {
    node.scrollTop = 0;
    const dispatchTouch = (
      type: "touchstart" | "touchmove" | "touchend",
      y: number,
    ) => {
      const event = new Event(type, { bubbles: true, cancelable: true });
      const touch = {
        clientX: 180,
        clientY: y,
        target: node,
      };
      Object.defineProperty(event, "touches", {
        value: type === "touchend" ? [] : [touch],
      });
      Object.defineProperty(event, "changedTouches", {
        value: [touch],
      });
      node.dispatchEvent(event);
    };

    dispatchTouch("touchstart", 120);
    dispatchTouch("touchmove", 210);
    dispatchTouch("touchend", 210);
  });

  await expect.poll(() => miniRequests).toBe(2);
});
