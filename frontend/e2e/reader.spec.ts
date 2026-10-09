import { expect, test } from "@playwright/test";

import { CHANNELS, openPaper, STORIES, summaryFor } from "./mock-backend";

const headline = (page: import("@playwright/test").Page) =>
  page.locator("#story-headline");

test("signed-out readers see the sign-in page", async ({ page }) => {
  await openPaper(
    page,
    { channels: CHANNELS, stories: STORIES },
    { signedIn: false },
  );
  await expect(
    page.getByRole("button", { name: "Sign in with Google" }),
  ).toBeVisible();
  await expect(headline(page)).toHaveCount(0);
});

test("front page leads with the newest story from all channels", async ({
  page,
}) => {
  const backend = await openPaper(page, {
    channels: CHANNELS,
    stories: STORIES,
  });

  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );
  await expect(page.locator(".kicker")).toHaveText("The Long Build");
  await expect(
    page.getByText("Standfirst for Why small teams ship calmer software"),
  ).toBeVisible();
  await expect(
    page.getByRole("heading", { name: "At a glance" }),
  ).toBeVisible();
  await expect(page.getByText("3 stories left")).toBeVisible();

  const rail = page.getByRole("complementary", {
    name: "Also in this edition",
  });
  await expect(rail.getByRole("button")).toHaveText([
    /What sleep pressure actually is/,
    /Postgres was enough/,
  ]);

  const nav = page.getByRole("navigation", { name: "Sections" });
  await expect(nav.getByRole("button", { name: /Front page/ })).toHaveAttribute(
    "aria-current",
    "page",
  );
  await expect(
    nav.getByRole("button", { name: "Slow Science 1" }),
  ).toBeVisible();
  await expect(
    nav.getByRole("button", { name: "The Long Build 2" }),
  ).toBeVisible();

  // One request per channel: the first answers for the first channel.
  expect(backend.miniRequests.sort()).toEqual([null, "build"].sort());
});

test("marking a story read shows the next one and undo brings it back", async ({
  page,
}) => {
  const backend = await openPaper(page, {
    channels: CHANNELS,
    stories: STORIES,
  });
  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );

  await page.getByRole("button", { name: "Mark as read" }).click();
  await expect(headline(page)).toHaveText("What sleep pressure actually is");
  await expect(page.getByText("2 stories left")).toBeVisible();
  await expect
    .poll(() => backend.readRequests)
    .toEqual([{ id: "v-new", read: true }]);

  await page
    .getByRole("button", { name: /Undo: mark "Why small teams/ })
    .click();
  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );
  await expect
    .poll(() => backend.readRequests)
    .toEqual([
      { id: "v-new", read: true },
      { id: "v-new", read: false },
    ]);
});

test("the r key marks the lead story read", async ({ page }) => {
  await openPaper(page, { channels: CHANNELS, stories: STORIES });
  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );
  await page.keyboard.press("r");
  await expect(headline(page)).toHaveText("What sleep pressure actually is");
});

test("a story picked from the rail becomes the lead", async ({ page }) => {
  await openPaper(page, { channels: CHANNELS, stories: STORIES });
  await page
    .getByRole("complementary", { name: "Also in this edition" })
    .getByRole("button", { name: /Postgres was enough/ })
    .click();
  await expect(headline(page)).toHaveText("Postgres was enough");
});

test("a section shows one channel and ends with a way back", async ({
  page,
}) => {
  await openPaper(page, { channels: CHANNELS, stories: STORIES });
  await page
    .getByRole("navigation", { name: "Sections" })
    .getByRole("button", { name: "Slow Science 1" })
    .click();

  await expect(headline(page)).toHaveText("What sleep pressure actually is");
  await expect(
    page.getByText("This is the last story in this section."),
  ).toBeVisible();

  await page.getByRole("button", { name: "Mark as read" }).click();
  await expect(
    page.getByRole("heading", { name: "That is the whole edition." }),
  ).toBeVisible();
  await page.getByRole("button", { name: "Front page (2)" }).click();
  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );
});

test("summary HTML from the model is sanitized", async ({ page }) => {
  await openPaper(page, {
    channels: CHANNELS,
    stories: [
      {
        id: "v-evil",
        channelId: "science",
        title: "Untrusted summary",
        publishedAt: "2026-10-06T07:00:00Z",
        summary: `${summaryFor("x")}\n<script>window.__pwned = true</script>\n<img src=x onerror="window.__pwned = true">\n<a href="https://example.com" onclick="window.__pwned = true">raw link</a>\n\n[a link](https://example.com) and [bad link](javascript:window.__pwned=true)\n\n![pic](https://example.com/x.png)`,
      },
    ],
  });
  const article = page.locator("article");
  await expect(headline(page)).toHaveText("Untrusted summary");
  await expect(article.locator("script")).toHaveCount(0);
  await expect(article.locator("img")).toHaveCount(0);
  await expect(article.locator("[onclick]")).toHaveCount(0);
  // Raw HTML in the markdown shows as text instead of becoming elements.
  await expect(
    article.getByText("<script>window.__pwned = true"),
  ).toBeVisible();
  await expect(article.getByRole("link", { name: "raw link" })).toHaveCount(0);
  await expect(article.getByRole("link", { name: "a link" })).toHaveAttribute(
    "target",
    "_blank",
  );
  await expect(article.getByRole("link", { name: "a link" })).toHaveAttribute(
    "rel",
    "noopener noreferrer",
  );
  await expect(article.locator('a[href^="javascript:"]')).toHaveCount(0);
  expect(
    await page.evaluate(() => (window as { __pwned?: boolean }).__pwned),
  ).toBeUndefined();
});

