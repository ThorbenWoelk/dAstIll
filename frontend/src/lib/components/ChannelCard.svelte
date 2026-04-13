<script lang="ts">
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import ChevronIcon from "$lib/components/icons/ChevronIcon.svelte";
  import type { Channel } from "$lib/types";

  let {
    channel,
    active = false,
    draggableEnabled = false,
    dragging = false,
    dragOver = false,
    loading = false,
    trailingSpace = "none",
    onSelect = () => {},
    onDragStart = () => {},
    onDragOver = () => {},
    onDrop = () => {},
    onDragEnd = () => {},
    expanded = undefined,
  }: {
    channel: Channel;
    active?: boolean;
    draggableEnabled?: boolean;
    dragging?: boolean;
    dragOver?: boolean;
    loading?: boolean;
    trailingSpace?: "none" | "compact" | "wide";
    onSelect?: () => void;
    onDragStart?: (event: DragEvent) => void;
    onDragOver?: (event: DragEvent) => void;
    onDrop?: (event: DragEvent) => void;
    onDragEnd?: (event: DragEvent) => void;
    expanded?: boolean;
  } = $props();

  const normalizeThumbnail = (thumbnailUrl?: string | null): string | null => {
    const trimmed = thumbnailUrl?.trim();
    return trimmed ? trimmed : null;
  };

  let avatarLoadFailed = $state(false);
  let thumbnailUrl = $derived(normalizeThumbnail(channel.thumbnail_url));
  $effect(() => {
    channel.id;
    thumbnailUrl;
    avatarLoadFailed = false;
  });
  let avatarUrl = $derived(
    !avatarLoadFailed && thumbnailUrl ? thumbnailUrl : defaultChannelIcon,
  );
  let trailingSpaceClass = $derived(
    trailingSpace === "wide"
      ? "pr-28"
      : trailingSpace === "compact"
        ? "pr-12"
        : "",
  );

  function handleAvatarError() {
    avatarLoadFailed = true;
  }
</script>

<button
  type="button"
  draggable={draggableEnabled}
  ondragstart={onDragStart}
  ondragover={onDragOver}
  ondrop={onDrop}
  ondragend={onDragEnd}
  class={`group relative flex w-full min-w-0 items-center gap-3 rounded-[18px] px-3 py-2.5 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${trailingSpaceClass} ${
    active
      ? "bg-[var(--panel-surface)] shadow-[0_10px_30px_color-mix(in_srgb,var(--foreground)_8%,transparent)]"
      : "hover:bg-[var(--accent-wash)]"
  } ${dragging || loading ? "opacity-40" : ""} ${dragOver ? "shadow-[0_0_0_1px_var(--border-soft),0_16px_34px_color-mix(in_srgb,var(--foreground)_10%,transparent)]" : ""} ${loading ? "animate-pulse" : ""} ${draggableEnabled ? (dragging ? "cursor-grabbing" : "cursor-grab") : ""}`}
  onclick={onSelect}
  disabled={loading}
>
  <div
    class="h-10 w-10 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
  >
    <img
      src={avatarUrl}
      alt={channel.name}
      width="32"
      height="32"
      loading="lazy"
      referrerpolicy="no-referrer"
      class="h-full w-full object-cover"
      onerror={handleAvatarError}
    />
  </div>
  <div class="min-w-0 flex-1">
    <p
      class="truncate text-[13px] font-semibold leading-tight tracking-tight text-[var(--foreground)]"
    >
      {channel.name}
    </p>
    <p
      class="mt-1 truncate text-[10px] font-medium uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-45"
    >
      {channel.handle ?? channel.id}
    </p>
  </div>
  {#if !loading}
    {#if expanded !== undefined}
      <span
        class={`flex h-7 w-7 shrink-0 items-center justify-center text-[var(--soft-foreground)] transition-all duration-200 ${expanded ? "opacity-50" : "opacity-20"}`}
        aria-hidden="true"
      >
        <ChevronIcon
          direction={expanded ? "down" : "right"}
          size={9}
          strokeWidth={2.5}
        />
      </span>
    {/if}
  {/if}
</button>
