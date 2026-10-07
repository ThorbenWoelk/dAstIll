import type { TurnDirection } from "$lib/edition/stories";

/**
 * Pure decisions behind swiping between stories. A swipe to the left turns
 * to the next story, like turning a page; to the right goes back.
 */

/** Movement before a touch counts as a swipe or a scroll. */
export const SWIPE_SLOP_PX = 10;
/** A swipe this far, or this share of the width, always turns. */
const TURN_DISTANCE_PX = 120;
const TURN_SHARE = 0.25;
/** A quick flick turns from a shorter distance. */
const FLICK_DISTANCE_PX = 40;
const FLICK_SPEED_PX_PER_MS = 0.4;
/** Velocity looks at the last part of the movement only. */
const VELOCITY_WINDOW_MS = 100;
/** How far a story with nothing behind it can be pulled. */
const RESIST_MAX_PX = 56;

export type GestureAxis = "undecided" | "horizontal" | "vertical";

export interface TouchSample {
  x: number;
  time: number;
}

/** Mostly sideways movement is a swipe; anything else is left to scrolling. */
export function readGestureAxis(dx: number, dy: number): GestureAxis {
  if (Math.hypot(dx, dy) < SWIPE_SLOP_PX) return "undecided";
  return Math.abs(dx) > Math.abs(dy) * 1.2 ? "horizontal" : "vertical";
}

export function directionOf(dx: number): TurnDirection {
  return dx < 0 ? "next" : "previous";
}

/**
 * How far the story follows the finger. With no story that way it moves a
 * little and resists, so the reader feels the end of the section.
 */
export function followFinger(dx: number, canGo: boolean): number {
  if (canGo) return dx;
  const pulled =
    RESIST_MAX_PX * (1 - Math.exp(-Math.abs(dx) / (RESIST_MAX_PX * 3)));
  return Math.sign(dx) * pulled;
}

/** Horizontal speed over the last moments of the touch, in px per ms. */
export function swipeVelocity(samples: TouchSample[]): number {
  const last = samples.at(-1);
  if (!last) return 0;
  const first =
    samples.find((sample) => last.time - sample.time <= VELOCITY_WINDOW_MS) ??
    last;
  const elapsed = last.time - first.time;
  return elapsed > 0 ? (last.x - first.x) / elapsed : 0;
}

/** At release: the direction to turn, or null to let the story settle back. */
export function settleSwipe(
  dx: number,
  velocity: number,
  width: number,
  canGo: (direction: TurnDirection) => boolean,
): TurnDirection | null {
  if (dx === 0) return null;
  const direction = directionOf(dx);
  if (!canGo(direction)) return null;
  const distance = Math.abs(dx);
  const far = distance >= Math.min(width * TURN_SHARE, TURN_DISTANCE_PX);
  const flicked =
    distance >= FLICK_DISTANCE_PX &&
    Math.sign(velocity) === Math.sign(dx) &&
    Math.abs(velocity) >= FLICK_SPEED_PX_PER_MS;
  return far || flicked ? direction : null;
}
