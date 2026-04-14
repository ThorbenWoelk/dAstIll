import { resolveDocsUrl } from "$lib/docs-url";
import { resolveMaintenanceMode } from "$lib/maintenance-mode";

const publicEnv = (
  import.meta as {
    env?: {
      PUBLIC_DOCS_URL?: string;
      PUBLIC_CONTACT_EMAIL?: string;
      PUBLIC_APP_MAINTENANCE_MODE?: string;
    };
  }
).env;

export const DOCS_URL = resolveDocsUrl(publicEnv?.PUBLIC_DOCS_URL);
export const CONTACT_EMAIL = publicEnv?.PUBLIC_CONTACT_EMAIL?.trim() || null;
export const APP_MAINTENANCE_MODE = resolveMaintenanceMode(
  publicEnv?.PUBLIC_APP_MAINTENANCE_MODE,
);
