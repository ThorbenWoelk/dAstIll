export function shouldCloseDrawerForKey(open: boolean, key: string): boolean {
  return open && key === "Escape";
}
