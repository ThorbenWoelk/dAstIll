import { expect, test, type Locator, type Page } from "@playwright/test";

import { CHANNELS, openPaper, STORIES, summaryFor } from "./mock-backend";

const headline = (page: Page) => page.locator("#story-headline");
const pager = (page: Page) =>
  page.getByRole("navigation", { name: "Stories in this section" });
const tabBar = (page: Page) => page.getByRole("navigation", { name: "Pages" });
const readBar = (page: Page) => page.locator(".read-bar");
/** Unavailable turns are hidden, so they also leave the accessibility tree. */
const turnButton = (page: Page, name: "Previous story" | "Next story") =>
  page.getByRole("button", { name, includeHidden: true });

const LONG_STORIES = STORIES.map((story) => ({
  ...story,
  summary: `${summaryFor(story.title)}\n${"A long paragraph of detail. ".repeat(120)}`,
}));

/** A touch swipe across `target`: negative `distance` is to the left. */
async function swipe(target: Locator, distance: number) {
  const box = (await target.boundingBox())!;
  const start = box.x + box.width / 2;
  const touch = {
    pointerId: 7,
    pointerType: "touch",
    isPrimary: true,
    clientY: 300,
  };
  await target.dispatchEvent("pointerdown", { ...touch, clientX: start });
  for (let step = 1; step <= 8; step++) {
    await target.dispatchEvent("pointermove", {
      ...touch,
      clientX: start + (distance * step) / 8,
    });
  }
  await target.dispatchEvent("pointerup", {
    ...touch,
    clientX: start + distance,
  });
}

/** Scrolls in small steps, the way a reader does, rather than one jump. */
async function scrollBy(page: Page, distance: number) {
  for (let step = 0; step < 4; step++) {
    await page.mouse.wheel(0, distance / 4);
    await page.waitForTimeout(50);
  }
}

test("the pager and arrow keys browse a section without marking anything read", async ({
  page,
}) => {
  const backend = await openPaper(page, {
    channels: CHANNELS,
    stories: STORIES,
  });
  await expect(headline(page)).toHaveText(
    "Why small teams ship calmer software",
  );
  await expect(pager(page).getByText("1 of 3")).toBeVisible();
  await expect(turnButton(page, "Previous story")).toBeDisabled();
  await expect(turnButton(page, "Previous story")).toBeHidden();

  await page.getByRole("button", { name: "Next story" }).click();
  await expect(headline(page)).toHaveText("What sleep pressure actually is");
  await expect(pager(page).getByText("2 of 3")).toBeVisible();

  await page.keyboard.press("ArrowRight");
  await expect(headline(page)).toHaveText("Postgres was enough");
  await expect(turnButton(page, "Next story")).toBeDisabled();

  await page.keyboard.press("ArrowLeft");
  await expect(headline(page)).toHaveText("What sleep pressure actually is");
  await expect(page.getByText("3 stories left")).toBeVisible();
  expect(backend.readRequests).toEqual([]);
});

test("marking a story read keeps the reader's place in the section", async ({
  page,
}) => {
  await openPaper(page, { channels: CHANNELS, stories: STORIES });
  await page.getByRole("button", { name: "Next story" }).click();
  await expect(headline(page)).toHaveText("What sleep pressure actually is");

  await page.getByRole("button", { name: "Mark as read" }).click();
  await expect(headline(page)).toHaveText("Postgres was enough");
  await expect(pager(page).getByText("2 of 2")).toBeVisible();

  await page.keyboard.press("u");
  await expect(headline(page)).toHaveText("What sleep pressure actually is");
  await expect(pager(page).getByText("2 of 3")).toBeVisible();
});

