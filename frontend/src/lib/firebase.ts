import { dev } from "$app/environment";
import { getApps, initializeApp, type FirebaseApp } from "firebase/app";
import * as firebaseAuth from "firebase/auth";
import type { Auth } from "firebase/auth";

export interface FirebaseClientConfig {
  apiKey: string;
  authDomain: string;
  projectId: string;
}

const LOCAL_DEV_FIREBASE_CONFIG: FirebaseClientConfig = {
  apiKey: "fake-api-key",
  authDomain: "demo-dastill.firebaseapp.com",
  projectId: "demo-dastill",
};

const publicEnv = (
  import.meta as {
    env?: {
      PUBLIC_FIREBASE_API_KEY?: string;
      PUBLIC_FIREBASE_AUTH_DOMAIN?: string;
      PUBLIC_FIREBASE_AUTH_EMULATOR_HOST?: string;
      PUBLIC_FIREBASE_PROJECT_ID?: string;
      FIREBASE_AUTH_EMULATOR_HOST?: string;
    };
  }
).env;

function readProcessEnv(key: string): string | undefined {
  return typeof process !== "undefined" ? process.env[key] : undefined;
}

function shouldUseLocalFallbackConfig(): boolean {
  return (
    dev ||
    import.meta.env.MODE === "test" ||
    Boolean(readFirebaseAuthEmulatorHost())
  );
}

function requiredPublicEnv(
  key: string,
  localFallback: string,
  value?: string,
): string {
  const normalizedValue = value?.trim();
  if (!normalizedValue) {
    if (shouldUseLocalFallbackConfig()) {
      return localFallback;
    }
    throw new Error(`${key} must be set`);
  }
  return normalizedValue;
}

function readFirebaseWebApiKey(): string {
  const value =
    publicEnv?.PUBLIC_FIREBASE_API_KEY?.trim() ||
    readProcessEnv("PUBLIC_FIREBASE_API_KEY")?.trim();
  if (value) {
    return value;
  }
  if (shouldUseLocalFallbackConfig()) {
    return LOCAL_DEV_FIREBASE_CONFIG.apiKey;
  }
  throw new Error("PUBLIC_FIREBASE_API_KEY (Firebase Web API key) must be set");
}

function readFirebaseAuthEmulatorHost(): string | null {
  const configuredHost =
    publicEnv?.PUBLIC_FIREBASE_AUTH_EMULATOR_HOST?.trim() ??
    import.meta.env.PUBLIC_FIREBASE_AUTH_EMULATOR_HOST ??
    import.meta.env.FIREBASE_AUTH_EMULATOR_HOST ??
    readProcessEnv("FIREBASE_AUTH_EMULATOR_HOST");
  const normalizedHost = configuredHost?.trim();
  if (normalizedHost) {
    return normalizedHost;
  }

  return null;
}

export const firebaseConfig: FirebaseClientConfig = {
  apiKey: readFirebaseWebApiKey(),
  authDomain: requiredPublicEnv(
    "PUBLIC_FIREBASE_AUTH_DOMAIN",
    LOCAL_DEV_FIREBASE_CONFIG.authDomain,
    publicEnv?.PUBLIC_FIREBASE_AUTH_DOMAIN,
  ),
  projectId: requiredPublicEnv(
    "PUBLIC_FIREBASE_PROJECT_ID",
    LOCAL_DEV_FIREBASE_CONFIG.projectId,
    publicEnv?.PUBLIC_FIREBASE_PROJECT_ID,
  ),
};

export const firebaseApp: FirebaseApp =
  getApps()[0] ?? initializeApp(firebaseConfig);

export const auth: Auth = firebaseAuth.getAuth(firebaseApp);

const authEmulatorHost = readFirebaseAuthEmulatorHost();

if (
  typeof window !== "undefined" &&
  authEmulatorHost &&
  !auth.emulatorConfig &&
  typeof firebaseAuth.connectAuthEmulator === "function"
) {
  firebaseAuth.connectAuthEmulator(auth, `http://${authEmulatorHost}`, {
    disableWarnings: true,
  });
}
