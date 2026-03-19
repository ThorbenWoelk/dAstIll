<script lang="ts">
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
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
    onDelete = () => {},
    showDelete = false,
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
    onDelete?: (event: Event) => void;
    showDelete?: boolean;
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
  class={`group relative flex w-full min-w-0 items-center gap-2.5 rounded-[var(--radius-sm)] px-2 py-2 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${trailingSpaceClass} ${
    active ? "bg-[var(--accent-wash)]" : "hover:bg-[var(--accent-wash)]"
  } ${dragging || loading ? "opacity-40" : ""} ${dragOver ? "ring-2 ring-[var(--accent)]/30" : ""} ${loading ? "animate-pulse" : ""} ${draggableEnabled ? (dragging ? "cursor-grabbing" : "cursor-grab") : ""}`}
  onclick={onSelect}
  disabled={loading}
>
  {#if active}
    <div
      class="absolute left-0 top-1/2 h-5 w-0.5 -translate-y-1/2 rounded-r-full bg-[var(--accent)]"
    ></div>
  {/if}
  {#if draggableEnabled && !showDelete}
    <div
      class="hidden shrink-0 items-center justify-center lg:flex"
      aria-hidden="true"
    >
      <div
        class={`inline-flex h-8 w-6 items-center justify-center rounded-full border transition-all ${dragging ? "border-[var(--accent)]/25 bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "border-transparent text-[var(--soft-foreground)] opacity-35 group-hover:opacity-70"}`}
      >
        <svg
          width="10"
          height="16"
          viewBox="0 0 10 16"
          fill="none"
          stroke="currentColor"
          stroke-width="1.8"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <path d="M2 3h.01"></path>
          <path d="M2 8h.01"></path>
          <path d="M2 13h.01"></path>
          <path d="M8 3h.01"></path>
          <path d="M8 8h.01"></path>
          <path d="M8 13h.01"></path>
        </svg>
      </div>
    </div>
  {/if}
  <div class="h-8 w-8 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]">
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
      class="mt-0.5 truncate text-[11px] font-medium text-[var(--soft-foreground)] opacity-40"
    >
      {channel.handle ?? channel.id}
    </p>
  </div>
  {#if !loading}
    <div
      role="button"
      tabindex="0"
      class={`absolute right-1 top-1/2 -translate-y-1/2 flex h-8 w-8 items-center justify-center transition-all duration-200 ${showDelete ? "opacity-100 translate-x-0" : "opacity-0 lg:group-hover:opacity-30 translate-x-2 pointer-events-none lg:pointer-events-auto max-lg:hidden"} hover:!opacity-100 text-[var(--soft-foreground)] hover:text-[var(--danger)]`}
      onclick={(e) => {
        e.stopPropagation();
        onDelete(e);
      }}
      onkeydown={(e) => {
        if (e.key === "Enter" || e.key === " ") {
          e.stopPropagation();
          e.preventDefault();
          onDelete(e);
        }
      }}
      aria-label="Delete channel"
    >
      <svg
        viewBox="0 0 24 24"
        fill="none"
        class="h-3.5 w-3.5"
        stroke="currentColor"
        stroke-width="2.5"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M3 6h18"></path>
        <path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path>
        <path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path>
      </svg>
    </div>
  {/if}
</button>
