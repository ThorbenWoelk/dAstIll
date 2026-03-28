import { writable } from "svelte/store";
import { isSignInRequiredFailure } from "$lib/api-client";

/** Message for the global sign-in modal when an API call requires authentication. */
export const authRequiredNotice = writable<string | null>(null);

let suppressWhileFeatureGuideOpen = false;

/**
 * While the home feature guide is open, background fetches may hit sign-in-only
 * APIs; suppress the modal so the tour is not interrupted.
 */
export function setFeatureGuideSuppressesAuthRequiredNotice(
  suppress: boolean,
): void {
  suppressWhileFeatureGuideOpen = suppress;
}

export function presentAuthRequiredNotice(
  detail = "Sign in to continue.",
): void {
  if (suppressWhileFeatureGuideOpen) return;
  authRequiredNotice.set(detail);
}

export function dismissAuthRequiredNotice(): void {
  authRequiredNotice.set(null);
}

/**
 * If `error` indicates the user must sign in, opens the global sign-in modal and
 * returns true. Callers should skip toasts / inline error text when this returns true.
 */
export function presentAuthRequiredNoticeIfNeeded(error: unknown): boolean {
  if (!isSignInRequiredFailure(error)) return false;
  presentAuthRequiredNotice();
  return true;
}
