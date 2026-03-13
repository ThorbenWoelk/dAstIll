<script lang="ts">
  import { browser } from "$app/environment";
  import { onMount } from "svelte";

  import { resolveVisualViewportBottomInset } from "$lib/mobile-shell/viewport";

  const VIEWPORT_OFFSET_VAR = "--mobile-viewport-offset-bottom";

  function syncViewportOffset() {
    if (!browser) return;

    const inset = resolveVisualViewportBottomInset({
      innerHeight: window.innerHeight,
      visualViewportHeight: window.visualViewport?.height ?? null,
      visualViewportOffsetTop: window.visualViewport?.offsetTop ?? null,
    });

    document.documentElement.style.setProperty(
      VIEWPORT_OFFSET_VAR,
      `${inset}px`,
    );
  }

  onMount(() => {
    if (!browser) return;

    const visualViewport = window.visualViewport;

    syncViewportOffset();

    window.addEventListener("resize", syncViewportOffset);
    visualViewport?.addEventListener("resize", syncViewportOffset);
    visualViewport?.addEventListener("scroll", syncViewportOffset);

    return () => {
      window.removeEventListener("resize", syncViewportOffset);
      visualViewport?.removeEventListener("resize", syncViewportOffset);
      visualViewport?.removeEventListener("scroll", syncViewportOffset);
      document.documentElement.style.removeProperty(VIEWPORT_OFFSET_VAR);
    };
  });
</script>
