import { afterEach, beforeEach, describe, expect, it } from "bun:test";

import {
  clearEphemeralThreads,
  conversationToSummary,
  createEmptyEphemeralConversation,
  loadEphemeralThreads,
  saveEphemeralThreads,
} from "../src/lib/chat/ephemeral-session";

const originalSessionStorage = globalThis.sessionStorage;

function createMemoryStorage(): Storage {
  const values = new Map<string, string>();
  return {
    get length() {
      return values.size;
    },
    clear() {
      values.clear();
    },
    getItem(key: string) {
      return values.has(key) ? values.get(key)! : null;
    },
    key(index: number) {
      return [...values.keys()][index] ?? null;
    },
    removeItem(key: string) {
      values.delete(key);
    },
    setItem(key: string, value: string) {
      values.set(key, value);
    },
  };
}

beforeEach(() => {
  Object.defineProperty(globalThis, "sessionStorage", {
    value: createMemoryStorage(),
    configurable: true,
  });
});

afterEach(() => {
  if (originalSessionStorage === undefined) {
    delete (globalThis as typeof globalThis & { sessionStorage?: Storage })
      .sessionStorage;
  } else {
    Object.defineProperty(globalThis, "sessionStorage", {
      value: originalSessionStorage,
      configurable: true,
    });
  }
});

describe("ephemeral-session", () => {
  it("creates a conversation with stable summary shape", () => {
    const conv = createEmptyEphemeralConversation();
    expect(conv.messages).toEqual([]);
    expect(conv.id.startsWith("conv_")).toBe(true);
    const summary = conversationToSummary(conv);
    expect(summary.id).toBe(conv.id);
    expect(summary.title_status).toBe(conv.title_status);
  });

  it("isolates anonymous threads by auth scope", () => {
    const scopeAThreads = [createEmptyEphemeralConversation()];
    const scopeBThreads = [createEmptyEphemeralConversation()];

    saveEphemeralThreads("anonymous:scope-a", scopeAThreads);
    saveEphemeralThreads("anonymous:scope-b", scopeBThreads);

    expect(loadEphemeralThreads("anonymous:scope-a")).toEqual(scopeAThreads);
    expect(loadEphemeralThreads("anonymous:scope-b")).toEqual(scopeBThreads);
  });

  it("migrates legacy unscoped anonymous threads into the active scope", () => {
    const legacyThreads = [createEmptyEphemeralConversation()];

    sessionStorage.setItem(
      "dastill.chat.ephemeralThreads.v1",
      JSON.stringify(legacyThreads),
    );

    expect(loadEphemeralThreads("anonymous:scope-a")).toEqual(legacyThreads);
    expect(
      sessionStorage.getItem("dastill.chat.ephemeralThreads.v1"),
    ).toBeNull();
  });

  it("migrates bootstrap-scoped anonymous threads into the resolved scope", () => {
    const bootstrapThreads = [createEmptyEphemeralConversation()];

    sessionStorage.setItem(
      "dastill.chat.ephemeralThreads.v1:anonymous:bootstrap",
      JSON.stringify(bootstrapThreads),
    );

    expect(loadEphemeralThreads("anonymous:uid-123")).toEqual(bootstrapThreads);
    expect(
      sessionStorage.getItem(
        "dastill.chat.ephemeralThreads.v1:anonymous:bootstrap",
      ),
    ).toBeNull();
  });

  it("clears only the requested auth scope", () => {
    saveEphemeralThreads("anonymous:scope-a", [
      createEmptyEphemeralConversation(),
    ]);
    saveEphemeralThreads("anonymous:scope-b", [
      createEmptyEphemeralConversation(),
    ]);

    clearEphemeralThreads("anonymous:scope-a");

    expect(loadEphemeralThreads("anonymous:scope-a")).toEqual([]);
    expect(loadEphemeralThreads("anonymous:scope-b")).toHaveLength(1);
  });
});
