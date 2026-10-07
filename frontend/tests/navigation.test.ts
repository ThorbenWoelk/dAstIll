import { describe, expect, it } from "bun:test";

import { describeWayBack } from "../src/lib/navigation/pages";
import {
  directionOf,
  followFinger,
  readGestureAxis,
  settleSwipe,
  swipeVelocity,
} from "../src/lib/navigation/swipe";
import { followScroll, type ScrollFollow } from "../src/lib/navigation/tuck";

describe("describeWayBack", () => {
  it("names the paper page the reader came from", () => {
    expect(describeWayBack("/")).toBe("Front page");
    expect(describeWayBack("/finished")).toBe("Finished");
    expect(describeWayBack("/highlights/")).toBe("Highlights");
  });

  it("falls back to a plain Back for anything else", () => {
    expect(describeWayBack("/stories/v-1")).toBe("Back");
  });
});

describe("readGestureAxis", () => {
  it("waits until the finger has moved a little", () => {
    expect(readGestureAxis(4, 3)).toBe("undecided");
  });

  it("treats mostly sideways movement as a swipe", () => {
    expect(readGestureAxis(-30, 8)).toBe("horizontal");
  });

  it("leaves vertical and diagonal movement to scrolling", () => {
    expect(readGestureAxis(3, 40)).toBe("vertical");
    expect(readGestureAxis(20, 20)).toBe("vertical");
  });
});

describe("directionOf", () => {
  it("turns to the next story on a swipe to the left", () => {
    expect(directionOf(-50)).toBe("next");
    expect(directionOf(50)).toBe("previous");
  });
});

describe("followFinger", () => {
  it("follows the finger when there is a story that way", () => {
    expect(followFinger(-140, true)).toBe(-140);
  });

  it("resists, and never passes a small limit, when there is none", () => {
    const short = followFinger(40, false);
    const long = followFinger(400, false);
    expect(short).toBeGreaterThan(0);
    expect(short).toBeLessThan(40);
    expect(long).toBeGreaterThan(short);
    expect(long).toBeLessThan(56);
    expect(followFinger(-400, false)).toBe(-long);
  });
});

describe("swipeVelocity", () => {
  it("measures the last moments of the movement", () => {
    expect(
      swipeVelocity([
        { x: 300, time: 0 },
        { x: 290, time: 400 },
        { x: 250, time: 450 },
        { x: 190, time: 500 },
      ]),
    ).toBe(-1);
  });

  it("is zero without movement over time", () => {
    expect(swipeVelocity([])).toBe(0);
    expect(swipeVelocity([{ x: 10, time: 5 }])).toBe(0);
  });
});

describe("settleSwipe", () => {
  const anyWay = () => true;

  it("turns after a long enough swipe", () => {
    expect(settleSwipe(-100, 0, 375, anyWay)).toBe("next");
    expect(settleSwipe(130, 0, 1024, anyWay)).toBe("previous");
  });

  it("turns after a short quick flick", () => {
    expect(settleSwipe(-50, -0.8, 375, anyWay)).toBe("next");
  });

  it("settles back after a short slow swipe or a flick the other way", () => {
    expect(settleSwipe(-50, -0.1, 375, anyWay)).toBeNull();
    expect(settleSwipe(-50, 0.8, 375, anyWay)).toBeNull();
  });

  it("settles back when there is no story that way", () => {
    expect(
      settleSwipe(200, 1, 375, (direction) => direction === "next"),
    ).toBeNull();
  });
});

describe("followScroll", () => {
  const VIEWPORT = 800;
  const PAGE = 5000;
  const scrolled = (state: ScrollFollow, ...ys: number[]) =>
    ys.reduce((next, y) => followScroll(next, y, VIEWPORT, PAGE), state);
  const start: ScrollFollow = { tucked: false, lastY: 0, travel: 0 };

  it("tucks the bars away while scrolling down a story", () => {
    expect(scrolled(start, 100, 300, 400).tucked).toBe(true);
  });

  it("brings them back on a short scroll up", () => {
    const reading = scrolled(start, 300, 600);
    expect(reading.tucked).toBe(true);
    expect(scrolled(reading, 590).tucked).toBe(true);
    expect(scrolled(reading, 590, 570).tucked).toBe(false);
  });

  it("always shows them near the top and at the end of the page", () => {
    const reading = scrolled(start, 300, 600);
    expect(scrolled(reading, 200).tucked).toBe(false);
    expect(scrolled(reading, 900, PAGE - VIEWPORT).tucked).toBe(false);
  });

  it("ignores jumps such as a restored scroll position", () => {
    expect(scrolled(start, 2400).tucked).toBe(false);
  });
});
