<script lang="ts">
	import type { Video } from "$lib/types";

	export let video: Video;
	export let active = false;
	export let onSelect: () => void = () => {};
	const dateFormatter = new Intl.DateTimeFormat(undefined, {
		month: "short",
		day: "numeric",
		year: "numeric",
	});

	const formatDate = (value: string) => {
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) {
			return "Date Unavailable";
		}
		return dateFormatter.format(date);
	};
</script>

<button
	type="button"
	class={`group flex h-full w-full min-w-0 flex-col gap-3.5 rounded-[var(--radius-md)] p-3 text-left transition-all duration-300 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] ${
		active
			? "bg-[var(--surface)] shadow-[0_12px_32px_-8px_rgba(0,0,0,0.06)] border border-[var(--border)]"
			: "border border-transparent hover:bg-[var(--surface)]/60 hover:border-[var(--border-soft)]"
	}`}
	onclick={onSelect}
>
	<div
		class="aspect-video w-full overflow-hidden rounded-[var(--radius-md)] bg-[var(--muted)] relative"
	>
		{#if video.thumbnail_url}
			<img
				src={video.thumbnail_url}
				alt={video.title}
				width="480"
				height="270"
				loading="lazy"
				class="h-full w-full object-cover transition duration-500 group-hover:scale-[1.03]"
			/>
			<div class="absolute inset-0 bg-black/5 opacity-0 group-hover:opacity-100 transition-opacity duration-300"></div>
		{:else}
			<div
				class="flex h-full w-full items-center justify-center text-[10px] font-bold uppercase tracking-[0.3em] text-[var(--soft-foreground)] opacity-50"
			>
				No Preview
			</div>
		{/if}
	</div>
	<div class="flex min-w-0 flex-1 flex-col gap-1.5 px-0.5">
		<p
			class="line-clamp-2 min-h-[2.8rem] break-words text-[14.5px] font-bold leading-[1.35] tracking-tight text-[var(--foreground)] transition-colors duration-300 group-hover:text-[var(--accent-strong)]"
		>
			{video.title}
		</p>
		<div class="flex items-center justify-between mt-auto">
			<p class="text-[11.5px] font-medium tracking-wide text-[var(--soft-foreground)] opacity-60">
				{formatDate(video.published_at)}
			</p>
			<div class="text-[10px] font-bold tracking-[0.05em]">
				{#if video.transcript_status === "loading" || video.summary_status === "loading"}
					<span
						class="text-[var(--accent)] flex items-center gap-1.5"
					>
						<span class="relative flex h-1.5 w-1.5">
							<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-[var(--accent)] opacity-75"></span>
							<span class="relative inline-flex rounded-full h-1.5 w-1.5 bg-[var(--accent)]"></span>
						</span>
						DISTILLING
					</span>
				{:else if video.transcript_status === "failed" || video.summary_status === "failed"}
					<span class="text-rose-600 flex items-center gap-1.5 opacity-80">
						<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
						FAILED
					</span>
				{:else if video.transcript_status === "ready" && video.summary_status === "ready"}
					<span class="text-[var(--soft-foreground)] opacity-40 group-hover:opacity-100 transition-opacity duration-300 flex items-center gap-1.5">
						<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
						READY
					</span>
				{:else}
					<span
						class="text-[var(--soft-foreground)] flex items-center gap-1.5 opacity-60"
					>
						<svg width="10" height="10" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
						QUEUED
					</span>
				{/if}
			</div>
		</div>
	</div>
</button>
