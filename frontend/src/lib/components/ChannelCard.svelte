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
    onToggleExpanded = undefined,
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
    onToggleExpanded?: (() => void) | undefined;
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

<div
  role="group"
  draggable={draggableEnabled}
  ondragstart={onDragStart}
  ondragover={onDragOver}
  ondrop={onDrop}
  ondragend={onDragEnd}
  class={`group relative flex w-full min-w-0 items-center gap-3 rounded-md px-3 py-2 text-left transition-colors duration-150 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${trailingSpaceClass} ${
    active
      ? "bg-[var(--surface-strong)] text-[var(--foreground)]"
      : "hover:bg-[var(--surface)] text-[var(--soft-foreground)] hover:text-[var(--foreground)]"
  } ${dragging || loading ? "opacity-40" : ""} ${dragOver ? "shadow-[0_0_0_1px_var(--border-soft)]" : ""} ${loading ? "animate-pulse" : ""} ${draggableEnabled ? (dragging ? "cursor-grabbing" : "cursor-grab") : ""}`}
>
  <button
    type="button"
    class="flex min-w-0 flex-1 items-center gap-3 text-left focus-visible:outline-none"
    onclick={onSelect}
    disabled={loading}
  >
    <div
      class="h-8 w-8 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
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
        class={`truncate text-sm leading-tight tracking-tight ${active ? "font-semibold text-[var(--foreground)]" : "font-medium"}`}
      >
        {channel.name}
      </p>
      <p
        class="mt-0.5 truncate text-[10px] font-medium uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-45"
      >
        {channel.handle ?? channel.id}
      </p>
    </div>
  </button>
  {#if !loading}
    {#if expanded !== undefined}
      <button
        type="button"
        class="flex h-6 w-6 shrink-0 items-center justify-center rounded-md text-[var(--soft-foreground)] opacity-100 transition-colors duration-150 hover:bg-[var(--surface)] hover:text-[var(--foreground)]"
        onclick={() => onToggleExpanded?.()}
        aria-label={expanded ? "Collapse channel" : "Expand channel"}
      >
        <ChevronIcon
          direction={expanded ? "down" : "right"}
          size={9}
          strokeWidth={2}
        />
      </button>
    {/if}
  {/if}
</div>
