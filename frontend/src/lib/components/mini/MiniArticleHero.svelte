<script lang="ts">
  interface Props {
    src: string | null | undefined;
    alt: string;
  }

  let { src, alt }: Props = $props();
</script>

{#if src}
  <div class="hero">
    <img class="hero-img" {src} {alt} />
  </div>
{/if}

<style>
  .hero {
    margin-bottom: var(--space-lg);
    border-radius: var(--radius-md);
    overflow: hidden;
    background: var(--muted);
  }
  .hero-img {
    display: block;
    width: 100%;
    aspect-ratio: 16 / 9;
    object-fit: cover;
  }
</style>
