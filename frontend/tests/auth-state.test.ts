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

mock.module("$lib/firebase", () => ({
  auth: firebaseAuthInstance,
}));

mock.module("firebase/auth", () => ({
  GoogleAuthProvider: MockGoogleAuthProvider,
  onAuthStateChanged: mockOnAuthStateChanged,
  signInAnonymously: mockSignInAnonymously,
  signInWithPopup: mockSignInWithPopup,
  signOut: mockSignOut,
}));

mock.module("$lib/api-cache-reset", () => ({
  resetApiCacheForAuthChange: mockResetApiCacheForAuthChange,
}));

const originalWindow = globalThis.window;

async function loadAuthStateModule() {
  return import(
    `../src/lib/auth-state.svelte.ts?test=${Date.now()}-${Math.random()}`
  );
}

beforeEach(() => {
  authStateListener = null;
  firebaseAuthInstance.currentUser = null;
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
