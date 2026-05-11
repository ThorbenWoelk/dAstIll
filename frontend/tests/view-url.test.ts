import { describe, expect, it } from "bun:test";

import {
  buildWorkspaceViewHref,
  parseWorkspaceViewUrlState,
} from "../src/lib/navigation/view-url";

describe("workspace view URLs", () => {
  it("builds generic source and item params for workspace deep links", () => {
    expect(
      buildWorkspaceViewHref({
        selectedChannelId: "channel-1",
        selectedVideoId: "video-2",
        contentMode: "summary",
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      }),
    ).toBe("/?source=channel-1&item=video-2&content=summary&type=all&ack=all");
  });

  it("parses generic source and item params back into legacy workspace state", () => {
    const url = new URL(
      "https://example.com/?source=source-1&item=item-2&content=transcript&type=short&ack=unack",
    );

    expect(parseWorkspaceViewUrlState(url)).toEqual({
      selectedSourceId: "source-1",
      selectedChannelId: "source-1",
      selectedItemId: "item-2",
      selectedVideoId: "item-2",
      contentMode: "transcript",
      videoTypeFilter: "short",
      acknowledgedFilter: "unack",
    });
  });

  it("keeps legacy channel and video params readable", () => {
    const url = new URL(
      "https://example.com/?channel=channel-1&video=video-2&content=summary&type=all&ack=all",
    );

    expect(parseWorkspaceViewUrlState(url)).toEqual({
      selectedSourceId: "channel-1",
      selectedChannelId: "channel-1",
      selectedItemId: "video-2",
      selectedVideoId: "video-2",
      contentMode: "summary",
      videoTypeFilter: "all",
      acknowledgedFilter: "all",
    });
  });
});
