/** Build-time configuration. Values come from `PUBLIC_*` env vars set by the deploy workflow. */

interface PublicEnv {
  DEV?: boolean;
  MODE?: string;
  PUBLIC_API_BASE?: string;
  VITE_API_BASE?: string;
  PUBLIC_FIREBASE_API_KEY?: string;
  PUBLIC_FIREBASE_AUTH_DOMAIN?: string;
  PUBLIC_FIREBASE_PROJECT_ID?: string;
  PUBLIC_FIREBASE_AUTH_EMULATOR_HOST?: string;
  FIREBASE_AUTH_EMULATOR_HOST?: string;
}

const env: PublicEnv = (import.meta as { env?: PublicEnv }).env ?? {};

export function normalizeApiBase(value: string | undefined): string {
  const trimmed = value?.trim() ?? "";
  return trimmed.endsWith("/") ? trimmed.slice(0, -1) : trimmed;
}

/** Empty in local dev: Vite proxies `/api` to the backend. */
export const API_BASE = normalizeApiBase(
  env.PUBLIC_API_BASE || env.VITE_API_BASE,
);

/** True for `vite dev` and unit tests. Enables the local auth fallbacks below. */
export const IS_LOCAL = env.DEV === true || env.MODE === "test";

export interface FirebaseClientConfig {
  apiKey: string;
  authDomain: string;
  projectId: string;
}

const LOCAL_FIREBASE_CONFIG: FirebaseClientConfig = {
  apiKey: "fake-api-key",
  authDomain: "demo-dastill.firebaseapp.com",
  projectId: "demo-dastill",
};

export const FIREBASE_AUTH_EMULATOR_HOST =
  env.PUBLIC_FIREBASE_AUTH_EMULATOR_HOST?.trim() ||
  env.FIREBASE_AUTH_EMULATOR_HOST?.trim() ||
  "";

export function readFirebaseConfig(): FirebaseClientConfig {
  const config = {
    apiKey: env.PUBLIC_FIREBASE_API_KEY?.trim() ?? "",
    authDomain: env.PUBLIC_FIREBASE_AUTH_DOMAIN?.trim() ?? "",
    projectId: env.PUBLIC_FIREBASE_PROJECT_ID?.trim() ?? "",
  };
  if (config.apiKey && config.authDomain && config.projectId) {
    return config;
  }
  if (IS_LOCAL || FIREBASE_AUTH_EMULATOR_HOST) {
    return LOCAL_FIREBASE_CONFIG;
  }
  throw new Error(
    "PUBLIC_FIREBASE_API_KEY, PUBLIC_FIREBASE_AUTH_DOMAIN and PUBLIC_FIREBASE_PROJECT_ID must be set.",
  );
}
