import type { Page } from "@playwright/test";

export async function resetClientState(page: Page): Promise<void> {
  await page.goto("/robots.txt");
  await page.evaluate(async () => {
    try {
      window.localStorage.clear();
      window.sessionStorage.clear();
    } catch {
      /* ignore */
    }

    try {
      if ("serviceWorker" in navigator) {
        const registrations = await navigator.serviceWorker.getRegistrations();
        await Promise.all(
          registrations.map((registration) => registration.unregister()),
        );
      }
    } catch {
      /* ignore */
    }

    try {
      if ("caches" in window) {
        const cacheNames = await caches.keys();
        await Promise.all(
          cacheNames.map((cacheName) => caches.delete(cacheName)),
        );
      }
    } catch {
      /* ignore */
    }

    try {
      const databaseNames = new Set<string>(["dastill-workspace-cache"]);
      if (
        typeof indexedDB !== "undefined" &&
        typeof indexedDB.databases === "function"
      ) {
        const databases = await indexedDB.databases();
        for (const database of databases) {
          if (database.name) {
            databaseNames.add(database.name);
          }
        }
      }

      await Promise.all(
        [...databaseNames].map(
          (databaseName) =>
            new Promise<void>((resolve) => {
              const request = indexedDB.deleteDatabase(databaseName);
              request.onsuccess = () => resolve();
              request.onerror = () => resolve();
              request.onblocked = () => resolve();
            }),
        ),
      );
    } catch {
      /* ignore */
    }
  });

  await page.goto("about:blank");
}

export async function openFreshGuestPage(
  page: Page,
  path = "/",
): Promise<void> {
  await resetClientState(page);
  await page.goto(path);
}
