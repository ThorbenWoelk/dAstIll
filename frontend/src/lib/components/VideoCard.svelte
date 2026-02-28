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

	const statusLabel = (status: Video["transcript_status"]) => {
		switch (status) {
			case "ready":
				return "ready";
			case "loading":
				return "generating";
			case "failed":
				return "failed";
			default:
				return "queued";
		}
	};

	const statusHighlightClass = (status: Video["transcript_status"]) => {
		switch (status) {
			case "ready":
				return "text-emerald-700 bg-emerald-50/50";
			case "loading":
				return "text-[var(--accent)] bg-[var(--accent)]/5 animate-pulse";
			case "failed":
				return "text-rose-700 bg-rose-50/50";
			default:
				return "text-stone-500 bg-stone-100/50";
		}
	};

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
	class={`group flex h-full w-full min-w-0 flex-col gap-3 rounded-2xl p-3 text-left transition-all duration-200 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--background)] ${
		active
			? "bg-[var(--surface)] shadow-[0_4px_24px_rgba(0,0,0,0.02)] border border-[var(--border)]"
			: "border border-transparent hover:bg-white/40 hover:border-[var(--border)]"
	}`}
	onclick={onSelect}
>
	<div
		class="aspect-video w-full overflow-hidden rounded-xl bg-[var(--muted)]"
	>
		{#if video.thumbnail_url}
			<img
				src={video.thumbnail_url}
				alt={video.title}
				width="480"
				height="270"
				loading="lazy"
				class="h-full w-full object-cover transition duration-300 group-hover:scale-[1.02]"
			/>
		{:else}
			<div
				class="flex h-full w-full items-center justify-center text-xs uppercase tracking-[0.3em] text-[var(--soft-foreground)]"
			>
				Video
			</div>
		{/if}
	</div>
	<div class="flex min-w-0 flex-1 flex-col gap-1.5 px-0.5">
		<p
			class="line-clamp-2 min-h-[2.5rem] break-words text-sm font-semibold leading-relaxed text-[var(--foreground)]"
		>
			{video.title}
		</p>
		<p class="text-xs text-[var(--soft-foreground)]">
			{formatDate(video.published_at)}
		</p>
		<div class="mt-2 text-[10.5px] font-medium tracking-wide">
			{#if video.transcript_status === "loading" || video.summary_status === "loading"}
				<span
					class="text-[var(--accent)] animate-pulse flex items-center gap-1.5"
				>
					<svg
						width="12"
						height="12"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						class="animate-spin"
						><path d="M21 12a9 9 0 1 1-6.219-8.56" /></svg
					>
					Distilling knowledge...
				</span>
			{:else if video.transcript_status === "ready" && video.summary_status === "ready"}{:else if video.transcript_status === "failed" || video.summary_status === "failed"}
				<span class="text-rose-600/80 flex items-center gap-1.5">
					<svg
						width="12"
						height="12"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						><circle cx="12" cy="12" r="10" /><line
							x1="12"
							y1="8"
							x2="12"
							y2="12"
						/><line x1="12" y1="16" x2="12.01" y2="16" /></svg
					>
					Distillation failed
				</span>
			{:else}
				<span
					class="text-[var(--soft-foreground)] flex items-center gap-1.5"
				>
					<svg
						width="12"
						height="12"
						viewBox="0 0 24 24"
						fill="none"
						stroke="currentColor"
						stroke-width="2.5"
						stroke-linecap="round"
						stroke-linejoin="round"
						><circle cx="12" cy="12" r="10" /><polyline
							points="12 6 12 12 16 14"
						/></svg
					>
					Queued for processing
				</span>
			{/if}
		</div>
	</div>
</button>
