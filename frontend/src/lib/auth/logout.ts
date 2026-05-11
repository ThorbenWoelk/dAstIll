import { authState } from "$lib/auth/state.svelte";

export const HOME_WORKSPACE_HREF = "/";

type LocationLike = {
  href: string;
};

export async function signOutAndReloadHome({
  signOut = () => authState.signOut(),
  location = typeof window !== "undefined" ? window.location : undefined,
}: {
  signOut?: () => Promise<unknown>;
  location?: LocationLike;
} = {}) {
  await signOut();

  if (location) {
    location.href = HOME_WORKSPACE_HREF;
  }
}
