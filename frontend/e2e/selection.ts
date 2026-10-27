import { expect, type Page } from "@playwright/test";

/** Selects the first occurrence of `needle` inside the article. */
export async function selectInArticle(page: Page, needle: string) {
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
