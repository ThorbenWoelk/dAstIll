import { resolveDocsUrl } from "$lib/docs-url";

const publicEnv = (
  import.meta as {
    env?: {
      PUBLIC_DOCS_URL?: string;
      PUBLIC_CONTACT_EMAIL?: string;
    };
  }
).env;

export const DOCS_URL = resolveDocsUrl(publicEnv?.PUBLIC_DOCS_URL);
export const CONTACT_EMAIL = publicEnv?.PUBLIC_CONTACT_EMAIL?.trim() || null;
