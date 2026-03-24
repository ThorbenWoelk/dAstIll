<script lang="ts">
  import type { Snippet } from "svelte";

  let {
    active = false,
    loading = false,
    draggableEnabled = false,
    dragging = false,
    dragOver = false,
    trailingSpaceClass = "",
    onSelect = () => {},
    onDragStart = () => {},
    onDragOver = () => {},
    onDrop = () => {},
    onDragEnd = () => {},
    children,
  }: {
    active?: boolean;
    loading?: boolean;
    draggableEnabled?: boolean;
    dragging?: boolean;
    dragOver?: boolean;
    trailingSpaceClass?: string;
    onSelect?: () => void;
    onDragStart?: (event: DragEvent) => void;
    onDragOver?: (event: DragEvent) => void;
    onDrop?: (event: DragEvent) => void;
    onDragEnd?: (event: DragEvent) => void;
    children?: Snippet;
  } = $props();
</script>

<button
  type="button"
  draggable={draggableEnabled}
  ondragstart={onDragStart}
  ondragover={onDragOver}
  ondrop={onDrop}
  ondragend={onDragEnd}
  class={`group relative flex w-full min-w-0 items-center gap-2 rounded-[var(--radius-sm)] p-2 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 ${trailingSpaceClass} ${
    active
      ? "bg-[var(--accent-wash)] shadow-sm"
      : "hover:bg-[var(--accent-wash)]"
  } ${dragging || loading ? "opacity-40" : ""} ${dragOver ? "ring-2 ring-[var(--accent)]/30" : ""} ${loading ? "animate-pulse" : ""} ${draggableEnabled ? (dragging ? "cursor-grabbing" : "cursor-grab") : ""}`}
  onclick={onSelect}
  disabled={loading}
>
  {@render children?.()}
</button>
