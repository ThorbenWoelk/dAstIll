import { describe, expect, it } from "bun:test";
import {
  defaultEarliestSyncFloorDateInputValue,
  resolveSyncDateInputValue,
} from "../src/lib/workspace/sidebar-sync-date";
import type { Channel, SyncDepth } from "../src/lib/types";

function channel(overrides: Partial<Channel> = {}): Channel {
  return {
    id: "c1",
    handle: null,
    name: "N",
    thumbnail_url: null,
    added_at: "2026-01-01T00:00:00.000Z",
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
    ...overrides,
  };
}

function depth(overrides: Partial<SyncDepth> = {}): SyncDepth {
  return {
    earliest_sync_date: null,
    earliest_sync_date_user_set: false,
    derived_earliest_ready_date: null,
    ...overrides,
  };
}

describe("defaultEarliestSyncFloorDateInputValue", () => {
  it("returns UTC first of month for the given instant", () => {
    expect(
      defaultEarliestSyncFloorDateInputValue(
        new Date("2026-03-15T14:22:00.000Z"),
      ),
    ).toBe("2026-03-01");
    expect(
      defaultEarliestSyncFloorDateInputValue(
        new Date("2025-12-31T23:00:00.000Z"),
      ),
    ).toBe("2025-12-01");
  });
});

describe("resolveSyncDateInputValue", () => {
  it("uses persisted channel date when present", () => {
    expect(
      resolveSyncDateInputValue(
        channel({ earliest_sync_date: "2026-02-10T08:30:00.000Z" }),
        depth(),
        new Date("2026-03-01T00:00:00.000Z"),
      ),
    ).toBe("2026-02-10");
  });

  it("falls back to sync depth when the channel row omits the floor", () => {
    expect(
      resolveSyncDateInputValue(
        channel({ earliest_sync_date: null }),
        depth({ earliest_sync_date: "2026-01-01T00:00:00.000Z" }),
        new Date("2026-03-01T00:00:00.000Z"),
      ),
    ).toBe("2026-01-01");
  });

  it("uses UTC month-start when nothing is persisted (so the date input is not blank)", () => {
    expect(
      resolveSyncDateInputValue(
        channel({ earliest_sync_date: null }),
        depth({ earliest_sync_date: null }),
        new Date("2026-03-28T12:00:00.000Z"),
      ),
    ).toBe("2026-03-01");
  });
});
