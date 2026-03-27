/// <reference types="vite/client" />

interface ImportMetaEnv {
  readonly FIREBASE_AUTH_EMULATOR_HOST?: string;
}

interface ImportMeta {
  readonly env: ImportMetaEnv;
}
