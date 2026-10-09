import { expect, test, type Page } from "@playwright/test";

import {
  CHANNELS,
  openPaper,
  STORIES,
  type MockHighlight,
} from "./mock-backend";

const headline = (page: Page) => page.locator("#story-headline");
const marks = (page: Page) => page.locator("article mark.reader-highlight");

/** Selects the first occurrence of `needle` inside the article. */
async function selectInArticle(page: Page, needle: string) {
  await expect(page.locator("article")).toContainText(needle);
  await page.evaluate((text) => {
    const article = document.querySelector("article")!;
    const walker = document.createTreeWalker(article, NodeFilter.SHOW_TEXT);
    for (let node = walker.nextNode(); node; node = walker.nextNode()) {
      const at = node.textContent!.indexOf(text);
      if (at < 0) continue;
      const range = document.createRange();
      range.setStart(node, at);
      range.setEnd(node, at + text.length);
      const selection = document.getSelection()!;
      selection.removeAllRanges();
      selection.addRange(range);
      return;
    }
    throw new Error(`"${text}" is not in the article`);
  }, needle);
}

function savedHighlight(
  overrides: Partial<MockHighlight> & Pick<MockHighlight, "video_id" | "text">,
): MockHighlight {
  return {
    id: "90071992547409931",
    source: "summary",
    prefix_context: "",
    suffix_context: "",
    created_at: "2026-10-06T09:00:00Z",
    ...overrides,
  };
}

test("selected text becomes a highlight that stays after a reload", async ({
  page,
}) => {
  const backend = await openPaper(page, {
    channels: CHANNELS,
    stories: STORIES,
  });
  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );

  await selectInArticle(page, "Detail about Why small teams");
  await page.getByRole("button", { name: "Highlight" }).click();

  await expect(marks(page)).toHaveText(["Detail about Why small teams"]);
  await expect.poll(() => backend.highlights.length).toBe(1);
  expect(backend.highlights[0]).toMatchObject({
    video_id: "v-new",
    source: "summary",
    text: "Detail about Why small teams",
    prefix_context: expect.stringContaining("First idea: "),
    suffix_context: expect.stringMatching(/^ ship calmer software/),
  });

  await page.reload();
  await expect(marks(page)).toHaveText(["Detail about Why small teams"]);

  await marks(page).click();
  await page.getByRole("button", { name: "Remove highlight" }).click();
  await expect(marks(page)).toHaveCount(0);
  // Ids beyond JavaScript's safe integers must survive the round trip.
  expect(backend.deletedHighlightIds).toEqual(["90071992547409930"]);
});

test("tapping Highlight works even when the tap clears the selection", async ({
  page,
}) => {
  const backend = await openPaper(page, {
    channels: CHANNELS,
    stories: STORIES,
  });
  await expect(headline(page)).toBeVisible();
  await selectInArticle(page, "Detail about Why small teams");
  const button = page.getByRole("button", { name: "Highlight" });
  await expect(button).toBeVisible();

  // Phones drop the selection on touch, before the click is delivered.
  await button.dispatchEvent("pointerdown");
  await page.evaluate(() => document.getSelection()!.removeAllRanges());
  await page.waitForTimeout(100);
  await button.dispatchEvent("pointerup");
  await button.click();

  await expect(marks(page)).toHaveText(["Detail about Why small teams"]);
  await expect.poll(() => backend.highlights.length).toBe(1);
});

test("a highlight can span paragraphs", async ({ page }) => {
  const backend = await openPaper(page, {
    channels: CHANNELS,
    stories: STORIES,
  });
  await expect(headline(page)).toBeVisible();

  await page.evaluate(() => {
    const items = document.querySelectorAll("article .body li");
    const range = document.createRange();
    range.setStart(items[0].lastChild!, 0);
    range.setEnd(items[1].firstChild!, 1);
    document.getSelection()!.removeAllRanges();
    document.getSelection()!.addRange(range);
  });
  await page.getByRole("button", { name: "Highlight" }).click();

  await expect.poll(() => backend.highlights.length).toBe(1);
  expect(backend.highlights[0].text).toContain("\n");
  await expect(marks(page)).toHaveCount(2);
});

test("double-clicking a word offers to highlight it", async ({ page }) => {
  const backend = await openPaper(page, {
    channels: CHANNELS,
    stories: STORIES,
  });
  await page.locator("article .standfirst").dblclick();
  await page.getByRole("button", { name: "Highlight" }).click();
  await expect(marks(page)).toHaveCount(1);
  await expect.poll(() => backend.highlights.length).toBe(1);
  await expect(marks(page)).toHaveText(backend.highlights[0].text);
});

