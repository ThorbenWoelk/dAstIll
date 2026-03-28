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

let mockedFirebaseAuth: {
  signInAnonymously: ReturnType<typeof mock>;
  signInWithPopup: ReturnType<typeof mock>;
  signOut: ReturnType<typeof mock>;
  onAuthStateChanged: ReturnType<typeof mock>;
} | null = null;

beforeEach(() => {
  authStateListener = null;
  firebaseAuthInstance.currentUser = null;
});

afterEach(() => {
  authStateListener = null;
  firebaseAuthInstance.currentUser = null;
  if (mockedFirebaseAuth) {
    mockedFirebaseAuth.signInAnonymously.mockClear();
    mockedFirebaseAuth.signInWithPopup.mockClear();
    mockedFirebaseAuth.signOut.mockClear();
    mockedFirebaseAuth.onAuthStateChanged.mockClear();
  }
});

mock.module("$lib/firebase", () => ({
  auth: firebaseAuthInstance,
}));

mock.module("firebase/auth", () => {
  class GoogleAuthProvider {}

  const connectAuthEmulator = mock(() => undefined);

  const getAuth = mock(() => firebaseAuthInstance);

  const onAuthStateChanged = mock(
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

  const signInAnonymously = mock(async () => {
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

  const signInWithPopup = mock(async () => {
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

  const signOut = mock(async () => {
    firebaseAuthInstance.currentUser = null;
    authStateListener?.(null);
  });

  const module = {
    GoogleAuthProvider,
    connectAuthEmulator,
    getAuth,
    onAuthStateChanged,
    signInAnonymously,
    signInWithPopup,
    signOut,
  };

  mockedFirebaseAuth = module;

  return module;
});

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

describe("auth state controller", () => {
  beforeEach(() => {
    Object.defineProperty(globalThis, "window", {
      value: {},
      configurable: true,
    });
  });

  afterEach(() => {
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

    expect(mockedFirebaseAuth!.signInAnonymously).toHaveBeenCalledTimes(1);
    expect(authState.current).toEqual({
      userId: "anon-123",
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });
  });

  it("reuses a cookie-backed anonymous session on start instead of creating a second anonymous user", async () => {
    globalThis.fetch = createFetchMock() as typeof fetch;

    const { authState } = await loadAuthStateModule();

    authState.setServerAuth({
      userId: "anon-123",
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });

    await authState.start();

    expect(mockedFirebaseAuth!.signInAnonymously).not.toHaveBeenCalled();
    expect(authState.current).toEqual({
      userId: "anon-123",
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });
  });

  it("re-bootstraps an anonymous session when a started client later receives anonymous server auth without a user id", async () => {
    globalThis.fetch = createFetchMock() as typeof fetch;

    const { authState } = await loadAuthStateModule();

    authState.setServerAuth({
      userId: null,
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });

    await authState.start();
    expect(mockedFirebaseAuth!.signInAnonymously).toHaveBeenCalledTimes(1);

    authState.setServerAuth({
      userId: null,
      authState: "anonymous",
      accessRole: "anonymous",
      email: null,
    });

    await new Promise((resolve) => setTimeout(resolve, 0));

    expect(mockedFirebaseAuth!.signInAnonymously).toHaveBeenCalledTimes(2);
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

    expect(mockedFirebaseAuth!.signInWithPopup).toHaveBeenCalledTimes(1);
    expect(authState.current).toEqual({
      userId: "google-123",
      authState: "authenticated",
      accessRole: "user",
      email: "person@example.com",
    });
  });
});
