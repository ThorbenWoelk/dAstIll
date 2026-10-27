// See https://svelte.dev/docs/kit/types#app.d.ts
declare global {
  namespace App {
    interface PageState {
      /** A story opened from the front page grid on a wide screen. */
      storyOpen?: boolean;
      /** That story's id, so coming back shows it again. */
      storyId?: string;
    }
  }
}

export {};