test.describe("phone", () => {
  test.use({ viewport: { width: 375, height: 812 } });

  test("swiping turns between stories and stops at the ends", async ({
    page,
  }) => {
    const backend = await openPaper(page, {
      channels: CHANNELS,
      stories: STORIES,
    });
    const article = page.locator("article");
    await expect(headline(page)).toHaveText(
      "Why small teams ship calmer software",
    );

    await swipe(article, -160);
    await expect(headline(page)).toHaveText("What sleep pressure actually is");
    await swipe(article, -160);
    await expect(headline(page)).toHaveText("Postgres was enough");

    // Nothing after the last story: the swipe settles back.
    await swipe(article, -160);
    await page.waitForTimeout(400);
    await expect(headline(page)).toHaveText("Postgres was enough");

    await swipe(article, 160);
    await expect(headline(page)).toHaveText("What sleep pressure actually is");

    // A short, slow swipe is not a turn.
    await swipe(article, 30);
    await page.waitForTimeout(400);
    await expect(headline(page)).toHaveText("What sleep pressure actually is");
    await expect(article).not.toHaveAttribute("style", /translate/);
    expect(backend.readRequests).toEqual([]);
  });

  test("the tab bar sits at the bottom and tucks away while reading down", async ({
    page,
  }) => {
    await openPaper(page, { channels: CHANNELS, stories: LONG_STORIES });
    await expect(headline(page)).toBeVisible();
    const tabs = tabBar(page);
    await expect(
      tabs.getByRole("link", { name: "Front page" }),
    ).toHaveAttribute("aria-current", "page");
    const tabBox = (await tabs.boundingBox())!;
    expect(tabBox.y + tabBox.height).toBeCloseTo(812, 0);
    // Measured once the story's print-in animation has settled.
    await expect
      .poll(async () => {
        const box = (await readBar(page).boundingBox())!;
        return box.y + box.height <= tabBox.y + 1;
      })
      .toBe(true);

    await scrollBy(page, 1200);
    await expect(tabs).not.toBeInViewport();
    await expect(
      page.getByRole("navigation", { name: "Sections" }),
    ).not.toBeInViewport();
    await expect
      .poll(async () => {
        const box = (await readBar(page).boundingBox())!;
        return Math.round(box.y + box.height);
      })
      .toBe(812);

    await scrollBy(page, -200);
    await expect(tabs).toBeInViewport();
    // The section bar comes back stuck to the top of the screen.
    await expect
      .poll(
        async () =>
          (await page
            .getByRole("navigation", { name: "Sections" })
            .boundingBox())!.y,
      )
      .toBe(0);
  });

  test("tapping a tab opens that page", async ({ page }) => {
    await openPaper(page, { channels: CHANNELS, stories: STORIES });
    await tabBar(page).getByRole("link", { name: "Finished" }).click();
    await expect(page.getByRole("heading", { name: "Finished" })).toBeVisible();
    await expect(
      tabBar(page).getByRole("link", { name: "Finished" }),
    ).toHaveAttribute("aria-current", "page");
  });

  test("a story opened from a list goes back by button or swipe", async ({
    page,
  }) => {
    await openPaper(
      page,
      { channels: CHANNELS, stories: STORIES, read: ["v-mid"] },
      { path: "/finished" },
    );
    const story = page.getByRole("link", {
      name: "What sleep pressure actually is",
    });

    await story.click();
    await expect(headline(page)).toHaveText("What sleep pressure actually is");
    await page.getByRole("button", { name: "Back to Finished" }).click();
    await expect(page.getByRole("heading", { name: "Finished" })).toBeVisible();

    await story.click();
    await expect(headline(page)).toHaveText("What sleep pressure actually is");
    await swipe(page.locator("article"), 160);
    await expect(page.getByRole("heading", { name: "Finished" })).toBeVisible();
  });
});

test.describe("desktop", () => {
  test.use({ viewport: { width: 1280, height: 900 } });

  test("the page links sit in the dateline, not along the bottom", async ({
    page,
  }) => {
    await openPaper(page, { channels: CHANNELS, stories: STORIES });
    await expect(headline(page)).toBeVisible();
    const box = (await tabBar(page).boundingBox())!;
    expect(box.y).toBeLessThan(100);
    // The front page does not link to itself.
    await expect(
      tabBar(page).getByRole("link", { name: "Front page" }),
    ).toBeHidden();
  });
});
