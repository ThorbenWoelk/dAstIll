import { expect, test, type Page } from "@playwright/test";

import { CHANNELS, openPaper, STORIES } from "./mock-backend";
import { selectInArticle } from "./selection";

const headline = (page: Page) => page.locator("#story-headline");
const flagMarks = (page: Page) => page.locator("article mark.writing-flag");
const noteField = (page: Page) =>
  page.getByRole("textbox", { name: "What is wrong with it?" });

test("a flagged passage keeps its note and can be removed", async ({
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
  await page.getByRole("button", { name: "Flag writing" }).click();

  const save = page.getByRole("button", { name: "Flag writing" });
  await expect(noteField(page)).toBeFocused();
  await expect(save).toBeDisabled();
  await noteField(page).fill("Vague: says nothing concrete");
  await save.click();

  await expect(flagMarks(page)).toHaveText(["Detail about Why small teams"]);
  await expect.poll(() => backend.writingFlags.length).toBe(1);
  expect(backend.writingFlags[0]).toMatchObject({
    video_id: "v-new",
    text: "Detail about Why small teams",
    comment: "Vague: says nothing concrete",
    prefix_context: expect.stringContaining("First idea: "),
  });

  await page.reload();
  await expect(flagMarks(page)).toHaveText(["Detail about Why small teams"]);

  await flagMarks(page).click();
  await expect(page.getByText("Vague: says nothing concrete")).toBeVisible();
  await page.getByRole("button", { name: "Remove flag" }).click();
  await expect(flagMarks(page)).toHaveCount(0);
  expect(backend.deletedWritingFlagIds).toHaveLength(1);
});

test("cancelling the note flags nothing", async ({ page }) => {
  const backend = await openPaper(page, {
    channels: CHANNELS,
    stories: STORIES,
  });
  await expect(headline(page)).toBeVisible();

  await selectInArticle(page, "Detail about Why small teams");
  await page.getByRole("button", { name: "Flag writing" }).click();
  await noteField(page).fill("Not sure");
  await page.getByRole("button", { name: "Cancel" }).click();

  await expect(noteField(page)).toHaveCount(0);
  await expect(flagMarks(page)).toHaveCount(0);
  expect(backend.writingFlags).toHaveLength(0);
});

test("the profile lists flagged passages and removes them", async ({
  page,
}) => {
  const backend = await openPaper(
    page,
    {
      channels: CHANNELS,
      stories: STORIES,
      writingFlags: [
        {
          id: "90071992547409931",
          video_id: "v-new",
          text: "the half worth fearing more",
          comment: "Invented ranking",
          prefix_context: "",
          suffix_context: "",
          created_at: "2026-10-06T09:00:00Z",
        },
      ],
    },
    { path: "/profile" },
  );

  const feedback = page.getByRole("region", { name: "Writing feedback" });
  await expect(feedback.getByText("the half worth fearing more")).toBeVisible();
  await expect(feedback.getByText("Invented ranking")).toBeVisible();
  await expect(
    feedback.getByRole("link", { name: "Open story" }),
  ).toHaveAttribute("href", "/stories/v-new");

  await feedback
    .getByRole("button", { name: "Remove flag: Invented ranking" })
    .click();
  await expect(
    feedback.getByText("You have not flagged any passages yet."),
  ).toBeVisible();
  expect(backend.deletedWritingFlagIds).toEqual(["90071992547409931"]);
});

test.describe("phone", () => {
  test.use({ viewport: { width: 375, height: 812 } });

  test("the note form fits on a phone screen", async ({ page }) => {
    await openPaper(page, { channels: CHANNELS, stories: STORIES });
    await expect(headline(page)).toBeVisible();

    await selectInArticle(page, "Detail about Why small teams");
    await page.getByRole("button", { name: "Flag writing" }).click();

    const form = page.locator("form.flag-form");
    await expect(form).toBeVisible();
    const box = (await form.boundingBox())!;
    expect(box.x).toBeGreaterThanOrEqual(0);
    expect(box.x + box.width).toBeLessThanOrEqual(375);
  });
});
