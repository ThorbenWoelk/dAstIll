import type { AuthContext } from "$lib/auth";

// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
  namespace App {
    // interface Error {}
    interface Locals {
      auth: AuthContext;
      isOperator: boolean;
    }
    interface PageData {
      auth?: AuthContext;
      isOperator?: boolean;
    }
    // interface PageState {}
    // interface Platform {}
  }
}

export {};
