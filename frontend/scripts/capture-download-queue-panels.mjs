import { chromium } from "@playwright/test";
import fs from "node:fs/promises";
import path from "node:path";

const BASE = "http://localhost:3543";
const OUT_DIR = path.resolve(process.cwd(), "test-results", "queue-ui");

const URLS = [
  {
    key: "queue-desktop",
    viewport: { width: 1280, height: 900 },
    url: `${BASE}/download-queue?channel=UC0C-17n9iuUQPylguM1d-lQ&video=u-giatW9mYU&type=all&ack=all`,
  },
  {
    key: "queue-mobile",
    viewport: { width: 390, height: 844 },
    url: `${BASE}/download-queue?channel=UC0C-17n9iuUQPylguM1d-lQ&video=u-giatW9mYU&type=all&ack=all`,
  },
];

function clampText(s) {
  return (s ?? "").replace(/\s+/g, " ").trim().slice(0, 120);
}

async function computeOverflowNote(page) {
  return page.evaluate(() => {
    const panel = document.querySelector("#content-view") ?? document.body;
    const vw = document.documentElement.clientWidth;
    const els = Array.from(panel.querySelectorAll("*"));

    let worst = null;
    for (const el of els) {
      const r = el.getBoundingClientRect();
      if (!r.width || r.height < 1) continue;
      const inVertical = r.top < window.innerHeight + 50 && r.bottom > -50;
      if (!inVertical) continue;

      const overflowPx = r.right - vw;
      if (overflowPx > 1) {
        const text = (el.textContent ?? "").trim();
        const className = typeof el.className === "string" ? el.className : "";
        if (!worst || overflowPx > worst.overflowPx) {
          worst = {
            tag: el.tagName.toLowerCase(),
            className,
            text,
            overflowPx,
            right: r.right,
            left: r.left,
            top: r.top,
            width: r.width,
          };
        }
      }
    }

    const docScrollWidth = Math.max(
      document.documentElement.scrollWidth,
      document.body.scrollWidth,
    );

    return {
      vw,
      hasHorizontalScroll: docScrollWidth > vw + 1,
      docScrollWidth,
      worst,
    };
  });
}

async function waitForQueuePanelState(page) {
  const panel = page.locator("#content-view");
  await panel.waitFor({ state: "visible", timeout: 60_000 });

  const selectedLabel = page
    .getByText("Selected queue item", { exact: true })
    .first();
  const notInQueue = page
    .getByText("Video not in the current queue list", { exact: true })
    .first();

  await Promise.race([
    selectedLabel
      .waitFor({ state: "visible", timeout: 60_000 })
      .then(() => "selected"),
    notInQueue
      .waitFor({ state: "visible", timeout: 60_000 })
      .then(() => "not-in-queue"),
  ]);
}

async function extractPanelState(page) {
  return page.evaluate(() => {
    const panel = document.querySelector("#content-view");
    if (!panel) return { state: "unknown" };

    const text = panel.innerText ?? "";
    if (text.includes("Video not in the current queue list")) {
      return { state: "not-in-queue" };
    }

    const labels = Array.from(panel.querySelectorAll("p"));
    const label = labels.find(
      (p) => (p.textContent ?? "").trim() === "Selected queue item",
    );

    if (label) {
      const titleEl = label.nextElementSibling;
      const title = (titleEl?.textContent ?? "").trim();
      return { state: "selected", title };
    }

    return { state: "unknown" };
  });
}

async function main() {
  await fs.mkdir(OUT_DIR, { recursive: true });

  const browser = await chromium.launch();
  const results = [];

  for (const item of URLS) {
    const page = await browser.newPage({ viewport: item.viewport });
    await page.goto(item.url, { waitUntil: "domcontentloaded" });

    await waitForQueuePanelState(page);
    const panelState = await extractPanelState(page);
    const overflow = await computeOverflowNote(page);

    const outPath = path.join(OUT_DIR, `${item.key}.png`);
    await page.screenshot({ path: outPath, fullPage: false });
    await page.close();

    let overflowNote = "No right-edge overflow detected in #content-view.";
    if (overflow.worst) {
      const worst = overflow.worst;
      overflowNote = `Element '${worst.tag}' (class='${clampText(
        worst.className,
      )}', text='${clampText(worst.text)}') overflows right edge by ~${Math.round(
        worst.overflowPx,
      )}px.`;
      if (overflow.hasHorizontalScroll) {
        overflowNote += ` Horizontal scroll present (docScrollWidth=${overflow.docScrollWidth}, vw=${overflow.vw}).`;
      }
    } else if (overflow.hasHorizontalScroll) {
      overflowNote = `Horizontal scroll present (docScrollWidth=${overflow.docScrollWidth}, vw=${overflow.vw}), but no single overflowing element detected.`;
    }

    results.push({
      key: item.key,
      url: item.url,
      viewport: item.viewport,
      panelState,
      screenshotPath: outPath,
      overflowNote,
    });
  }

  await browser.close();
  console.log(JSON.stringify(results, null, 2));
}

main().catch((err) => {
  console.error(err);
  process.exit(1);
});
