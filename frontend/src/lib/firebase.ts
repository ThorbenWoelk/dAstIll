import { env } from "$env/dynamic/public";
import { getApps, initializeApp, type FirebaseApp } from "firebase/app";
import { connectAuthEmulator, getAuth, type Auth } from "firebase/auth";

export interface FirebaseClientConfig {
  apiKey: string;
  authDomain: string;
  projectId: string;
}

function requiredPublicEnv(key: keyof typeof env): string {
  const value = env[key]?.trim();
  if (!value) {
    throw new Error(`${key} must be set`);
  }
  return value;
}

function readFirebaseAuthEmulatorHost(): string | null {
  const configuredHost =
    import.meta.env.FIREBASE_AUTH_EMULATOR_HOST ??
    process.env.FIREBASE_AUTH_EMULATOR_HOST;
  const normalizedHost = configuredHost?.trim();
  return normalizedHost ? normalizedHost : null;
}

export const firebaseConfig: FirebaseClientConfig = {
  apiKey: requiredPublicEnv("PUBLIC_FIREBASE_API_KEY"),
  authDomain: requiredPublicEnv("PUBLIC_FIREBASE_AUTH_DOMAIN"),
  projectId: requiredPublicEnv("PUBLIC_FIREBASE_PROJECT_ID"),
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
