import { expect, test } from "@playwright/test";
import { openFreshGuestPage } from "./test-helpers";

function desktopChatSidebar(page: import("@playwright/test").Page) {
  return page.locator("#conversations-panel > .hidden.lg\\:flex").first();
}

test("delete all clears chat history from the sidebar", async ({ page }) => {
  await openFreshGuestPage(page, "/chat");
  const sidebar = desktopChatSidebar(page);
  const newConversationButton = sidebar.getByRole("button", {
    name: "New",
    exact: true,
  });
  await expect(newConversationButton).toBeVisible();
  await newConversationButton.click();
  await expect
    .poll(() => sidebar.getByLabel("Delete conversation").count())
    .toBe(1);
  await expect(newConversationButton).toBeEnabled();
  await newConversationButton.click();
  await expect
    .poll(() => sidebar.getByLabel("Delete conversation").count())
    .toBe(2);

  const deleteAllButton = sidebar.getByRole("button", {
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
  await expect(sidebar.getByLabel("Delete conversation")).toHaveCount(0);
});
