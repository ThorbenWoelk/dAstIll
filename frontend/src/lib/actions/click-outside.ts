type ClickOutsideOptions = {
  enabled?: boolean;
  onClickOutside: (event: PointerEvent) => void;
};

export function clickOutside(node: HTMLElement, options: ClickOutsideOptions) {
  let currentOptions = options;

  const handlePointerDown = (event: PointerEvent) => {
    if (currentOptions.enabled === false) return;
    if (node.contains(event.target as Node)) return;
    currentOptions.onClickOutside(event);
  };

  document.addEventListener("pointerdown", handlePointerDown);

  return {
    update(nextOptions: ClickOutsideOptions) {
      currentOptions = nextOptions;
    },
    destroy() {
      document.removeEventListener("pointerdown", handlePointerDown);
    },
  };
}
