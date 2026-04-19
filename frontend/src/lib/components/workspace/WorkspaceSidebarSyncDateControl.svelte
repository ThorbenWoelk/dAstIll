<script lang="ts">
  import { channelOverviewSyncSettingsHref } from "$lib/workspace/sidebar-sync-boundary-link";

  let {
    channelId,
    label,
    wrapperClass = "",
    linkClass = "",
  }: {
    channelId: string;
    label: string;
    wrapperClass?: string;
    linkClass?: string;
  } = $props();

  const settingsHref = $derived(channelOverviewSyncSettingsHref(channelId));
</script>

<div class={wrapperClass}>
  <a
    href={settingsHref}
    class={linkClass}
    aria-label={`Open sync settings. Synced to ${label}`}
  >
    <span class="font-bold uppercase tracking-[0.06em] opacity-70"
      >Synced to</span
    >
    <span class="text-[var(--foreground)]">{label}</span>
    <span class="font-bold uppercase tracking-[0.06em] text-[var(--accent)]"
      >Sync settings</span
    >
  </a>
</div>
