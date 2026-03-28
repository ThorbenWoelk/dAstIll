import { dev } from "$app/environment";
import { env } from "$env/dynamic/public";
import { getApps, initializeApp, type FirebaseApp } from "firebase/app";
import { connectAuthEmulator, getAuth, type Auth } from "firebase/auth";

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
const LOCAL_DEV_FIREBASE_AUTH_EMULATOR_HOST = "127.0.0.1:9099";

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
  key: keyof typeof env,
  localFallback: string,
): string {
  const value = env[key]?.trim();
  if (!value) {
    if (shouldUseLocalFallbackConfig()) {
      return localFallback;
    }
    throw new Error(`${key} must be set`);
  }
  return value;
}

function readFirebaseAuthEmulatorHost(): string | null {
  const configuredHost =
    env.PUBLIC_FIREBASE_AUTH_EMULATOR_HOST?.trim() ??
    import.meta.env.PUBLIC_FIREBASE_AUTH_EMULATOR_HOST ??
    import.meta.env.FIREBASE_AUTH_EMULATOR_HOST ??
    readProcessEnv("FIREBASE_AUTH_EMULATOR_HOST");
  const normalizedHost = configuredHost?.trim();
  if (normalizedHost) {
    return normalizedHost;
  }

  return dev ? LOCAL_DEV_FIREBASE_AUTH_EMULATOR_HOST : null;
}

export const firebaseConfig: FirebaseClientConfig = {
  apiKey: requiredPublicEnv(
    "PUBLIC_FIREBASE_API_KEY",
    LOCAL_DEV_FIREBASE_CONFIG.apiKey,
  ),
  authDomain: requiredPublicEnv(
    "PUBLIC_FIREBASE_AUTH_DOMAIN",
    LOCAL_DEV_FIREBASE_CONFIG.authDomain,
  ),
  projectId: requiredPublicEnv(
    "PUBLIC_FIREBASE_PROJECT_ID",
    LOCAL_DEV_FIREBASE_CONFIG.projectId,
  ),
};

export const firebaseApp: FirebaseApp =
  getApps()[0] ?? initializeApp(firebaseConfig);

export const auth: Auth = getAuth(firebaseApp);

const authEmulatorHost = readFirebaseAuthEmulatorHost();

if (typeof window !== "undefined" && authEmulatorHost && !auth.emulatorConfig) {
  connectAuthEmulator(auth, `http://${authEmulatorHost}`, {
    disableWarnings: true,
  });
}
