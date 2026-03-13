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
