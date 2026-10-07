<script lang="ts">
  import type { Section, SectionId } from "$lib/edition/stories";

  let {
    sections,
    active,
    onSelect,
  }: {
    sections: Section[];
    active: SectionId;
    onSelect: (section: SectionId) => void;
  } = $props();

  let bar = $state<HTMLElement | null>(null);
  let placed = false;

  /** On phones the bar scrolls sideways: keep the active section in view. */
  $effect(() => {
    void active;
    const current = bar?.querySelector('[aria-current="page"]');
    if (!bar || !current) return;
    const barBox = bar.getBoundingClientRect();
    const box = current.getBoundingClientRect();
    const offset = box.left + box.width / 2 - (barBox.left + barBox.width / 2);
    const smooth =
      placed && !window.matchMedia("(prefers-reduced-motion: reduce)").matches;
    bar.scrollBy({ left: offset, behavior: smooth ? "smooth" : "auto" });
    placed = true;
  });
</script>

<nav class="sections" aria-label="Sections" bind:this={bar}>
  <ul>
    {#each sections as section (section.id)}
      <li>
        <button
          type="button"
          class="label"
          aria-current={section.id === active ? "page" : undefined}
          onclick={() => onSelect(section.id)}
        >
          {section.name}
          <span class="count">{section.unread}</span>
        </button>
      </li>
    {/each}
  </ul>
</nav>

<style>
  /*
   * Phones: full width, sticks to the top of the screen, and tucks away
   * while the reader scrolls down a story.
   */
  .sections {
    position: sticky;
    top: 0;
    z-index: 5;
    margin: 0 calc(var(--space-4) * -1);
    padding: env(safe-area-inset-top) var(--space-4) 0;
    border-bottom: 1px solid var(--rule);
    background: var(--paper);
    overflow-x: auto;
    scrollbar-width: none;
    transform: translateY(calc(var(--tucked, 0) * -100%));
  }

  @media (prefers-reduced-motion: no-preference) {
    .sections {
      transition: transform var(--tuck-ms) ease;
    }
  }

  .sections::-webkit-scrollbar {
    display: none;
  }

  ul {
    display: flex;
    gap: var(--space-5);
    width: max-content;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  button {
    display: flex;
    align-items: center;
    gap: 6px;
    min-height: var(--touch);
    padding: 0;
    border: 0;
    background: none;
    color: var(--ink-soft);
    white-space: nowrap;
  }

  button:hover {
    color: var(--ink);
  }

  button[aria-current="page"] {
    color: var(--kicker);
  }

  .count {
    font-weight: 400;
    letter-spacing: 0;
    color: var(--ink-faint);
  }

  @media (min-width: 640px) {
    .sections {
      position: static;
      margin: 0;
      padding: 0;
      transform: none;
      transition: none;
    }

    ul {
      flex-wrap: wrap;
      justify-content: center;
      width: auto;
      column-gap: var(--space-6);
    }
  }
</style>