test("the highlights page lists, searches, opens, and removes highlights", async ({
  page,
}) => {
  const backend = await openPaper(
    page,
    {
      channels: CHANNELS,
      stories: STORIES,
      read: ["v-mid"],
      highlights: [
        savedHighlight({
          id: "90071992547409931",
          video_id: "v-mid",
          text: "Detail about What sleep pressure actually is",
          prefix_context: "First idea: ",
        }),
        savedHighlight({
          id: "90071992547409932",
          video_id: "v-old",
          text: "Standfirst for Postgres was enough",
        }),
        savedHighlight({
          id: "90071992547409933",
          video_id: "v-new",
          text: "Something to remember.",
        }),
      ],
    },
    { path: "/highlights" },
  );

  await expect(page.getByRole("heading", { name: "Highlights" })).toBeVisible();
  await expect(page.getByText("3 highlights")).toBeVisible();
  // Newest story first, whatever channel it belongs to.
  await expect(page.locator(".highlights-page h2")).toHaveText([
    "Why small teams ship calmer software",
    "What sleep pressure actually is",
    "Postgres was enough",
  ]);

  await page.getByLabel("Search highlights").fill("sleep");
  await expect(page.locator("blockquote")).toHaveText([
    "Detail about What sleep pressure actually is",
  ]);
  await expect(page.getByText("1 of 3 highlights")).toBeVisible();
  await page.getByLabel("Search highlights").fill("nothing like this");
  await expect(
    page.getByText('No highlight matches "nothing like this".'),
  ).toBeVisible();
  await page.getByLabel("Search highlights").fill("");

  // A finished story opens on its own page, highlight in place.
  await page
    .getByRole("link", { name: "What sleep pressure actually is" })
    .click();
  await expect(headline(page)).toHaveText("What sleep pressure actually is");
  await expect(marks(page)).toHaveText([
    "Detail about What sleep pressure actually is",
  ]);
  await expect(page.getByText("You finished this story.")).toBeVisible();

  await page.goBack();
  await page
    .getByRole("button", { name: /Remove highlight: Standfirst for Postgres/ })
    .click();
  await expect(page.locator("blockquote")).toHaveCount(2);
  expect(backend.deletedHighlightIds).toEqual(["90071992547409932"]);
});

test("finished stories can be reread and marked unread", async ({ page }) => {
  const backend = await openPaper(page, {
    channels: CHANNELS,
    stories: STORIES,
  });
  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );
  await page.getByRole("button", { name: "Mark as read" }).click();
  await expect(headline(page)).toHaveText("What sleep pressure actually is");

  await page.getByRole("link", { name: "Finished" }).click();
  await expect(page.getByRole("heading", { name: "Finished" })).toBeVisible();
  await expect(page.getByRole("heading", { name: "Today" })).toBeVisible();
  await page
    .getByRole("link", { name: "Why small teams ship calmer software" })
    .click();

  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );
  await page.getByRole("button", { name: "Mark as unread" }).click();
  await expect(
    page.getByRole("button", { name: "Mark as read" }),
  ).toBeVisible();
  await expect
    .poll(() => backend.readRequests)
    .toEqual([
      { id: "v-new", read: true },
      { id: "v-new", read: false },
    ]);

  await page.getByRole("link", { name: "Front page" }).click();
  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );
});

test("an unknown story says so", async ({ page }) => {
  await openPaper(
    page,
    { channels: CHANNELS, stories: STORIES },
    { path: "/stories/not-a-story" },
  );
  await expect(
    page.getByRole("heading", { name: "This story is not in your paper." }),
  ).toBeVisible();
});

test.describe("phone", () => {
  test.use({ viewport: { width: 375, height: 812 } });

  test("the masthead links fit without sideways scrolling", async ({
    page,
  }) => {
    await openPaper(page, { channels: CHANNELS, stories: STORIES });
    await expect(headline(page)).toBeVisible();
    for (const name of ["Highlights", "Finished", "Sections"]) {
      await expect(
        page
          .getByRole("navigation", { name: "Pages" })
          .getByRole("link", { name }),
      ).toBeInViewport();
    }
    const overflow = await page.evaluate(
      () => document.documentElement.scrollWidth - window.innerWidth,
    );
    expect(overflow).toBeLessThanOrEqual(0);
  });
});
