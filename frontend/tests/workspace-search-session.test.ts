import { describe, expect, it } from "bun:test";

import {
  createWorkspaceSearchSessionState,
  readWorkspaceSearchSession,
  writeWorkspaceSearchSession,
} from "../src/lib/workspace-search-session";

function createMemoryStorage() {
  const values = new Map<string, string>();

  return {
    getItem(key: string) {
      return values.get(key) ?? null;
    },
    setItem(key: string, value: string) {
      values.set(key, value);
    },
  };
}

describe("workspace search session storage", () => {
  it("returns the default state when nothing has been persisted", () => {
    expect(readWorkspaceSearchSession(createMemoryStorage())).toEqual(
      createWorkspaceSearchSessionState(),
    );
  });

  it("round-trips a retained search across storage", () => {
    const storage = createMemoryStorage();

    writeWorkspaceSearchSession(storage, {
      query: "vector search",
      retainedQuery: "vector search",
      source: "summary",
      sections: {
        keyword: {
          results: [
            {
              video_id: "video-1",
              channel_id: "channel-1",
              channel_name: "Channel 1",
              video_title: "Vector Search Basics",
              published_at: "2026-03-18T00:00:00Z",
              matches: [
                {
                  source: "summary",
                  snippet: "A concise vector-search overview.",
                  score: 0.91,
                },
              ],
            },
          ],
          loading: false,
          error: null,
        },
        semantic: {
          results: [],
          loading: false,
          error: null,
        },
      },
      modeKeyword: true,
      modeSemantic: false,
    });

    expect(readWorkspaceSearchSession(storage)).toEqual({
      query: "vector search",
      retainedQuery: "vector search",
      source: "summary",
      sections: {
        keyword: {
          results: [
            {
              video_id: "video-1",
              channel_id: "channel-1",
              channel_name: "Channel 1",
              video_title: "Vector Search Basics",
              published_at: "2026-03-18T00:00:00Z",
              matches: [
                {
                  source: "summary",
                  snippet: "A concise vector-search overview.",
                  score: 0.91,
                },
              ],
            },
          ],
          loading: false,
          error: null,
        },
        semantic: {
          results: [],
          loading: false,
          error: null,
        },
      },
      modeKeyword: true,
      modeSemantic: false,
    });
  });

  it("falls back to the default state when the stored payload is malformed", () => {
    const storage = createMemoryStorage();
    storage.setItem(
      "workspace-search-session",
      JSON.stringify({
        query: 42,
        retainedQuery: null,
        source: "invalid",
        sections: {
          keyword: {
            results: [{ bad: true }],
            loading: "yes",
            error: 7,
          },
        },
        modeKeyword: "nope",
      }),
    );

    expect(readWorkspaceSearchSession(storage)).toEqual(
      createWorkspaceSearchSessionState(),
    );
  });
});
