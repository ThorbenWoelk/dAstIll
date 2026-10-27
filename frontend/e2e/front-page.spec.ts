import { expect, test, type Page } from "@playwright/test";

import { CHANNELS, openPaper, STORIES, type MockStory } from "./mock-backend";

const headline = (page: Page) => page.locator("#story-headline");
const leadOnFrontPage = (page: Page) => page.locator("#front-lead-headline");

/** Six stories, so the grid has a lead, three beside it, and two below. */
const EDITION: MockStory[] = [
  ...STORIES,
  {
    id: "v-4",
    channelId: "science",
    title: "How tides keep time",
    publishedAt: "2026-10-01T07:00:00Z",
  },
  {
    id: "v-5",
    channelId: "build",
    title: "Deleting code on purpose",
    publishedAt: "2026-09-30T07:00:00Z",
  },
  {
    id: "v-6",
    channelId: "science",
    title: "The trouble with averages",
    publishedAt: "2026-09-29T07:00:00Z",
  },
];

test.describe("wide screen", () => {
  test.use({ viewport: { width: 1440, height: 900 } });

  test("the front page opens on a grid of the whole edition", async ({
    page,
  }) => {
    await openPaper(page, { channels: CHANNELS, stories: EDITION });

    await expect(leadOnFrontPage(page)).toHaveText(
      "Why small teams ship calmer software",
    );
    await expect(
      page
        .locator(".front-page")
        .getByText("Standfirst for Why small teams ship calmer software"),
    ).toBeVisible();
    await expect(
      page
        .getByRole("list", { name: "Next in this edition" })
        .getByRole("button"),
    ).toHaveText([
      /What sleep pressure actually is/,
      /Postgres was enough/,
      /How tides keep time/,
    ]);
    const more = page.getByRole("region", { name: "More in this edition" });
    await expect(more.getByRole("button")).toHaveText([
      /Deleting code on purpose/,
      /The trouble with averages/,
    ]);
    // The more grid runs four columns across.
    await expect(more.locator("ol")).toHaveCSS(
      "grid-template-columns",
      /^(\S+ ){3}\S+$/,
    );

    // The reader waits until a story is opened.
    await expect(headline(page)).toBeHidden();
    await expect(
      page.getByRole("complementary", { name: "Also in this edition" }),
    ).toBeHidden();
  });

  test("a story opened from the grid reads between the sections and the margin", async ({
    page,
  }) => {
    await openPaper(page, { channels: CHANNELS, stories: EDITION });
    await page
      .getByRole("button", { name: /What sleep pressure actually is/ })
      .click();

    await expect(headline(page)).toHaveText("What sleep pressure actually is");
    await expect(leadOnFrontPage(page)).toBeHidden();

    const sections = page.getByRole("navigation", { name: "Sections" });
    await expect(sections).toHaveCount(1);
    const margin = page.getByRole("complementary", { name: "In the margin" });
    await expect(
      margin
        .getByRole("navigation", { name: "In this story" })
        .getByRole("button"),
    ).toHaveText(["At a glance", "Key Points", "Takeaways"]);
    await expect(
      page.getByRole("complementary", { name: "Also in this edition" }),
    ).toBeHidden();

    // Sections on the left, the story in the middle, the margin on the right.
    const sectionsBox = await sections.boundingBox();
    const articleBox = await page.getByRole("article").boundingBox();
    const marginBox = await margin.boundingBox();
    expect(sectionsBox!.x + sectionsBox!.width).toBeLessThan(articleBox!.x);
    expect(marginBox!.x).toBeGreaterThan(articleBox!.x + articleBox!.width);

    await page.getByRole("button", { name: "Front page", exact: true }).click();
    await expect(leadOnFrontPage(page)).toBeVisible();
    await expect(headline(page)).toBeHidden();

    // The browser's own Back returns to the grid as well.
    await page.getByRole("button", { name: "Read the story" }).click();
    await expect(headline(page)).toHaveText("What sleep pressure actually is");
    await page.goBack();
    await expect(leadOnFrontPage(page)).toBeVisible();
  });

  test("the margin lists the story's highlights and removes them", async ({
    page,
  }) => {
    const backend = await openPaper(page, {
      channels: CHANNELS,
      stories: EDITION,
      highlights: [
        {
          id: "90071992547409931",
          video_id: "v-new",
          source: "summary",
          text: "Detail about Why small teams",
          prefix_context: "First idea: ",
          suffix_context: " ship calmer software.",
          created_at: "2026-10-06T09:00:00Z",
        },
      ],
    });
    await page.getByRole("button", { name: "Read the story" }).click();

    const margin = page.getByRole("complementary", { name: "In the margin" });
    await expect(margin.locator("mark")).toHaveText([
      "Detail about Why small teams",
    ]);
    await margin
      .getByRole("button", { name: "Remove highlight: Detail about" })
      .click();
    await expect(margin.locator("mark")).toHaveCount(0);
    await expect(
      margin.getByText("Select text in the story to highlight it."),
    ).toBeVisible();
    await expect
      .poll(() => backend.deletedHighlightIds)
      .toEqual(["90071992547409931"]);
  });

  test("marking the grid's lead read brings the next story up", async ({
    page,
  }) => {
    const backend = await openPaper(page, {
      channels: CHANNELS,
      stories: EDITION,
    });
    await expect(leadOnFrontPage(page)).toHaveText(
      "Why small teams ship calmer software",
    );

    await page.getByRole("button", { name: "Mark as read" }).click();
    await expect(leadOnFrontPage(page)).toHaveText(
      "What sleep pressure actually is",
    );
    await expect
      .poll(() => backend.readRequests)
      .toEqual([{ id: "v-new", read: true }]);

    await page.getByRole("button", { name: /^Undo/ }).click();
    await expect(leadOnFrontPage(page)).toHaveText(
      "Why small teams ship calmer software",
    );
  });
});

test.describe("phone", () => {
  test.use({ viewport: { width: 375, height: 812 } });

  test("the front page still leads with the story, not the grid", async ({
    page,
  }) => {
    await openPaper(page, { channels: CHANNELS, stories: EDITION });
    await expect(headline(page)).toHaveText(
      "Why small teams ship calmer software",
    );
    await expect(leadOnFrontPage(page)).toBeHidden();
    await expect(
      page.getByRole("complementary", { name: "In the margin" }),
    ).toBeHidden();
  });
});
