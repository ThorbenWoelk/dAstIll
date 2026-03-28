import { afterEach, describe, expect, it, mock } from "bun:test";

const appModule = {
  getApps: mock(() => []),
  initializeApp: mock(() => ({ name: "firebase-app" })),
};

const authModule = {
  getAuth: mock(() => ({ emulatorConfig: null })),
  connectAuthEmulator: mock(() => undefined),
  GoogleAuthProvider: class GoogleAuthProvider {},
  onAuthStateChanged: mock(() => () => {}),
  signInAnonymously: mock(async () => ({ user: null })),
  signInWithPopup: mock(async () => ({ user: null })),
  signOut: mock(async () => undefined),
};

mock.module("$app/environment", () => ({
  dev: false,
}));

mock.module("$env/dynamic/public", () => ({
  env: process.env,
}));

mock.module("firebase/app", () => appModule);
mock.module("firebase/auth", () => authModule);

const originalEnv = {
  PUBLIC_FIREBASE_API_KEY: process.env.PUBLIC_FIREBASE_API_KEY,
  PUBLIC_FIREBASE_AUTH_DOMAIN: process.env.PUBLIC_FIREBASE_AUTH_DOMAIN,
  PUBLIC_FIREBASE_PROJECT_ID: process.env.PUBLIC_FIREBASE_PROJECT_ID,
  FIREBASE_AUTH_EMULATOR_HOST: process.env.FIREBASE_AUTH_EMULATOR_HOST,
};
const originalWindow = globalThis.window;

function restoreEnv() {
  for (const [key, value] of Object.entries(originalEnv)) {
    if (value === undefined) {
      delete process.env[key];
      continue;
    }

    process.env[key] = value;
  }
}

afterEach(() => {
  restoreEnv();
  appModule.getApps.mockClear();
  appModule.initializeApp.mockClear();
  authModule.getAuth.mockClear();
  authModule.connectAuthEmulator.mockClear();
  if (originalWindow === undefined) {
    delete (globalThis as typeof globalThis & { window?: unknown }).window;
  } else {
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
    });
  }
});

async function loadFirebaseModule() {
  Object.defineProperty(globalThis, "window", {
    value: {},
    configurable: true,
  });
  return import(`../src/lib/firebase?test=${Date.now()}-${Math.random()}`);
}

describe("firebase client config", () => {
  it("initializes firebase app and auth with the configured public env", async () => {
    process.env.PUBLIC_FIREBASE_API_KEY = "demo-key";
    process.env.PUBLIC_FIREBASE_AUTH_DOMAIN = "demo.firebaseapp.com";
    process.env.PUBLIC_FIREBASE_PROJECT_ID = "demo-project";
    delete process.env.FIREBASE_AUTH_EMULATOR_HOST;

    const firebase = await loadFirebaseModule();

    expect(firebase.firebaseConfig).toEqual({
      apiKey: "demo-key",
      authDomain: "demo.firebaseapp.com",
      projectId: "demo-project",
    });
    expect(appModule.initializeApp).toHaveBeenCalledWith(
      firebase.firebaseConfig,
    );
    expect(authModule.getAuth).toHaveBeenCalled();
    expect(authModule.connectAuthEmulator).not.toHaveBeenCalled();
  });

  it("connects the auth emulator when FIREBASE_AUTH_EMULATOR_HOST is set", async () => {
    process.env.PUBLIC_FIREBASE_API_KEY = "demo-key";
    process.env.PUBLIC_FIREBASE_AUTH_DOMAIN = "demo.firebaseapp.com";
    process.env.PUBLIC_FIREBASE_PROJECT_ID = "demo-project";
    process.env.FIREBASE_AUTH_EMULATOR_HOST = "127.0.0.1:9099";

    const firebase = await loadFirebaseModule();

    expect(firebase.firebaseConfig.projectId).toBe("demo-project");
    expect(authModule.connectAuthEmulator).toHaveBeenCalledWith(
      firebase.auth,
      "http://127.0.0.1:9099",
      { disableWarnings: true },
    );
  });
});
