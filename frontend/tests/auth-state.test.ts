import { afterEach, beforeEach, describe, expect, it, mock } from "bun:test";

type MockUser = {
  uid: string;
  email: string | null;
  isAnonymous: boolean;
  getIdToken: (forceRefresh?: boolean) => Promise<string>;
};

let authStateListener: ((user: MockUser | null) => void) | null = null;
let authStateReadyPromise: Promise<void> = Promise.resolve();
const firebaseAuthInstance = {
  currentUser: null as MockUser | null,
  authStateReady: mock(() => authStateReadyPromise),
};

class MockGoogleAuthProvider {}

const mockOnAuthStateChanged = mock(
  (
    _auth: typeof firebaseAuthInstance,
    callback: ((user: MockUser | null) => void) | null,
  ) => {
    authStateListener = callback;
    return () => {
      authStateListener = null;
    };
  },
);
const mockSignInAnonymously = mock(async () => {
  const user: MockUser = {
    uid: "anon-123",
    email: null,
    isAnonymous: true,
    getIdToken: async () => "anon-token",
  };
  firebaseAuthInstance.currentUser = user;
  authStateListener?.(user);
  return { user };
});
const mockSignInWithPopup = mock(async () => {
  const user: MockUser = {
    uid: "google-123",
    email: "person@example.com",
    isAnonymous: false,
    getIdToken: async () => "google-token",
  };
  firebaseAuthInstance.currentUser = user;
  authStateListener?.(user);
  return { user };
});
const mockSignOut = mock(async () => {
  firebaseAuthInstance.currentUser = null;
  authStateListener?.(null);
});
const mockResetApiCacheForAuthChange = mock(() => undefined);

mock.module("$lib/auth/firebase", () => ({
  auth: firebaseAuthInstance,
}));

mock.module("firebase/auth", () => ({
  GoogleAuthProvider: MockGoogleAuthProvider,
  onAuthStateChanged: mockOnAuthStateChanged,
  signInAnonymously: mockSignInAnonymously,
  signInWithPopup: mockSignInWithPopup,
  signOut: mockSignOut,
}));

mock.module("$lib/api/cache-reset", () => ({
  resetApiCacheForAuthChange: mockResetApiCacheForAuthChange,
}));

const originalWindow = globalThis.window;

async function loadAuthStateModule() {
  return import(
    `../src/lib/auth/state.svelte.ts?test=${Date.now()}-${Math.random()}`
  );
}

beforeEach(() => {
  authStateListener = null;
  firebaseAuthInstance.currentUser = null;
  authStateReadyPromise = Promise.resolve();
  Object.defineProperty(globalThis, "window", {
    value: {},
    configurable: true,
  });
});

afterEach(() => {
  authStateListener = null;
  firebaseAuthInstance.currentUser = null;
  mockOnAuthStateChanged.mockClear();
  mockSignInAnonymously.mockClear();
  mockSignInWithPopup.mockClear();
  mockSignOut.mockClear();
  mockResetApiCacheForAuthChange.mockClear();
  firebaseAuthInstance.authStateReady.mockClear();

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
  it("bootstraps an anonymous Firebase user when no current user exists", async () => {
    const { authState } = await loadAuthStateModule();

    authState.setServerAuth({
      userId: null,
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });

    await authState.start();

    expect(mockSignInAnonymously).toHaveBeenCalledTimes(1);
    expect(authState.current).toEqual({
      userId: "anon-123",
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });
  });

  it("reuses an existing Firebase user on start instead of creating a second anonymous user", async () => {
    firebaseAuthInstance.currentUser = {
      uid: "google-123",
      email: "person@example.com",
      isAnonymous: false,
      getIdToken: async () => "google-token",
    };

    const { authState } = await loadAuthStateModule();
    await authState.start();

    expect(mockSignInAnonymously).not.toHaveBeenCalled();
    expect(authState.current).toEqual({
      userId: "google-123",
      authState: "authenticated",
      accessRole: "user",
      email: "person@example.com",
    });
  });

  it("waits for Firebase auth restoration before creating an anonymous session", async () => {
    let resolveAuthStateReady: (() => void) | null = null;
    authStateReadyPromise = new Promise<void>((resolve) => {
      resolveAuthStateReady = resolve;
    });

    const persistedUser: MockUser = {
      uid: "google-123",
      email: "person@example.com",
      isAnonymous: false,
      getIdToken: async () => "google-token",
    };

    const { authState } = await loadAuthStateModule();
    const startPromise = authState.start();

    await Promise.resolve();
    expect(mockSignInAnonymously).not.toHaveBeenCalled();

    firebaseAuthInstance.currentUser = persistedUser;
    authStateListener?.(persistedUser);
    resolveAuthStateReady?.();

    await startPromise;

    expect(mockSignInAnonymously).not.toHaveBeenCalled();
    expect(authState.current).toEqual({
      userId: "google-123",
      authState: "authenticated",
      accessRole: "user",
      email: "person@example.com",
    });
  });

  it("uses the dev/test E2E auth override without bootstrapping Firebase", async () => {
    Object.defineProperty(globalThis, "window", {
      value: {
        localStorage: {
          getItem: (key: string) =>
            key === "__dastill_e2e_auth"
              ? JSON.stringify({
                  userId: "mini-e2e-user",
                  email: "mini-e2e@example.com",
                  token: "mini-e2e-token",
                })
              : null,
        },
      },
      configurable: true,
    });

    const { authState } = await loadAuthStateModule();
    const { getCurrentAuthToken } = await import("../src/lib/auth/token");

    await authState.start();

    expect(mockOnAuthStateChanged).not.toHaveBeenCalled();
    expect(mockSignInAnonymously).not.toHaveBeenCalled();
    expect(await getCurrentAuthToken()).toBe("mini-e2e-token");
    expect(authState.current).toEqual({
      userId: "mini-e2e-user",
      authState: "authenticated",
      accessRole: "user",
      email: "mini-e2e@example.com",
    });
  });

  it("preserves an established Firebase session when server auth falls back to anonymous", async () => {
    const { authState } = await loadAuthStateModule();
    await authState.signInWithGoogle();

    mockSignOut.mockClear();
    mockSignInAnonymously.mockClear();

    authState.setServerAuth({
      userId: null,
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });

    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(mockSignOut).not.toHaveBeenCalled();
    expect(mockSignInAnonymously).not.toHaveBeenCalled();
    expect(authState.current).toEqual({
      userId: "google-123",
      authState: "authenticated",
      accessRole: "user",
      email: "person@example.com",
    });
  });

  it("signs in with Google without exchanging a server session", async () => {
    const { authState } = await loadAuthStateModule();

    await authState.signInWithGoogle();

    expect(mockSignInWithPopup).toHaveBeenCalledTimes(1);
    expect(authState.current).toEqual({
      userId: "google-123",
      authState: "authenticated",
      accessRole: "user",
      email: "person@example.com",
    });
  });

  it("signs out of Firebase and re-establishes the anonymous session", async () => {
    const { authState } = await loadAuthStateModule();
    await authState.signInWithGoogle();

    const nextAuth = await authState.signOut();

    expect(mockSignOut).toHaveBeenCalled();
    expect(mockSignInAnonymously).toHaveBeenCalledTimes(1);
    expect(nextAuth).toEqual({
      userId: "anon-123",
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });
    expect(authState.current).toEqual(nextAuth);
  });
});
