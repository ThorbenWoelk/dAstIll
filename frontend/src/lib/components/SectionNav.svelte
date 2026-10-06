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
</script>

<nav class="sections" aria-label="Sections">
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
  .sections {
    border-bottom: 1px solid var(--rule);
    overflow-x: auto;
    scrollbar-width: none;
  }

  .sections::-webkit-scrollbar {
    display: none;
  }

  ul {
    display: flex;
    gap: var(--space-5);
    margin: 0;
    padding: 0;
    list-style: none;
  }

  button {
    display: flex;
    align-items: baseline;
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
    ul {
      flex-wrap: wrap;
      justify-content: center;
      column-gap: var(--space-6);
    }
  }
</style>
