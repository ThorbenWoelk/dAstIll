import { afterEach, beforeEach, describe, expect, it, mock } from "bun:test";

type MockUser = {
  uid: string;
  email: string | null;
  isAnonymous: boolean;
  getIdToken: (forceRefresh?: boolean) => Promise<string>;
};

let authStateListener: ((user: MockUser | null) => void) | null = null;
const firebaseAuthInstance = {
  currentUser: null as MockUser | null,
};

const firebaseAuthModule = {
  GoogleAuthProvider: class GoogleAuthProvider {},
  connectAuthEmulator: mock(() => undefined),
  getAuth: mock(() => firebaseAuthInstance),
  onAuthStateChanged: mock(
    (
      _auth: typeof firebaseAuthInstance,
      callback: typeof authStateListener,
    ) => {
      authStateListener = callback as typeof authStateListener;
      return () => {
        authStateListener = null;
      };
    },
  ),
  signInAnonymously: mock(async () => {
    const user: MockUser = {
      uid: "anon-123",
      email: null,
      isAnonymous: true,
      getIdToken: async () => "anon-token",
    };
    firebaseAuthInstance.currentUser = user;
    authStateListener?.(user);
    return { user };
  }),
  signInWithPopup: mock(async () => {
    const user: MockUser = {
      uid: "google-123",
      email: "person@example.com",
      isAnonymous: false,
      getIdToken: async () => "google-token",
    };
    firebaseAuthInstance.currentUser = user;
    authStateListener?.(user);
    return { user };
  }),
  signOut: mock(async () => {
    firebaseAuthInstance.currentUser = null;
    authStateListener?.(null);
  }),
};

mock.module("$lib/firebase", () => ({
  auth: firebaseAuthInstance,
}));

mock.module("firebase/auth", () => firebaseAuthModule);

const originalFetch = globalThis.fetch;
const originalWindow = globalThis.window;

function createFetchMock() {
  return mock(async (input: string | URL | Request, init?: RequestInit) => {
    const url =
      typeof input === "string"
        ? input
        : input instanceof URL
          ? input.pathname
          : new URL(input.url).pathname;

    if (url === "/auth/session" && init?.method === "DELETE") {
      return new Response(
        JSON.stringify({
          userId: null,
          authState: "anonymous",
          accessRole: "anonymous",
          email: null,
        }),
        {
          status: 200,
          headers: {
            "Content-Type": "application/json",
          },
        },
      );
    }

    const body = init?.body ? JSON.parse(String(init.body)) : {};
    const authPayload =
      body.idToken === "google-token"
        ? {
            userId: "google-123",
            authState: "authenticated",
            accessRole: "user",
            email: "person@example.com",
          }
        : {
            userId: "anon-123",
            authState: "anonymous",
            accessRole: "anonymous",
            email: null,
          };

    return new Response(JSON.stringify(authPayload), {
      status: 200,
      headers: {
        "Content-Type": "application/json",
      },
    });
  });
}

async function loadAuthStateModule() {
  return import(
    `../src/lib/auth-state.svelte.ts?test=${Date.now()}-${Math.random()}`
  );
}

beforeEach(() => {
  Object.defineProperty(globalThis, "window", {
    value: {},
    configurable: true,
  });
});

afterEach(() => {
  firebaseAuthInstance.currentUser = null;
  authStateListener = null;
  firebaseAuthModule.onAuthStateChanged.mockClear();
  firebaseAuthModule.signInAnonymously.mockClear();
  firebaseAuthModule.signInWithPopup.mockClear();
  firebaseAuthModule.signOut.mockClear();
  firebaseAuthModule.connectAuthEmulator.mockClear();
  firebaseAuthModule.getAuth.mockClear();
  globalThis.fetch = originalFetch;
  if (originalWindow === undefined) {
    delete (globalThis as typeof globalThis & { window?: unknown }).window;
  } else {
    Object.defineProperty(globalThis, "window", {
      value: originalWindow,
      configurable: true,
    });
  }
});

describe("auth state controller", () => {
  it("silently bootstraps an anonymous session when no cookie-backed user exists", async () => {
    globalThis.fetch = createFetchMock() as typeof fetch;

    const { authState } = await loadAuthStateModule();

    authState.setServerAuth({
      userId: null,
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });

    await authState.start();

    expect(firebaseAuthModule.signInAnonymously).toHaveBeenCalledTimes(1);
    expect(authState.current).toEqual({
      userId: "anon-123",
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });
  });

  it("signs in with Google and exchanges the popup token for a server session", async () => {
    globalThis.fetch = createFetchMock() as typeof fetch;

    const { authState } = await loadAuthStateModule();

    authState.setServerAuth({
      userId: "anon-123",
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });

    await authState.start();
    await authState.signInWithGoogle();

    expect(firebaseAuthModule.signInWithPopup).toHaveBeenCalledTimes(1);
    expect(authState.current).toEqual({
      userId: "google-123",
      authState: "authenticated",
      accessRole: "user",
      email: "person@example.com",
    });
  });
});
