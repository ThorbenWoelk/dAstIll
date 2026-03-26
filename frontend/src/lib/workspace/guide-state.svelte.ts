/**
 * Shared guide tour state composable for route-level feature guides.
 *
 * Manages open/close state, current step, and URL persistence so each route
 * does not duplicate the same reactive wiring. Tour step definitions remain
 * route-specific - this only handles the state machine.
 */

import { resolveGuideStepFromUrl, writeGuideStepToUrl } from "$lib/utils/guide";

export type GuideState = {
  readonly isOpen: boolean;
  readonly step: number;
  open(): void;
  close(): void;
  setStep(s: number): void;
  restoreFromUrl(): void;
};

/**
 * Creates reactive guide tour state using Svelte 5 runes.
 *
 * Must be called synchronously during component initialization (top-level
 * script block), not inside an async callback or `onMount`.
 */
export function createGuideState(stepCount: number): GuideState {
  let guideOpen = $state(false);
  let currentStep = $state(0);

  return {
    get isOpen() {
      return guideOpen;
    },
    get step() {
      return currentStep;
    },

    open() {
      currentStep = 0;
      guideOpen = true;
      writeGuideStepToUrl(0);
    },

    close() {
      guideOpen = false;
      writeGuideStepToUrl(null);
    },

    setStep(s: number) {
      currentStep = s;
      writeGuideStepToUrl(s);
    },

    restoreFromUrl() {
      const restored = resolveGuideStepFromUrl(
        window.location.search,
        stepCount,
      );
      if (restored !== null) {
        currentStep = restored;
        guideOpen = true;
      }
    },
  };
}
