import { describe, expect, it } from "vitest";
import {
  hasCitationMarkers,
  linkifyCitationMarkers,
} from "../src/lib/utils/chat-citations";
import type { ChatSource } from "../src/lib/types";

const baseSource = (overrides: Partial<ChatSource>): ChatSource => ({
  video_id: "v1",
  channel_id: "c1",
  channel_name: "Ch",
  video_title: "Title",
  source_kind: "transcript",
  section_title: null,
  snippet: "x",
  score: 1,
  chunk_id: "chunk-1",
  ...overrides,
});

describe("chat-citations", () => {
  it("detects numeric bracket markers", () => {
    expect(hasCitationMarkers("See [1] and [2].")).toBe(true);
    expect(hasCitationMarkers("See [Source 1].")).toBe(true);
    expect(hasCitationMarkers("No markers here.")).toBe(false);
  });

  it("linkifies markers when index maps to a source", () => {
    const sources = [
      baseSource({ video_id: "a" }),
      baseSource({ video_id: "b", video_title: "Other" }),
    ];
    const html = "<p>Ref [1] ok</p>";
    const out = linkifyCitationMarkers(html, sources);
    expect(out).toContain('class="chat-cite-sup"');
    expect(out).toContain('class="chat-cite-ref"');
    expect(out).toContain('target="_blank"');
    expect(out).toContain('rel="noopener noreferrer"');
    expect(out).toContain("data-tooltip=");
    expect(out).toContain('href="/?');
    expect(out).toContain("chunk=chunk-1");
    expect(out).toContain("cite=x");
  });

  it("leaves unknown indices unchanged", () => {
    const sources = [baseSource({})];
    const html = "<p>[99]</p>";
    const out = linkifyCitationMarkers(html, sources);
    expect(out).toContain("[99]");
  });
});
