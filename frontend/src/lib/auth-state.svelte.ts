import type { AuthContext } from "$lib/auth";
import { cloneAuthContext } from "$lib/auth";
import { createSubscriber } from "svelte/reactivity";

type FirebaseUserLike = {
  getIdToken: (forceRefresh?: boolean) => Promise<string>;
};

type AuthController = {
  readonly current: AuthContext;
  readonly ready: boolean;
  readonly syncing: boolean;
  readonly error: string | null;
  setServerAuth(nextAuth: AuthContext): void;
  start(): Promise<void>;
  signInWithGoogle(): Promise<AuthContext>;
  signOut(): Promise<AuthContext>;
};

const DEFAULT_AUTH: AuthContext = {
  userId: null,
  authState: "anonymous",
  accessRole: "anonymous",
  email: null,
};

function normalizeAuthContext(value: AuthContext): AuthContext {
  return cloneAuthContext(value);
}

async function importFirebaseAuthModule() {
  const [{ auth }, firebaseAuth] = await Promise.all([
    import("$lib/firebase"),
    import("firebase/auth"),
  ]);

  return {
    auth,
    GoogleAuthProvider: firebaseAuth.GoogleAuthProvider,
    onAuthStateChanged: firebaseAuth.onAuthStateChanged,
    signInAnonymously: firebaseAuth.signInAnonymously,
    signInWithPopup: firebaseAuth.signInWithPopup,
    signOut: firebaseAuth.signOut,
  };
}

async function parseSessionResponse(response: Response): Promise<AuthContext> {
  let payload: unknown;
  try {
    payload = await response.json();
  } catch {
    payload = null;
  }

  if (!response.ok) {
    const message =
      typeof payload === "object" &&
      payload !== null &&
      "message" in payload &&
      typeof payload.message === "string"
        ? payload.message
        : `Auth request failed (${response.status})`;
    throw new Error(message);
  }

  if (
    typeof payload !== "object" ||
    payload === null ||
    !("authState" in payload) ||
    !("accessRole" in payload)
  ) {
    throw new Error("Auth response payload was malformed.");
  }

  return normalizeAuthContext(payload as AuthContext);
}

async function exchangeUserSession(
  user: FirebaseUserLike,
): Promise<AuthContext> {
  const idToken = await user.getIdToken();
  const response = await fetch("/auth/session", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ idToken }),
  });
  return parseSessionResponse(response);
}

class AuthStateController implements AuthController {
  #current: AuthContext = DEFAULT_AUTH;
  #ready = false;
  #syncing = false;
  #error: string | null = null;
  #started = false;
  #bootstrapPromise: Promise<AuthContext> | null = null;
  #events = new EventTarget();
  #subscribe = createSubscriber((update) => {
    const listener = () => update();
    this.#events.addEventListener("change", listener);
    return () => {
      this.#events.removeEventListener("change", listener);
    };
  });

  #emit() {
    this.#events.dispatchEvent(new Event("change"));
  }

  #setState(
    next: Partial<{
      current: AuthContext;
      ready: boolean;
      syncing: boolean;
      error: string | null;
    }>,
  ) {
    if (next.current) {
      this.#current = normalizeAuthContext(next.current);
    }
    if (next.ready !== undefined) {
      this.#ready = next.ready;
    }
    if (next.syncing !== undefined) {
      this.#syncing = next.syncing;
    }
    if (next.error !== undefined) {
      this.#error = next.error;
    }
    this.#emit();
  }

  get current() {
    this.#subscribe();
    return this.#current;
  }

  get ready() {
    this.#subscribe();
    return this.#ready;
  }

  get syncing() {
    this.#subscribe();
    return this.#syncing;
  }

  get error() {
    this.#subscribe();
    return this.#error;
  }

  setServerAuth(nextAuth: AuthContext) {
    this.#setState({
      current: nextAuth,
      ready: this.#ready || Boolean(nextAuth.userId),
      error: nextAuth.userId ? null : this.#error,
    });
  }

  async #bootstrapAnonymousSession(): Promise<AuthContext> {
    if (this.#bootstrapPromise) {
      return this.#bootstrapPromise;
    }

    this.#bootstrapPromise = (async () => {
      this.#setState({
        syncing: true,
        error: null,
      });

      const {
        auth,
        signInAnonymously,
        signOut: signOutFirebase,
      } = await importFirebaseAuthModule();

      if (auth.currentUser) {
        await signOutFirebase(auth);
      }

      const credential = await signInAnonymously(auth);
      const nextAuth = await exchangeUserSession(credential.user);
      this.#setState({
        current: nextAuth,
        ready: true,
      });
      return nextAuth;
    })()
      .catch((cause) => {
        this.#setState({
          error:
            cause instanceof Error
              ? cause.message
              : "Anonymous auth bootstrap failed.",
          ready: true,
        });
        throw cause;
      })
      .finally(() => {
        this.#bootstrapPromise = null;
        this.#setState({
          syncing: false,
        });
      });

    return this.#bootstrapPromise;
  }

  async start() {
    if (this.#started || typeof window === "undefined") {
      this.#setState({
        ready: this.#ready || Boolean(this.#current.userId),
      });
      return;
    }

    this.#started = true;
    const { auth, onAuthStateChanged } = await importFirebaseAuthModule();
    onAuthStateChanged(auth, () => undefined);

    if (this.#current.userId) {
      this.#setState({
        ready: true,
      });
      return;
    }

    await this.#bootstrapAnonymousSession();
  }

  async signInWithGoogle() {
    this.#setState({
      syncing: true,
      error: null,
    });

    try {
      const { auth, GoogleAuthProvider, signInWithPopup } =
        await importFirebaseAuthModule();
      const provider = new GoogleAuthProvider();
      const credential = await signInWithPopup(auth, provider);
      const nextAuth = await exchangeUserSession(credential.user);
      this.#setState({
        current: nextAuth,
        ready: true,
      });
      return nextAuth;
    } catch (cause) {
      this.#setState({
        error:
          cause instanceof Error ? cause.message : "Google sign-in failed.",
      });
      throw cause;
    } finally {
      this.#setState({
        syncing: false,
      });
    }
  }

  async signOut() {
    const response = await fetch("/auth/session", {
      method: "DELETE",
    });
    await parseSessionResponse(response);

    const { auth, signOut: signOutFirebase } = await importFirebaseAuthModule();
    if (auth.currentUser) {
      await signOutFirebase(auth);
    }

    this.#setState({
      current: DEFAULT_AUTH,
      ready: false,
    });
    return this.#bootstrapAnonymousSession();
  }
}

export const authState = new AuthStateController();
