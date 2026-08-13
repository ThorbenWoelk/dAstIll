export function shouldReloadMiniForAuthScope(params: {
  authReady: boolean;
  loadedAuthScopeKey: string | null;
  loadingAuthScopeKey: string | null;
  authScopeKey: string;
}): boolean {
  if (!params.authReady) {
    return false;
  }

  return (
    params.loadedAuthScopeKey !== params.authScopeKey &&
    params.loadingAuthScopeKey !== params.authScopeKey
  );
}

export function shouldRedirectMiniToLogin(authState: string): boolean {
  return authState !== "authenticated";
}

export type MiniReaderAuthScopeResetTarget = {
  clearReaderState(): void;
  clearPreferences(): void;
  resetHighlights(): void;
  resetVocabulary(): void;
};

export function resetMiniReaderForAuthScopeChange(
  target: MiniReaderAuthScopeResetTarget,
) {
  target.clearReaderState();
  target.clearPreferences();
  target.resetHighlights();
  target.resetVocabulary();
}
