// See https://svelte.dev/docs/kit/types#app.d.ts
declare global {
  namespace App {
    interface PageState {
      /** A story opened from the front page grid on a wide screen. */
      storyOpen?: boolean;
    }
  }
}

export {};
