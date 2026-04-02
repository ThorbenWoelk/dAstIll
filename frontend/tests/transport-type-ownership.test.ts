import { describe, expect, it } from "bun:test";
import { readFileSync } from "node:fs";
import { join } from "node:path";

const repoFile = (...parts: string[]) =>
  readFileSync(join(import.meta.dir, "..", ...parts), "utf8");

const HANDWRITTEN_TRANSPORT_INTERFACES = [
  "Channel",
  "SyncDepth",
  "ChannelSnapshot",
  "ChannelVideoPage",
  "WorkspaceBootstrap",
  "AiHealthResponse",
  "Video",
  "AddVideoResult",
  "Transcript",
  "CleanTranscriptResponse",
  "Highlight",
  "CreateHighlightRequest",
  "HighlightVideoGroup",
  "HighlightChannelGroup",
  "Summary",
  "SearchMatch",
  "SearchResult",
  "SearchResponse",
  "SearchStatus",
];

describe("transport type ownership", () => {
  it("keeps backend-owned transport DTOs out of handwritten interface definitions", () => {
    const typesSource = repoFile("src", "lib", "types.ts");

    for (const name of HANDWRITTEN_TRANSPORT_INTERFACES) {
      expect(typesSource).not.toMatch(
        new RegExp(`export interface ${name}\\b`),
      );
    }

    expect(typesSource).toContain('from "./transport-types"');
  });

  it("defines transport compatibility types in a dedicated module backed by generated bindings", () => {
    const transportTypesSource = repoFile("src", "lib", "transport-types.ts");

    expect(transportTypesSource).toContain('from "./bindings/Channel"');
    expect(transportTypesSource).toContain(
      'from "./bindings/WorkspaceBootstrapPayload"',
    );
    expect(transportTypesSource).toContain("export type Channel =");
    expect(transportTypesSource).toContain("export type WorkspaceBootstrap =");
  });
});