test("a TL;DR summary with tags and timestamps in prose reads like the standard one", async ({
  page,
}) => {
  await openPaper(page, {
    channels: CHANNELS,
    stories: [
      {
        id: "v-tldr",
        channelId: "build",
        title: "Components in prose",
        publishedAt: "2026-10-06T07:00:00Z",
        summary: [
          "Let me analyze this transcript first.",
          "",
          "## TL;DR",
          "- Wrap the page in <PricingTable /> once.",
          "",
          "## Overview",
          "In this video, Dr. Rivera builds a pricing page.",
          "",
          "## Key Points",
          "[0:00]: Intro",
          "",
          "[12:34] The <command_name> placeholder is explained.",
        ].join("\n"),
      },
    ],
  });
  const article = page.locator("article");
  await expect(headline(page)).toHaveText("Components in prose");
  await expect(
    article.getByText("In this video, Dr. Rivera builds a pricing page."),
  ).toBeVisible();
  const glance = article.getByRole("complementary", { name: "At a glance" });
  await expect(glance).toContainText("Wrap the page in <PricingTable /> once.");
  const body = article.locator(".body");
  await expect(body).not.toContainText("TL;DR");
  await expect(body).not.toContainText("Let me analyze");
  await expect(body).toContainText("[0:00]: Intro");
  await expect(body).toContainText(
    "[12:34] The <command_name> placeholder is explained.",
  );
  await expect(body.getByRole("link")).toHaveCount(0);
});

test("a reader without channels is sent to add one", async ({ page }) => {
  await openPaper(page, { channels: [], stories: [] });
  await expect(
    page.getByRole("heading", { name: "Your paper has no sections yet." }),
  ).toBeVisible();
  await page.getByRole("link", { name: "Add a channel" }).click();
  await expect(
    page.getByRole("heading", { name: "Sections", exact: true }),
  ).toBeVisible();
});

test("sections page adds and removes channels", async ({ page }) => {
  await openPaper(page, { channels: CHANNELS, stories: STORIES });
  await page
    .getByRole("navigation", { name: "Pages" })
    .getByRole("link", { name: "Sections" })
    .click();

  await page.getByLabel("Add a channel").fill("@newchannel");
  await page.getByRole("button", { name: "Add", exact: true }).click();
  await expect(page.getByText("Added newchannel.")).toBeVisible();
  await expect(page.locator(".channel-list .name")).toHaveText([
    "newchannel",
    "Slow Science",
    "The Long Build",
  ]);

  await page.getByRole("button", { name: "Remove Slow Science" }).click();
  await page.getByRole("button", { name: "Remove", exact: true }).click();
  await expect(page.locator(".channel-list .name")).toHaveText([
    "newchannel",
    "The Long Build",
  ]);
});

test.describe("phone", () => {
  test.use({ viewport: { width: 375, height: 812 } });

  test("the read bar stays on screen while reading and the rail follows the article", async ({
    page,
  }) => {
    await openPaper(page, {
      channels: CHANNELS,
      stories: STORIES.map((story) => ({
        ...story,
        summary: `${summaryFor(story.title)}\n${"A long paragraph of detail. ".repeat(120)}`,
      })),
    });
    const readButton = page.getByRole("button", { name: "Mark as read" });
    await expect(readButton).toBeInViewport();
    await page.mouse.wheel(0, 1500);
    await expect(readButton).toBeInViewport();

    const articleBox = await page.locator("article").boundingBox();
    const railBox = await page
      .getByRole("complementary", { name: "Also in this edition" })
      .boundingBox();
    // Stacked: same left edge, rail below the article's top.
    expect(Math.abs(railBox!.x - articleBox!.x)).toBeLessThan(2);
    expect(railBox!.y).toBeGreaterThan(articleBox!.y + articleBox!.height / 2);
  });
});

test.describe("desktop", () => {
  test.use({ viewport: { width: 1280, height: 900 } });

  test("the rail sits beside the article and the body is set in two columns", async ({
    page,
  }) => {
    await openPaper(page, { channels: CHANNELS, stories: STORIES });
    await expect(headline(page)).toBeVisible();
    await expect
      .poll(async () => {
        const articleBox = await page.locator("article").boundingBox();
        const railBox = await page
          .getByRole("complementary", { name: "Also in this edition" })
          .boundingBox();
        return articleBox && railBox
          ? railBox.x > articleBox.x + articleBox.width - 1
          : false;
      })
      .toBe(true);
    await expect(page.locator(".body")).toHaveCSS("column-count", "2");
  });
});
