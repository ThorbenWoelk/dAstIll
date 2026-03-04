<script lang="ts">
	import defaultChannelIcon from "$lib/assets/channel-default.svg";
	import type { Channel } from "$lib/types";

	export let channel: Channel;
	export let active = false;
	export let draggableEnabled = false;
	export let dragging = false;
	export let dragOver = false;
	export let loading = false;
	export let onSelect: () => void = () => {};
	export let onDragStart: (event: DragEvent) => void = () => {};
	export let onDragOver: (event: DragEvent) => void = () => {};
	export let onDrop: (event: DragEvent) => void = () => {};
	export let onDragEnd: (event: DragEvent) => void = () => {};
	export let onDelete: (event: Event) => void = () => {};
	export let showDelete = false;

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
	class={`group relative flex w-full min-w-0 items-center gap-3 rounded-[var(--radius-md)] px-3 py-2.5 text-left transition-all duration-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] ${
		active
			? "bg-[var(--surface)] border border-[var(--border)] shadow-[0_4px_12px_-2px_rgba(0,0,0,0.04)]"
			: "border border-transparent hover:bg-[var(--surface)]/60 hover:border-[var(--border-soft)]"
	} ${dragging || loading ? "opacity-40" : ""} ${dragOver ? "ring-2 ring-[var(--accent)] ring-offset-2 ring-offset-[var(--background)]" : ""} ${loading ? "animate-pulse" : ""}`}
	onclick={onSelect}
	disabled={loading}
>
	{#if active}
		<div
			class="absolute left-0 top-1/2 h-6 w-1 -translate-y-1/2 rounded-r-full bg-[var(--accent)] transition-all duration-300"
		></div>
	{/if}
	<div
		class="h-9 w-9 shrink-0 overflow-hidden rounded-full border border-[var(--border-soft)] bg-[var(--muted)] transition-transform duration-300 {loading ? '' : 'group-hover:scale-105'}"
	>
		<img
			src={avatarUrl}
			alt={channel.name}
			width="36"
			height="36"
			loading="lazy"
			referrerpolicy="no-referrer"
			class="h-full w-full object-cover"
			onerror={handleAvatarError}
		/>
	</div>
	<div class="min-w-0 flex-1">
		<p
			class="truncate text-[14px] font-bold leading-tight tracking-tight text-[var(--foreground)] transition-colors"
		>
			{channel.name}
		</p>
		<p
			class="mt-0.5 truncate text-[11px] font-medium tracking-wide text-[var(--soft-foreground)] opacity-60 transition-colors"
		>
			{channel.handle ?? channel.id}
		</p>
	</div>
	{#if !loading}
		<div
			role="button"
			tabindex="0"
			class={`absolute right-1 top-1/2 -translate-y-1/2 flex h-10 w-10 items-center justify-center transition-all duration-300 ${showDelete ? "opacity-100 translate-x-0" : "opacity-0 lg:group-hover:opacity-40 translate-x-2 pointer-events-none lg:pointer-events-auto max-lg:hidden"} hover:!opacity-100 text-[var(--soft-foreground)] hover:text-red-500`}
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
				class="h-4 w-4"
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
