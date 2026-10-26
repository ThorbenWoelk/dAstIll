<script lang="ts">
  import { tick, type Snippet } from "svelte";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";

  let {
    title,
    level = 2,
    headingId,
    passageSkip = false,
    children,
  }: {
    title: string;
    /** 1 to 3: the heading tag. */
    level?: number;
    headingId?: string;
    /** Leave the heading out of the highlight passage. */
    passageSkip?: boolean;
    children: Snippet;
  } = $props();

  const contentId = $props.id();
  let open = $state(true);
  let section = $state<HTMLElement | null>(null);
  let heading = $state<HTMLElement | null>(null);

  /**
   * Folding a section the reader is scrolled into would leave them
   * somewhere further down. Bring its heading back to the top instead, so
   * the next section follows right under it.
   */
  async function toggle() {
    open = !open;
    if (open || !section || !heading) return;
    const scrolledInto =
      section.getBoundingClientRect().top <
      heading.getBoundingClientRect().top - 1;
    await tick();
    if (scrolledInto) section.scrollIntoView({ block: "start" });
  }
</script>

<section class="story-section" bind:this={section}>
  <svelte:element
    this={`h${Math.min(Math.max(level, 1), 3)}`}
    id={headingId}
    class="heading"
    data-passage-skip={passageSkip ? "" : undefined}
    bind:this={heading}
  >
    <button
      type="button"
      aria-expanded={open}
      aria-controls={contentId}
      onclick={toggle}
    >
      <span>{title}</span>
      <span class="chevron"><ChevronIcon /></span>
    </button>
  </svelte:element>
  <div id={contentId} class="content" hidden={!open}>
    {@render children()}
  </div>
</section>

<style>
  /*
   * The heading sticks to the top while its section is on screen. Pages
   * with a sticky bar above set --story-heading-top to stay below it.
   */
  .story-section {
    scroll-margin-top: calc(
      env(safe-area-inset-top) + var(--story-heading-top, 0px)
    );
  }

  .heading {
    position: sticky;
    top: calc(env(safe-area-inset-top) + var(--story-heading-top, 0px));
    z-index: 1;
    margin: var(--space-5) 0 var(--space-2);
    border-bottom: 1px solid var(--hairline);
    background: var(--paper);
  }

  @media (prefers-reduced-motion: no-preference) {
    .heading {
      transition: top var(--tuck-ms) ease;
    }
  }

  .story-section:first-child .heading {
    margin-top: 0;
  }

  button {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: var(--space-3);
    width: 100%;
    min-height: var(--touch);
    padding: 0;
    border: 0;
    background: none;
    font-family: var(--sans);
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.12em;
    text-align: left;
    text-transform: uppercase;
    color: inherit;
  }

  h3 button {
    letter-spacing: 0.08em;
    color: var(--ink-soft);
  }

  button:hover {
    color: var(--kicker);
  }

  .chevron {
    display: inline-flex;
    flex: none;
    color: var(--ink-faint);
  }

  button[aria-expanded="false"] .chevron {
    transform: rotate(-90deg);
  }

  .content[hidden] {
    display: none;
  }
</style>
