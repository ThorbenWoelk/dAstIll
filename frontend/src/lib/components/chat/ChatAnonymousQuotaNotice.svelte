<script lang="ts">
  import { CONTACT_EMAIL } from "$lib/app-config";
</script>

<div class="mt-3 rounded-[var(--radius-md)] bg-[var(--accent-wash)] px-4 py-3">
  <p
    class="text-[10px] font-bold uppercase tracking-[0.12em] text-[var(--accent-strong)]"
  >
    Preview limit reached
  </p>
  <p class="mt-2 text-[14px] leading-relaxed text-[var(--foreground)]">
    This is a showcase - guest chat is limited to give everyone a chance to try
    it.
  </p>
  {#if CONTACT_EMAIL}
    <p class="mt-2 text-[12px] leading-relaxed text-[var(--soft-foreground)]">
      Want to explore further?
      <a
        href="mailto:{CONTACT_EMAIL}"
        target="_blank"
        rel="noopener noreferrer"
        class="text-[var(--accent)] transition-colors hover:text-[var(--accent-strong)]"
        >{CONTACT_EMAIL}</a
      >
    </p>
  {:else}
    <p class="mt-2 text-[12px] leading-relaxed text-[var(--soft-foreground)]">
      Feel free to reach out if you'd like to explore further.
    </p>
  {/if}
</div>
