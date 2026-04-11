export function shouldRunSidebarPreviewSlideTransition({
  mobileVisible,
  desktopViewport,
}: {
  mobileVisible: boolean;
  desktopViewport: boolean;
}): boolean {
  return mobileVisible || desktopViewport;
}
