import { resetApiCacheForAuthChange as resetApiCacheForAuthChangeInternal } from "$lib/api";

export function resetApiCacheForAuthChange() {
  resetApiCacheForAuthChangeInternal();
}
