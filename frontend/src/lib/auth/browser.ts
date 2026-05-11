import { normalizeRedirectTarget } from "$lib/auth";
import { createApiRequestInit, resolveApiUrl } from "$lib/api/client";
import { openUrl } from "@tauri-apps/plugin-opener";

const MOBILE_AUTH_REDIRECT_SESSION_KEY = "dastill.mobile-auth.handoff-session";
const MOBILE_AUTH_REDIRECT_COMPLETE_TOKEN_KEY =
  "dastill.mobile-auth.handoff-complete-token";

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
  completeToken: string,
): string {
  const url = new URL(resolveSystemBrowserLoginUrl(redirectTo));
  url.searchParams.set("handoffSession", handoffSessionId);
  url.hash = new URLSearchParams({
    handoffCompleteToken: completeToken,
  }).toString();
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

type MobileAuthCreatePayload = {
  status: "pending";
  handoff_id: string;
  complete_token: string;
  redeem_token: string;
};

type MobileAuthHandoffStatus = {
  status: "pending" | "complete";
  google_id_token: string | null;
  google_access_token: string | null;
};

type MobileAuthHandoffSession = {
  handoffSessionId: string;
  redeemToken: string;
};

const MOBILE_AUTH_HANDOFF_TIMEOUT_MS = 2 * 60 * 1000;
const MOBILE_AUTH_HANDOFF_POLL_MS = 1500;

function readHandoffCompleteTokenFromHash(): string | null {
  if (typeof window === "undefined") {
    return null;
  }

  const params = new URLSearchParams(window.location.hash.replace(/^#/, ""));
  const value = params.get("handoffCompleteToken")?.trim();
  return value || null;
}

function clearHandoffSecretFromUrl() {
  if (typeof window === "undefined" || !window.location.hash) {
    return;
  }

  const replacement = `${window.location.pathname}${window.location.search}`;
  window.history.replaceState(window.history.state, "", replacement);
}

async function createMobileAuthHandoff(): Promise<MobileAuthCreatePayload> {
  const response = await fetch(
    resolveApiUrl("/api/auth/mobile-handoff"),
    await createApiRequestInit(
      {
        method: "POST",
      },
      {
        includeJsonContentType: false,
      },
    ),
  );

  if (!response.ok) {
    throw new Error("Could not create mobile auth handoff.");
  }

  return (await response.json()) as MobileAuthCreatePayload;
}

async function redeemMobileAuthHandoff(
  sessionId: string,
  redeemToken: string,
): Promise<Response> {
  return fetch(
    resolveApiUrl(
      `/api/auth/mobile-handoff/${encodeURIComponent(sessionId)}/redeem`,
    ),
    await createApiRequestInit({
      method: "POST",
      body: JSON.stringify({
        redeem_token: redeemToken,
      }),
    }),
  );
}

async function pollMobileAuthHandoff(sessionId: string, redeemToken: string) {
  const startedAt = Date.now();

  for (;;) {
    const response = await redeemMobileAuthHandoff(sessionId, redeemToken);
    if (!response.ok && response.status !== 202) {
      throw new Error("Mobile browser sign-in handoff failed.");
    }
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

export async function startTauriAndroidBrowserAuthHandoff(
  redirectTo: string,
): Promise<MobileAuthHandoffSession> {
  const handoff = await createMobileAuthHandoff();
  const url = resolveSystemBrowserLoginUrlForSession(
    redirectTo,
    handoff.handoff_id,
    handoff.complete_token,
  );
  try {
    await openUrl(url);
  } catch {
    if (typeof window !== "undefined") {
      window.open(url, "_blank", "noopener,noreferrer");
    }
  }
  return {
    handoffSessionId: handoff.handoff_id,
    redeemToken: handoff.redeem_token,
  };
}

export async function finishTauriAndroidBrowserAuthHandoff(
  sessionId: string,
  redeemToken: string,
) {
  const [{ auth }, firebaseAuth] = await Promise.all([
    import("$lib/auth/firebase"),
    import("firebase/auth"),
  ]);
  const payload = await pollMobileAuthHandoff(sessionId, redeemToken);
  const credential = firebaseAuth.GoogleAuthProvider.credential(
    payload.google_id_token,
    payload.google_access_token,
  );

  await firebaseAuth.signInWithCredential(auth, credential);
}

export async function completeBrowserGoogleAuthHandoff(sessionId: string) {
  const [{ auth }, firebaseAuth] = await Promise.all([
    import("$lib/auth/firebase"),
    import("firebase/auth"),
  ]);
  const handoffCompleteToken = readHandoffCompleteTokenFromHash();
  if (handoffCompleteToken && typeof sessionStorage !== "undefined") {
    sessionStorage.setItem(
      MOBILE_AUTH_REDIRECT_COMPLETE_TOKEN_KEY,
      handoffCompleteToken,
    );
    clearHandoffSecretFromUrl();
  }

  const redirectResult = await firebaseAuth.getRedirectResult(auth);
  if (!redirectResult) {
    const provider = new firebaseAuth.GoogleAuthProvider();
    if (typeof sessionStorage !== "undefined") {
      sessionStorage.setItem(MOBILE_AUTH_REDIRECT_SESSION_KEY, sessionId);
      if (handoffCompleteToken) {
        sessionStorage.setItem(
          MOBILE_AUTH_REDIRECT_COMPLETE_TOKEN_KEY,
          handoffCompleteToken,
        );
      }
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

  const completeToken =
    typeof sessionStorage !== "undefined"
      ? (sessionStorage
          .getItem(MOBILE_AUTH_REDIRECT_COMPLETE_TOKEN_KEY)
          ?.trim() ?? "")
      : "";
  if (!completeToken) {
    throw new Error("Missing mobile auth handoff completion token.");
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
        complete_token: completeToken,
        google_id_token: googleIdToken,
        google_access_token: googleAccessToken,
      }),
    }),
  );

  if (typeof sessionStorage !== "undefined") {
    sessionStorage.removeItem(MOBILE_AUTH_REDIRECT_SESSION_KEY);
    sessionStorage.removeItem(MOBILE_AUTH_REDIRECT_COMPLETE_TOKEN_KEY);
  }
}
