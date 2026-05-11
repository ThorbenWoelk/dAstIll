import { describe, expect, it } from "bun:test";

import {
  getUserErrorMessage,
  normalizeUserErrorMessage,
} from "../src/lib/api/user-error";

describe("normalizeUserErrorMessage", () => {
  it("rewrites technical TTS configuration errors", () => {
    expect(normalizeUserErrorMessage("Polly TTS is not configured")).toBe(
      "Sorry, audio playback is not available right now.",
    );
  });

  it("rewrites infrastructure failures to a plain fallback", () => {
    expect(
      normalizeUserErrorMessage("S3 error: bucket missing", { status: 503 }),
    ).toBe("Sorry, that part of the app is unavailable right now.");
  });

  it("keeps already-plain not-found messages user friendly", () => {
    expect(normalizeUserErrorMessage("Conversation not found")).toBe(
      "That conversation could not be found.",
    );
  });

  it("rewrites AI subscription capacity errors", () => {
    expect(
      normalizeUserErrorMessage("this model requires a subscription", {
        status: 403,
      }),
    ).toBe("AI model quota is used up. Please try again later.");
  });

  it("keeps Ollama Cloud limit hits clear", () => {
    expect(
      normalizeUserErrorMessage(
        "Ollama Cloud usage limit reached. The summary will retry when capacity returns.",
        { status: 429 },
      ),
    ).toBe("AI model quota is used up. Please try again later.");
  });

  it("keeps generic AI unavailability distinct from quota", () => {
    expect(
      normalizeUserErrorMessage(
        "AI generation is temporarily unavailable. The summary will retry when capacity returns.",
        { status: 503 },
      ),
    ).toBe("AI is temporarily unavailable. Please try again later.");
  });

  it("uses status-based fallback for generic request errors", () => {
    expect(
      normalizeUserErrorMessage("Request failed (500)", { status: 500 }),
    ).toBe("Sorry, that part of the app is unavailable right now.");
  });
});

describe("getUserErrorMessage", () => {
  it("extracts and normalizes Error instances", () => {
    expect(
      getUserErrorMessage(new Error("Ollama not available"), { status: 503 }),
    ).toBe("Sorry, AI features are not available right now.");
  });
});
