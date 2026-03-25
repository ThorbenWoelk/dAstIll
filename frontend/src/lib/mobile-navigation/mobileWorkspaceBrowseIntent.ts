import { writable } from "svelte/store";

/**
 * Set to true when the user chooses Home / Workspace on mobile so +page opens
 * the channel browse overlay even if workspace state would otherwise show a video.
 */
export const mobileWorkspaceBrowseIntent = writable(false);
