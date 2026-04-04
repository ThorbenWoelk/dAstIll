import { normalizeRedirectTarget } from "$lib/auth";
import { createApiRequestInit, resolveApiUrl } from "$lib/api-client";
import { openUrl } from "@tauri-apps/plugin-opener";

const MOBILE_AUTH_REDIRECT_SESSION_KEY = "dastill.mobile-auth.handoff-session";

function trimTrailingSlash(value: string) {
  return value.endsWith("/") ? value.slice(0, -1) : value;
}

function readConfiguredBrowserAuthBaseUrl(): string | null {
  const value = (
    import.meta as {
      env?: {
        PUBLIC_BROWSER_AUTH_BASE_URL?: string;
        VITE_BROWSER_AUTH_BASE_URL?: string;
      };
    }
  ).env?.PUBLIC_BROWSER_AUTH_BASE_URL?.trim()
    ? (
        import.meta as {
          env?: {
            PUBLIC_BROWSER_AUTH_BASE_URL?: string;
          };
        }
      ).env?.PUBLIC_BROWSER_AUTH_BASE_URL?.trim()
    : (
        import.meta as {
          env?: {
            VITE_BROWSER_AUTH_BASE_URL?: string;
          };
        }
      ).env?.VITE_BROWSER_AUTH_BASE_URL?.trim() || null;

  return value ? trimTrailingSlash(value) : null;
}

export function resolveBrowserAuthOrigin(
  currentUrl: URL = new URL(window.location.href),
): string {
  const configured = readConfiguredBrowserAuthBaseUrl();
  if (configured) {
    return configured;
  }

  return currentUrl.origin;
}

export function resolveSystemBrowserLoginUrl(redirectTo: string): string {
  const url = new URL("/login", resolveBrowserAuthOrigin());
  url.searchParams.set("redirectTo", normalizeRedirectTarget(redirectTo));
  url.searchParams.set("mobileBrowserAuth", "1");
  return url.toString();
}

export function resolveSystemBrowserLoginUrlForSession(
  redirectTo: string,
  handoffSessionId: string,
): string {
  const url = new URL(resolveSystemBrowserLoginUrl(redirectTo));
  url.searchParams.set("handoffSession", handoffSessionId);
  return url.toString();
}

export async function openSystemBrowserLogin(redirectTo: string) {
  const url = resolveSystemBrowserLoginUrl(redirectTo);
  try {
    await openUrl(url);
  } catch {
    if (typeof window !== "undefined") {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }
}

type MobileAuthHandoffStatus = {
  status: "pending" | "complete";
  google_id_token: string | null;
  google_access_token: string | null;
};

const MOBILE_AUTH_HANDOFF_TIMEOUT_MS = 2 * 60 * 1000;
const MOBILE_AUTH_HANDOFF_POLL_MS = 1500;

async function createMobileAuthHandoff(sessionId: string) {
  await fetch(
    resolveApiUrl(`/api/auth/mobile-handoff/${encodeURIComponent(sessionId)}`),
    await createApiRequestInit(
      {
        method: "POST",
      },
      {
        includeJsonContentType: false,
      },
    ),
  );
}

async function pollMobileAuthHandoff(sessionId: string) {
  const startedAt = Date.now();

  for (;;) {
    const response = await fetch(
      resolveApiUrl(
        `/api/auth/mobile-handoff/${encodeURIComponent(sessionId)}`,
      ),
      await createApiRequestInit(undefined, {
        includeJsonContentType: false,
      }),
    );
    const payload = (await response.json()) as MobileAuthHandoffStatus;
    if (
      payload.status === "complete" &&
      payload.google_id_token &&
      payload.google_access_token
    ) {
      return payload;
    }

    if (Date.now() - startedAt > MOBILE_AUTH_HANDOFF_TIMEOUT_MS) {
      throw new Error("Timed out waiting for browser sign-in to complete.");
    }

    await new Promise((resolve) =>
      window.setTimeout(resolve, MOBILE_AUTH_HANDOFF_POLL_MS),
    );
  }
}

export async function startTauriAndroidBrowserAuthHandoff(redirectTo: string) {
  const sessionId = crypto.randomUUID();
  await createMobileAuthHandoff(sessionId);
  const url = resolveSystemBrowserLoginUrlForSession(redirectTo, sessionId);
  try {
    await openUrl(url);
  } catch {
    if (typeof window !== "undefined") {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }
  return sessionId;
}

export async function finishTauriAndroidBrowserAuthHandoff(sessionId: string) {
  const [{ auth }, firebaseAuth] = await Promise.all([
    import("$lib/firebase"),
    import("firebase/auth"),
  ]);
  const payload = await pollMobileAuthHandoff(sessionId);
  const credential = firebaseAuth.GoogleAuthProvider.credential(
    payload.google_id_token,
    payload.google_access_token,
  );

  await firebaseAuth.signInWithCredential(auth, credential);

  await fetch(
    resolveApiUrl(`/api/auth/mobile-handoff/${encodeURIComponent(sessionId)}`),
    await createApiRequestInit(
      {
        method: "DELETE",
      },
      {
        includeJsonContentType: false,
      },
    ),
  );
}

export async function completeBrowserGoogleAuthHandoff(sessionId: string) {
  const [{ auth }, firebaseAuth] = await Promise.all([
    import("$lib/firebase"),
    import("firebase/auth"),
  ]);

  const redirectResult = await firebaseAuth.getRedirectResult(auth);
  if (!redirectResult) {
    const provider = new firebaseAuth.GoogleAuthProvider();
    if (typeof sessionStorage !== "undefined") {
      sessionStorage.setItem(MOBILE_AUTH_REDIRECT_SESSION_KEY, sessionId);
    }
    await firebaseAuth.signInWithRedirect(auth, provider);
    return;
  }

  if (
    typeof sessionStorage !== "undefined" &&
    sessionStorage.getItem(MOBILE_AUTH_REDIRECT_SESSION_KEY) !== sessionId
  ) {
    return;
  }

  const result = redirectResult;
  const credential =
    firebaseAuth.GoogleAuthProvider.credentialFromResult(result);

  const googleIdToken = credential?.idToken ?? null;
  const googleAccessToken = credential?.accessToken ?? null;

  if (!googleIdToken || !googleAccessToken) {
    throw new Error("Google did not return a reusable sign-in credential.");
  }

  await fetch(
    resolveApiUrl(`/api/auth/mobile-handoff/${encodeURIComponent(sessionId)}`),
    await createApiRequestInit({
      method: "PUT",
      body: JSON.stringify({
        google_id_token: googleIdToken,
        google_access_token: googleAccessToken,
      }),
    }),
  );

  if (typeof sessionStorage !== "undefined") {
    sessionStorage.removeItem(MOBILE_AUTH_REDIRECT_SESSION_KEY);
  }
}
