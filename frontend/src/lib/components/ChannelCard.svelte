<script lang="ts">
	import defaultChannelIcon from "$lib/assets/channel-default.svg";
	import type { Channel } from "$lib/types";

	export let channel: Channel;
	export let active = false;
	export let draggableEnabled = false;
	export let dragging = false;
	export let dragOver = false;
	export let onSelect: () => void = () => {};
	export let onDragStart: (event: DragEvent) => void = () => {};
	export let onDragOver: (event: DragEvent) => void = () => {};
	export let onDrop: (event: DragEvent) => void = () => {};
	export let onDragEnd: (event: DragEvent) => void = () => {};

	const normalizeThumbnail = (
		thumbnailUrl?: string | null,
	): string | null => {
		const trimmed = thumbnailUrl?.trim();
		return trimmed ? trimmed : null;
	};

	let thumbnailUrl: string | null = null;
	let avatarUrl = defaultChannelIcon;
	let avatarLoadFailed = false;

	$: thumbnailUrl = normalizeThumbnail(channel.thumbnail_url);
	$: {
		channel.id;
		thumbnailUrl;
		avatarLoadFailed = false;
	}
	$: avatarUrl =
		!avatarLoadFailed && thumbnailUrl ? thumbnailUrl : defaultChannelIcon;

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
	class={`group flex w-full min-w-0 items-center gap-3 rounded-2xl px-3 py-2.5 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] ${
		active
			? "bg-[var(--surface)] shadow-[0_2px_12px_rgba(0,0,0,0.03)] border border-[var(--border)]"
			: "border border-transparent hover:bg-white/40 hover:border-[var(--border)]"
	} ${dragging ? "opacity-50" : ""} ${dragOver ? "ring-2 ring-[var(--accent)] ring-offset-2 ring-offset-[var(--background)]" : ""}`}
	onclick={onSelect}
>
	<div
		class="h-12 w-12 overflow-hidden rounded-full border border-[var(--border)] bg-[var(--muted)]"
	>
		<img
			src={avatarUrl}
			alt={channel.name}
			width="48"
			height="48"
			loading="lazy"
			referrerpolicy="no-referrer"
			class="h-full w-full object-cover"
			onerror={handleAvatarError}
		/>
	</div>
	<div class="min-w-0 flex-1">
		<p class="truncate text-base font-semibold">{channel.name}</p>
		<p class="truncate text-xs text-[var(--soft-foreground)]">
			{channel.handle ?? channel.id}
		</p>
	</div>
	<div
		class="ml-1 text-[var(--soft-foreground)]/80"
		aria-hidden="true"
		title="Drag to reorder"
	>
		<svg viewBox="0 0 16 16" class="h-4 w-4">
			<circle cx="5" cy="4" r="1.1" fill="currentColor" />
			<circle cx="11" cy="4" r="1.1" fill="currentColor" />
			<circle cx="5" cy="8" r="1.1" fill="currentColor" />
			<circle cx="11" cy="8" r="1.1" fill="currentColor" />
			<circle cx="5" cy="12" r="1.1" fill="currentColor" />
			<circle cx="11" cy="12" r="1.1" fill="currentColor" />
		</svg>
	</div>
</button>
