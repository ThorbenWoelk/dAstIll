import { describe, expect, it } from "bun:test";
import {
  buildQueueViewHref,
  buildWorkspaceViewHref,
  mergeQueueViewState,
  mergeWorkspaceViewState,
  parseQueueViewUrlState,
  parseWorkspaceViewUrlState,
} from "../src/lib/view-url";

describe("parseWorkspaceViewUrlState", () => {
  it("restores a shared workspace view from query params", () => {
    const url = new URL(
      "https://example.com/?channel=abc&video=vid-1&content=highlights&type=short&ack=ack",
    );

    expect(parseWorkspaceViewUrlState(url)).toEqual({
      selectedChannelId: "abc",
      selectedVideoId: "vid-1",
      contentMode: "highlights",
      videoTypeFilter: "short",
      acknowledgedFilter: "ack",
    });
  });

  it("ignores invalid workspace params", () => {
    const url = new URL(
      "https://example.com/?channel=&video=&content=nope&type=wat&ack=maybe",
    );

    expect(parseWorkspaceViewUrlState(url)).toEqual({});
  });
});

describe("buildWorkspaceViewHref", () => {
  it("serializes the current workspace view into a shareable url", () => {
    expect(
      buildWorkspaceViewHref({
        selectedChannelId: "abc",
        selectedVideoId: "vid-1",
        contentMode: "highlights",
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      }),
    ).toBe("/?channel=abc&video=vid-1&content=highlights&type=all&ack=all");
  });

  it("includes chunk and cite for chat citation deep links", () => {
    expect(
      buildWorkspaceViewHref({
        selectedChannelId: "abc",
        selectedVideoId: "vid-1",
        contentMode: "transcript",
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
        chunkId: "idx-42",
        citeQuery: "hello world",
      }),
    ).toBe(
      "/?channel=abc&video=vid-1&content=transcript&type=all&ack=all&chunk=idx-42&cite=hello+world",
    );
  });
});

describe("mergeWorkspaceViewState", () => {
  it("prefers explicit url state over restored local state", () => {
    expect(
      mergeWorkspaceViewState(
        {
          selectedChannelId: "saved-channel",
          selectedVideoId: "saved-video",
          contentMode: "transcript",
          videoTypeFilter: "all",
          acknowledgedFilter: "all",
          channelOrder: ["saved-channel"],
        },
        {
          selectedChannelId: "url-channel",
          selectedVideoId: "url-video",
          contentMode: "highlights",
          videoTypeFilter: "short",
          acknowledgedFilter: "ack",
        },
      ),
    ).toEqual({
      selectedChannelId: "url-channel",
      selectedVideoId: "url-video",
      contentMode: "highlights",
      videoTypeFilter: "short",
      acknowledgedFilter: "ack",
      channelOrder: ["saved-channel"],
    });
  });
});

describe("parseQueueViewUrlState", () => {
  it("restores the selected queue channel and tab from query params", () => {
    const url = new URL(
      "https://example.com/download-queue?channel=abc&queue=summaries",
    );

    expect(parseQueueViewUrlState(url)).toEqual({
      selectedChannelId: "abc",
      queueTab: "summaries",
    });
  });

  it("restores video selection and browse filters from query params", () => {
    const url = new URL(
      "https://example.com/download-queue?channel=abc&queue=evaluations&video=vid-9&type=short&ack=ack",
    );

    expect(parseQueueViewUrlState(url)).toEqual({
      selectedChannelId: "abc",
      queueTab: "evaluations",
      selectedVideoId: "vid-9",
      videoTypeFilter: "short",
      acknowledgedFilter: "ack",
    });
  });

  it("ignores invalid queue params", () => {
    const url = new URL(
      "https://example.com/download-queue?channel=&queue=wat",
    );

    expect(parseQueueViewUrlState(url)).toEqual({});
  });
});

describe("buildQueueViewHref", () => {
  it("serializes the current queue view into a shareable url", () => {
    expect(
      buildQueueViewHref({
        selectedChannelId: "abc",
        queueTab: "evaluations",
        selectedVideoId: null,
        videoTypeFilter: "all",
        acknowledgedFilter: "all",
      }),
    ).toBe("/download-queue?channel=abc&queue=evaluations&type=all&ack=all");
  });

  it("includes optional video and non-default filters", () => {
    expect(
      buildQueueViewHref({
        selectedChannelId: "abc",
        queueTab: "transcripts",
        selectedVideoId: "vid-1",
        videoTypeFilter: "long",
        acknowledgedFilter: "unack",
      }),
    ).toBe(
      "/download-queue?channel=abc&queue=transcripts&video=vid-1&type=long&ack=unack",
    );
  });
});

describe("mergeQueueViewState", () => {
  it("prefers explicit url state over restored queue state", () => {
    expect(
      mergeQueueViewState(
        {
          selectedChannelId: "saved-channel",
          channelOrder: ["saved-channel"],
          channelSortMode: "alpha",
        },
        {
          selectedChannelId: "url-channel",
          queueTab: "summaries",
        },
      ),
    ).toEqual({
      selectedChannelId: "url-channel",
      channelOrder: ["saved-channel"],
      channelSortMode: "alpha",
      queueTab: "summaries",
    });
  });
});
