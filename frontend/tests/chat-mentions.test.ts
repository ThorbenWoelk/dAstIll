import { describe, expect, it } from "vitest";

import {
  buildResolvedMentionLabel,
  extractChatMentions,
  parseChatMentionSegments,
  pickExactMentionSuggestion,
  type ChatMentionToken,
} from "../src/lib/chat-mentions";

describe("chat mentions", () => {
  it("parses channel and video mentions into ordered segments", () => {
    expect(
      parseChatMentionSegments(
        "compare @{HealthyGamerGG} with +{3 Things Every Relationship Has}",
      ),
    ).toEqual([
      { type: "text", value: "compare " },
      {
        type: "mention",
        mention: {
          kind: "channel",
          raw: "@{HealthyGamerGG}",
          query: "HealthyGamerGG",
        },
      },
      { type: "text", value: " with " },
      {
        type: "mention",
        mention: {
          kind: "video",
          raw: "+{3 Things Every Relationship Has}",
          query: "3 Things Every Relationship Has",
        },
      },
    ]);
  });

  it("extracts mentions without surrounding text noise", () => {
    expect(
      extractChatMentions(
        "read +{3 Things Every Relationship Has} for @{HealthyGamerGG}",
      ),
    ).toEqual<ChatMentionToken[]>([
      {
        kind: "video",
        raw: "+{3 Things Every Relationship Has}",
        query: "3 Things Every Relationship Has",
      },
      {
        kind: "channel",
        raw: "@{HealthyGamerGG}",
        query: "HealthyGamerGG",
      },
    ]);
  });

  it("prefers exact case-insensitive suggestion matches", () => {
    const mention: ChatMentionToken = {
      kind: "video",
      raw: "+{3 Things Every Relationship Has}",
      query: "3 Things Every Relationship Has",
    };

    expect(
      pickExactMentionSuggestion(mention, [
        {
          label: "3 Things Every Relationship Has",
          subtitle: "HealthyGamerGG",
        },
        { label: "3 Things", subtitle: "Other Channel" },
      ]),
    ).toEqual({
      label: "3 Things Every Relationship Has",
      subtitle: "HealthyGamerGG",
    });
  });

  it("builds the minimal display label for video mentions", () => {
    expect(
      buildResolvedMentionLabel(
        "video",
        "3 Things Every Relationship Has",
        "HealthyGamerGG",
      ),
    ).toBe("HealthyGamerGG - 3 Things Every Relationship Has");
    expect(buildResolvedMentionLabel("channel", "HealthyGamerGG")).toBe(
      "HealthyGamerGG",
    );
  });
});
