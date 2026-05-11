type BrowserLocation = Pick<Location, "protocol" | "hostname">;

type BrowserNavigator = {
  serviceWorker?: Pick<ServiceWorkerContainer, "register">;
};

const LOCALHOST_HOSTNAMES = new Set(["localhost", "127.0.0.1", "[::1]"]);

export function canRegisterServiceWorker(location: BrowserLocation): boolean {
  return (
    location.protocol === "https:" || LOCALHOST_HOSTNAMES.has(location.hostname)
  );
}

export async function registerAppServiceWorker(
  browserNavigator: BrowserNavigator | undefined = typeof navigator ===
  "undefined"
    ? undefined
    : navigator,
  browserLocation: BrowserLocation | undefined = typeof location === "undefined"
    ? undefined
    : location,
  scriptUrl = "/sw.js",
): Promise<boolean> {
  if (
    !browserNavigator?.serviceWorker ||
    !browserLocation ||
    !canRegisterServiceWorker(browserLocation)
  ) {
    return false;
  }

  try {
    await browserNavigator.serviceWorker.register(scriptUrl);
    return true;
  } catch (error) {
    console.error("Service worker registration failed", error);
    return false;
  }
}

type BrowserServiceWorkerContainer = ServiceWorkerContainer & {
  getRegistrations?: () => Promise<ServiceWorkerRegistration[]>;
};

export async function unregisterAppServiceWorkers(
  browserNavigator: BrowserNavigator | undefined = typeof navigator ===
  "undefined"
    ? undefined
    : navigator,
): Promise<void> {
  const serviceWorker = browserNavigator?.serviceWorker as
    | BrowserServiceWorkerContainer
    | undefined;

  if (!serviceWorker?.getRegistrations) {
    return;
  }

  const registrations = await serviceWorker.getRegistrations();
  await Promise.all(
    registrations.map((registration) => registration.unregister()),
  );

  if (typeof caches === "undefined") {
    return;
  }

  const cacheNames = await caches.keys();
  await Promise.all(
    cacheNames
      .filter(
        (name) =>
          name.startsWith("static-") ||
          name.startsWith("api-") ||
          name.startsWith("avatars-"),
      )
      .map((name) => caches.delete(name)),
  );
}
