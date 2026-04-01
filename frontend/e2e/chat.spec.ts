import { expect, test, type APIRequestContext } from "@playwright/test";

async function clearConversations(request: APIRequestContext) {
  const listResponse = await request.get("/api/chat/conversations");
  expect(listResponse.ok()).toBeTruthy();

  const conversations = (await listResponse.json()) as Array<{ id: string }>;
  for (const conversation of conversations) {
    const deleteResponse = await request.delete(
      `/api/chat/conversations/${conversation.id}`,
    );
    expect(deleteResponse.ok()).toBeTruthy();
  }
}

test.beforeEach(async ({ context, request }) => {
  await context.addInitScript(() => {
    try {
      localStorage.clear();
      sessionStorage.clear();
    } catch {
      /* ignore */
    }
  });
  await clearConversations(request);
});

test.afterEach(async ({ request }) => {
  await clearConversations(request);
});

test("delete all clears chat history from the sidebar", async ({
  page,
  request,
}) => {
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
  await expect
    .poll(async () => {
      const listResponse = await request.get("/api/chat/conversations");
      expect(listResponse.ok()).toBeTruthy();
      const conversations = (await listResponse.json()) as Array<{
        id: string;
      }>;
      return conversations.length;
    })
    .toBe(0);
});
