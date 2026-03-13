export function resolveVisualViewportBottomInset({
  innerHeight,
  visualViewportHeight,
  visualViewportOffsetTop,
}: {
  innerHeight: number;
  visualViewportHeight: number | null;
  visualViewportOffsetTop: number | null;
}): number {
  if (!Number.isFinite(innerHeight) || innerHeight <= 0) {
    return 0;
  }

  if (
    visualViewportHeight === null ||
    !Number.isFinite(visualViewportHeight) ||
    visualViewportHeight <= 0
  ) {
    return 0;
  }

  const offsetTop =
    visualViewportOffsetTop !== null &&
    Number.isFinite(visualViewportOffsetTop) &&
    visualViewportOffsetTop > 0
      ? visualViewportOffsetTop
      : 0;

  return Math.max(
    0,
    Math.round(innerHeight - (visualViewportHeight + offsetTop)),
  );
}
