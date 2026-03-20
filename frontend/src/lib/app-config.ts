import { env } from "$env/dynamic/public";
import { resolveDocsUrl } from "$lib/docs-url";

export const DOCS_URL = resolveDocsUrl(env.PUBLIC_DOCS_URL);
export const CONTACT_EMAIL = env.PUBLIC_CONTACT_EMAIL?.trim() || null;
