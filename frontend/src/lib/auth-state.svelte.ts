import type { AuthContext } from "$lib/auth";
import {
  buildAnonymousAuthContext,
  buildAuthenticatedAuthContext,
  cloneAuthContext,
} from "$lib/auth";
import { resetApiCacheForAuthChange } from "$lib/api-cache-reset";
import { getAuthStorageScopeKey } from "$lib/auth-storage";
import { configureAuthTokenResolver } from "$lib/auth-token";
import { createSubscriber } from "svelte/reactivity";

type FirebaseUserLike = {
  uid: string;
  email: string | null;
  isAnonymous: boolean;
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

function maybeResetAuthScopedCaches(
  previousAuth: AuthContext,
  nextAuth: AuthContext,
) {
  if (
    getAuthStorageScopeKey(previousAuth) !== getAuthStorageScopeKey(nextAuth)
  ) {
    resetApiCacheForAuthChange();
  }
}

async function importFirebaseAuthModule() {
  const [{ auth }, firebaseAuth] = await Promise.all([
    import("$lib/firebase"),
    import("firebase/auth"),
  ]);

  configureAuthTokenResolver(async () => {
    const user = auth.currentUser;
    return user ? user.getIdToken() : null;
  });

  return {
    auth,
    GoogleAuthProvider: firebaseAuth.GoogleAuthProvider,
    onAuthStateChanged: firebaseAuth.onAuthStateChanged,
    signInAnonymously: firebaseAuth.signInAnonymously,
    signInWithPopup: firebaseAuth.signInWithPopup,
    signOut: firebaseAuth.signOut,
  };
}

function buildAuthContextFromFirebaseUser(user: FirebaseUserLike): AuthContext {
  if (user.isAnonymous) {
    return buildAnonymousAuthContext(user.uid);
  }

  return buildAuthenticatedAuthContext(user.uid, user.email);
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
      const normalizedCurrent = normalizeAuthContext(next.current);
      maybeResetAuthScopedCaches(this.#current, normalizedCurrent);
      this.#current = normalizedCurrent;
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
    const normalizedAuth = normalizeAuthContext(nextAuth);
    const shouldPreserveEstablishedClientSession =
      typeof window !== "undefined" &&
      normalizedAuth.authState === "anonymous" &&
      normalizedAuth.userId === null &&
      this.#current.userId !== null;

    // `data.auth` can be omitted when no server-side auth handshake exists.
    // Once Firebase has established a local session, do not treat that
    // anonymous placeholder as an instruction to sign the user out.
    if (shouldPreserveEstablishedClientSession) {
      return;
    }

    const shouldRebootstrapAnonymousSession =
      typeof window !== "undefined" &&
      this.#started &&
      this.#ready &&
      !this.#syncing &&
      this.#bootstrapPromise === null &&
      normalizedAuth.userId === null;

    this.#setState({
      current: normalizedAuth,
      ready: this.#ready || Boolean(normalizedAuth.userId),
      error: normalizedAuth.userId ? null : this.#error,
    });

    if (shouldRebootstrapAnonymousSession) {
      void this.#bootstrapAnonymousSession().catch(() => undefined);
    }
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

      try {
        const credential = await signInAnonymously(auth);
        const nextAuth = buildAuthContextFromFirebaseUser(credential.user);
        this.#setState({
          current: nextAuth,
          ready: true,
        });
        return nextAuth;
      } catch (firebaseError) {
        const isNetworkError =
          firebaseError instanceof Error &&
          (firebaseError.message.includes("network") ||
            firebaseError.message.includes("fetch") ||
            firebaseError.message.includes("connection") ||
            firebaseError.message.includes("timeout"));

        if (isNetworkError) {
          this.#setState({
            error: null,
            ready: true,
          });
          return this.#current;
        }

        throw firebaseError;
      }
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
    onAuthStateChanged(auth, (user) => {
      if (user) {
        this.#setState({
          current: buildAuthContextFromFirebaseUser(user as FirebaseUserLike),
          ready: true,
          error: null,
        });
        return;
      }

      if (!this.#syncing && this.#bootstrapPromise === null) {
        void this.#bootstrapAnonymousSession().catch(() => undefined);
      }
    });

    if (auth.currentUser) {
      this.#setState({
        current: buildAuthContextFromFirebaseUser(
          auth.currentUser as FirebaseUserLike,
        ),
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
      const { auth, GoogleAuthProvider, signInWithPopup, signOut } =
        await importFirebaseAuthModule();
      if (auth.currentUser?.isAnonymous) {
        await signOut(auth);
      }
      const provider = new GoogleAuthProvider();
      const credential = await signInWithPopup(auth, provider);
      const nextAuth = buildAuthContextFromFirebaseUser(credential.user);
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
