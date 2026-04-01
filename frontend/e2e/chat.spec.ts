import { expect, test } from "@playwright/test";

test.beforeEach(async ({ context }) => {
  await context.addInitScript(() => {
    try {
      localStorage.clear();
      sessionStorage.clear();
    } catch {
      /* ignore */
    }
  });
});

test("delete all clears chat history from the sidebar", async ({ page }) => {
  await page.goto("/chat");
  await page.waitForTimeout(1500);
  const newConversationButton = page
    .getByRole("button", { name: "New", exact: true })
    .first();
  await expect(newConversationButton).toBeVisible();
  await newConversationButton.click();
  await expect
    .poll(() => page.getByLabel("Delete conversation").count())
    .toBe(1);
  await newConversationButton.click();
  await expect
    .poll(() => page.getByLabel("Delete conversation").count())
    .toBe(2);

  const deleteAllButton = page.getByRole("button", {
    name: "Delete all conversations",
  });
  await expect(deleteAllButton).toBeVisible();
  await deleteAllButton.click();

  const dialog = page.getByRole("dialog");
  await expect(dialog).toBeVisible();
  await dialog.getByRole("button", { name: "Delete all" }).click();

  await expect(
    page.getByText(
      "Start a new conversation to ask grounded questions about your library.",
    ),
  ).toBeVisible();
  await expect
    .poll(() =>
      page.evaluate(() =>
        sessionStorage.getItem("dastill.chat.ephemeralThreads.v1"),
      ),
    )
    .toBe(null);
  await expect
    .poll(() => new URL(page.url()).searchParams.get("id"))
    .toBe(null);
  await expect(page.getByLabel("Delete conversation")).toHaveCount(0);
});
