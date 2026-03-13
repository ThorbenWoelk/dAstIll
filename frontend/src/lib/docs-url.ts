const DEFAULT_DOCS_URL = "http://localhost:4173";

export function resolveDocsUrl(configuredUrl?: string): string {
  const normalizedUrl = configuredUrl?.trim();
  if (normalizedUrl) {
    return normalizedUrl;
  }

  return DEFAULT_DOCS_URL;
}
