import { useReaderIdentity, useTokenSource } from "$lib/api";
import {
  FIREBASE_AUTH_EMULATOR_HOST,
  IS_LOCAL,
  readFirebaseConfig,
} from "$lib/config";
import type { Auth, User } from "firebase/auth";

export type SessionStatus = "starting" | "signed-out" | "signed-in";

export interface Reader {
  uid: string;
  email: string | null;
  name: string | null;
}

/**
 * Local dev and Playwright only: a stored fake identity skips Firebase so the
 * reader can run against a mocked API. Never honoured in production builds.
 */
export const LOCAL_SESSION_KEY = "__dastill_e2e_auth";

interface LocalSession {
  userId?: string;
  email?: string | null;
  token?: string | null;
}

function readLocalSession(): LocalSession | null {
  if (!IS_LOCAL || typeof localStorage === "undefined") return null;
  try {
    const raw = localStorage.getItem(LOCAL_SESSION_KEY);
    return raw ? (JSON.parse(raw) as LocalSession) : null;
  } catch {
    return null;
  }
}

function toReader(user: User): Reader {
  return { uid: user.uid, email: user.email, name: user.displayName };
}

const POPUP_FALLBACK_CODES = new Set([
  "auth/popup-blocked",
  "auth/operation-not-supported-in-this-environment",
]);

class Session {
  status = $state<SessionStatus>("starting");
  reader = $state<Reader | null>(null);
  busy = $state(false);
  error = $state<string | null>(null);

  #auth: Auth | null = null;
  #started = false;

  async start() {
    if (this.#started || typeof window === "undefined") return;
    this.#started = true;
    useReaderIdentity(() => this.reader?.uid ?? null);

    const local = readLocalSession();
    if (local) {
      useTokenSource(async () => local.token ?? null);
      this.reader = {
        uid: local.userId?.trim() || "local-reader",
        email: local.email ?? null,
        name: null,
      };
      this.status = "signed-in";
      return;
    }

    try {
      const auth = await this.#loadAuth();
      const { onAuthStateChanged } = await import("firebase/auth");
      onAuthStateChanged(auth, (user) => {
        // Anonymous sessions from the old app do not count as signed in.
        if (user && !user.isAnonymous) {
          this.reader = toReader(user);
          this.status = "signed-in";
        } else {
          this.reader = null;
          this.status = "signed-out";
        }
      });
    } catch (cause) {
      this.error = cause instanceof Error ? cause.message : String(cause);
      this.status = "signed-out";
    }
  }

  async signIn() {
    this.busy = true;
    this.error = null;
    try {
      const auth = await this.#loadAuth();
      const { GoogleAuthProvider, signInWithPopup, signInWithRedirect } =
        await import("firebase/auth");
      const provider = new GoogleAuthProvider();
      try {
        await signInWithPopup(auth, provider);
      } catch (cause) {
        const code = (cause as { code?: string }).code ?? "";
        if (!POPUP_FALLBACK_CODES.has(code)) throw cause;
        await signInWithRedirect(auth, provider);
      }
    } catch (cause) {
      const code = (cause as { code?: string }).code ?? "";
      if (code !== "auth/popup-closed-by-user") {
        this.error = "Sign-in did not work. Please try again.";
      }
    } finally {
      this.busy = false;
    }
  }

  async signOut() {
    const auth = await this.#loadAuth();
    const { signOut } = await import("firebase/auth");
    await signOut(auth);
  }

  async #loadAuth(): Promise<Auth> {
    if (this.#auth) return this.#auth;
    const [{ initializeApp, getApps }, firebaseAuth] = await Promise.all([
      import("firebase/app"),
      import("firebase/auth"),
    ]);
    const app = getApps()[0] ?? initializeApp(readFirebaseConfig());
    const auth = firebaseAuth.getAuth(app);
    if (FIREBASE_AUTH_EMULATOR_HOST && !auth.emulatorConfig) {
      firebaseAuth.connectAuthEmulator(
        auth,
        `http://${FIREBASE_AUTH_EMULATOR_HOST}`,
        { disableWarnings: true },
      );
    }
    useTokenSource(async () => auth.currentUser?.getIdToken() ?? null);
    this.#auth = auth;
    return auth;
  }
}

export const session = new Session();
