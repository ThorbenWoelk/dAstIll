<script lang="ts">
	import { onMount } from "svelte";
	import { listChannels, listVideos, refreshChannel } from "$lib/api";
	import ChannelCard from "$lib/components/ChannelCard.svelte";
	import type { Channel, ContentStatus, Video } from "$lib/types";

	const secondaryButtonClass =
		"inline-flex items-center justify-center rounded-full border border-[var(--border)] px-5 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-[var(--foreground)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]";

	const quietButtonClass =
		"text-xs font-semibold uppercase tracking-[0.2em] text-[var(--accent)] transition-colors hover:text-[var(--accent-strong)] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]";

	const activeNavClass =
		"rounded-full bg-[var(--accent)] px-4 py-2 text-xs font-semibold uppercase tracking-[0.2em] text-white";
	const inactiveNavClass =
		"rounded-full px-4 py-2 text-xs font-semibold uppercase tracking-[0.2em] text-[var(--foreground)] transition-colors hover:text-[var(--accent)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]";

	const dateFormatter = new Intl.DateTimeFormat(undefined, {
		month: "short",
		day: "numeric",
		year: "numeric",
	});
	const syncTimeFormatter = new Intl.DateTimeFormat(undefined, {
		hour: "numeric",
		minute: "2-digit",
		second: "2-digit",
	});
	let channels = $state<Channel[]>([]);
	let videos = $state<Video[]>([]);
	let selectedChannelId = $state<string | null>(null);
	let loadingChannels = $state(false);
	let loadingVideos = $state(false);
	let errorMessage = $state<string | null>(null);

	let offset = $state(0);
	const limit = 20;
	let hasMore = $state(true);
	let lastSyncedAt = $state<Date | null>(null);
	let queueDeltaSinceLastSync = $state<number | null>(null);
	let previousQueuedTotal = $state<number | null>(null);

	const selectedChannel = $derived(
		channels.find((channel) => channel.id === selectedChannelId) ?? null,
	);

	const queuedVideos = $derived(
		videos.filter(
			(video) =>
				video.transcript_status !== "ready" ||
				video.summary_status !== "ready",
		),
	);

	function getQueueState(video: Video): Exclude<ContentStatus, "ready"> {
		if (
			video.transcript_status === "failed" ||
			video.summary_status === "failed"
		) {
			return "failed";
		}

		if (
			video.transcript_status === "loading" ||
			video.summary_status === "loading"
		) {
			return "loading";
		}

		return "pending";
	}

	const queueStats = $derived({
		total: queuedVideos.length,
		loading: queuedVideos.filter(
			(video) => getQueueState(video) === "loading",
		).length,
		pending: queuedVideos.filter(
			(video) => getQueueState(video) === "pending",
		).length,
		failed: queuedVideos.filter(
			(video) => getQueueState(video) === "failed",
		).length,
	});

	function statusLabel(status: ContentStatus) {
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
	}

	function statusHighlightClass(status: ContentStatus) {
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
	}

	function formatDate(value: string) {
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return "Date unavailable";
		return dateFormatter.format(date);
	}

	function setSyncSnapshot(snapshot: Video[]) {
		const queuedCount = snapshot.filter(
			(video) =>
				video.transcript_status !== "ready" ||
				video.summary_status !== "ready",
		).length;

		if (previousQueuedTotal === null) {
			queueDeltaSinceLastSync = null;
		} else {
			queueDeltaSinceLastSync = queuedCount - previousQueuedTotal;
		}

		previousQueuedTotal = queuedCount;
		lastSyncedAt = new Date();
	}

	onMount(() => {
		void loadChannels();
	});

	async function loadChannels() {
		loadingChannels = true;
		errorMessage = null;
		let initialChannelId: string | null = null;

		try {
			channels = await listChannels();
			if (!selectedChannelId && channels.length > 0) {
				initialChannelId = channels[0].id;
			}
		} catch (error) {
			errorMessage = (error as Error).message;
		} finally {
			loadingChannels = false;
		}

		if (initialChannelId && !selectedChannelId) {
			await selectChannel(initialChannelId);
		}
	}

	async function selectChannel(channelId: string) {
		selectedChannelId = channelId;
		videos = [];
		offset = 0;
		hasMore = true;
		lastSyncedAt = null;
		queueDeltaSinceLastSync = null;
		previousQueuedTotal = null;
		await refreshAndLoadVideos(channelId);
	}

	let refreshingChannel = $state(false);

	async function refreshAndLoadVideos(channelId: string) {
		// Instantly load existing videos
		await loadVideos(true);

		// Lazy load/refresh the channel in the background
		refreshingChannel = true;
		try {
			await refreshChannel(channelId);
			// After refresh, silently reload the queue
			if (selectedChannelId === channelId) {
				await loadVideos(true, true);
			}
		} catch (error) {
			if (!errorMessage) {
				errorMessage = (error as Error).message;
			}
		} finally {
			refreshingChannel = false;
		}
	}

	async function loadVideos(reset = false, silent = false) {
		if (!selectedChannelId) return;
		if (loadingVideos && !silent) return;

		if (!silent) loadingVideos = true;
		if (!silent) errorMessage = null;

		try {
			const list = await listVideos(
				selectedChannelId,
				limit,
				reset ? 0 : offset,
				"all",
			);
			videos = reset ? list : [...videos, ...list];
			offset = (reset ? 0 : offset) + list.length;
			hasMore = list.length === limit;
			if (reset) {
				setSyncSnapshot(list);
			}
		} catch (error) {
			if (!silent || !errorMessage)
				errorMessage = (error as Error).message;
		} finally {
			if (!silent) loadingVideos = false;
		}
	}
</script>

<div class="page-shell min-h-screen px-4 py-6 sm:px-8">
	<a
		href="#main-content"
		class="skip-link absolute left-4 top-4 z-50 rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-white"
	>
		Skip to Main Content
	</a>

	<header class="mx-auto flex w-full max-w-[1440px] flex-col gap-5">
		<div
			class="flex flex-col justify-between gap-4 md:flex-row md:items-end px-2"
		>
			<div class="max-w-3xl space-y-2">
				<p
					class="text-xs uppercase tracking-[0.35em] text-[var(--accent)] font-semibold"
				>
					dAstIll
				</p>
				<h1
					class="text-balance text-3xl font-bold tracking-tight sm:text-4xl text-[var(--foreground)]"
				>
					Inspect Download Queue
				</h1>
				<p
					class="max-w-2xl font-serif text-[17px] text-[var(--soft-foreground)]"
				>
					Inspect transcript and summary processing states per
					channel.
				</p>
			</div>
		</div>

		<nav
			class="flex flex-wrap items-center gap-2 rounded-3xl border border-[var(--border)] bg-[var(--surface)] p-1.5 px-2"
			aria-label="Workspace sections"
		>
			<a
				href="/"
				class="rounded-full px-4 py-2 text-xs font-semibold uppercase tracking-[0.15em] text-[var(--soft-foreground)] transition-colors hover:bg-[var(--muted)]/30 hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
				>Workspace</a
			>
			<a
				href="/download-queue"
				class="rounded-full bg-[var(--muted)]/50 px-4 py-2 text-xs font-semibold uppercase tracking-[0.15em] text-[var(--foreground)] transition-colors hover:text-[var(--accent)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
				>Download Queue</a
			>
		</nav>
	</header>

	<main
		id="main-content"
		class="mx-auto mt-6 grid w-full max-w-[1440px] gap-8 lg:grid-cols-[280px_minmax(0,1fr)] xl:grid-cols-[280px_minmax(0,1fr)]"
	>
		<aside
			class="flex h-fit flex-col gap-4 rounded-3xl bg-[var(--surface)] border border-[var(--border)] p-4 lg:sticky lg:top-6"
		>
			<div class="flex items-center justify-between gap-2 px-1">
				<h2 class="text-lg font-semibold tracking-tight">Channels</h2>
			</div>
			<p
				class="px-1 text-xs uppercase tracking-[0.15em] text-[var(--soft-foreground)]"
			>
				Manage channels on the workspace page.
			</p>

			<div
				class="flex max-h-[70vh] flex-col gap-2 overflow-y-auto pr-1"
				aria-busy={loadingChannels}
			>
				{#if loadingChannels}
					<div class="space-y-3" role="status" aria-live="polite">
						{#each Array.from({ length: 3 }) as _, index (index)}
							<div
								class="flex animate-pulse items-center gap-3 rounded-2xl px-3 py-2.5"
							>
								<div
									class="h-12 w-12 rounded-full bg-[var(--muted)]"
								></div>
								<div class="min-w-0 flex-1 space-y-2">
									<div
										class="h-3 w-3/4 rounded-full bg-[var(--muted)]"
									></div>
									<div
										class="h-2.5 w-1/2 rounded-full bg-[var(--muted)]/80"
									></div>
								</div>
							</div>
						{/each}
					</div>
				{:else if channels.length === 0}
					<p class="px-1 text-sm text-[var(--soft-foreground)]">
						No channels yet.
					</p>
				{:else}
					{#each channels as channel}
						<ChannelCard
							{channel}
							active={selectedChannelId === channel.id}
							onSelect={() => selectChannel(channel.id)}
						/>
					{/each}
				{/if}
			</div>
		</aside>

		<section
			class="flex min-w-0 flex-col gap-4 rounded-3xl bg-[var(--surface)] border border-[var(--border)] p-6 md:p-10"
		>
			<div
				class="flex flex-wrap items-center justify-between gap-4 border-b border-[var(--border)] pb-4"
			>
				<div>
					<h2 class="text-lg font-semibold tracking-tight">
						Queue Items
					</h2>
					<p
						class="truncate text-xs uppercase tracking-[0.15em] text-[var(--accent)] font-medium mt-0.5"
					>
						{selectedChannel
							? selectedChannel.name
							: "No Channel Selected"}
					</p>
					<p class="mt-1 text-xs text-[var(--soft-foreground)]">
						{#if lastSyncedAt}
							Last sync {syncTimeFormatter.format(lastSyncedAt)}
						{:else}
							Waiting for initial sync
						{/if}
					</p>
				</div>
				<div
					class="flex flex-wrap items-center gap-2 text-xs uppercase tracking-[0.12em] font-medium"
				>
					<span
						class="rounded-full border border-slate-200 bg-slate-50 px-3 py-1 text-slate-700"
					>
						Pending {queueStats.pending}
					</span>
					<span
						class="rounded-full border border-amber-200 bg-amber-50 px-3 py-1 text-amber-700"
					>
						Loading {queueStats.loading}
					</span>
					<span
						class="rounded-full border border-rose-200 bg-rose-50 px-3 py-1 text-rose-700"
					>
						Failed {queueStats.failed}
					</span>
					{#if queueDeltaSinceLastSync !== null}
						{#if queueDeltaSinceLastSync < 0}
							<span
								class="rounded-full border border-emerald-200 bg-emerald-50 px-3 py-1 text-emerald-700"
							>
								Queue down {Math.abs(queueDeltaSinceLastSync)}
							</span>
						{:else if queueDeltaSinceLastSync > 0}
							<span
								class="rounded-full border border-amber-200 bg-amber-50 px-3 py-1 text-amber-700"
							>
								Queue up {queueDeltaSinceLastSync}
							</span>
						{:else}
							<span
								class="rounded-full border border-slate-200 bg-slate-50 px-3 py-1 text-slate-700"
							>
								No queue change
							</span>
						{/if}
					{/if}
					<button
						type="button"
						class={quietButtonClass}
						onclick={() => loadVideos(true)}
						disabled={loadingVideos}
					>
						Sync now
					</button>
				</div>
			</div>

			{#if !selectedChannelId}
				<p class="text-sm text-[var(--soft-foreground)]">
					Select a channel to inspect its queue.
				</p>
			{:else if loadingVideos && videos.length === 0}
				<div class="space-y-3 mt-4" role="status" aria-live="polite">
					{#each Array.from({ length: 4 }) as _, index (index)}
						<div
							class="animate-pulse rounded-2xl border border-[var(--border)] bg-white/70 p-4"
						>
							<div
								class="h-3.5 w-11/12 rounded-full bg-[var(--muted)]"
							></div>
							<div
								class="mt-2 h-2.5 w-1/3 rounded-full bg-[var(--muted)]/80"
							></div>
						</div>
					{/each}
				</div>
			{:else if queueStats.total === 0}
				<p class="text-sm text-[var(--soft-foreground)] mt-4">
					Queue is empty. Everything is ready.
				</p>
			{:else}
				<ul class="flex flex-col gap-4 mt-4">
					{#each queuedVideos as video}
						<li
							class="rounded-2xl border border-[var(--border)] bg-white/40 p-4 transition-all hover:bg-white/70"
						>
							<div
								class="flex flex-wrap items-start justify-between gap-3"
							>
								<div class="min-w-0 flex-1">
									<p
										class="line-clamp-2 text-sm font-semibold text-[var(--foreground)] leading-relaxed"
									>
										{video.title}
									</p>
									<p
										class="mt-1 text-xs text-[var(--soft-foreground)]"
									>
										{formatDate(video.published_at)}
									</p>
									<div
										class="mt-3 text-[10.5px] font-medium tracking-wide"
									>
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
													><path
														d="M21 12a9 9 0 1 1-6.219-8.56"
													/></svg
												>
												Distilling knowledge...
											</span>
										{:else if video.transcript_status === "ready" && video.summary_status === "ready"}{:else if video.transcript_status === "failed" || video.summary_status === "failed"}
											<span
												class="text-rose-600/80 flex items-center gap-1.5"
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
													><circle
														cx="12"
														cy="12"
														r="10"
													/><line
														x1="12"
														y1="8"
														x2="12"
														y2="12"
													/><line
														x1="12"
														y1="16"
														x2="12.01"
														y2="16"
													/></svg
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
													><circle
														cx="12"
														cy="12"
														r="10"
													/><polyline
														points="12 6 12 12 16 14"
													/></svg
												>
												Queued for processing
											</span>
										{/if}
									</div>
								</div>
							</div>
						</li>
					{/each}
				</ul>
			{/if}

			{#if hasMore && selectedChannelId}
				<div class="flex justify-center mt-4">
					<button
						type="button"
						class={secondaryButtonClass}
						onclick={() => loadVideos(false)}
						disabled={loadingVideos}
					>
						{loadingVideos ? "Loading…" : "Load More"}
					</button>
				</div>
			{/if}
		</section>
	</main>

	{#if errorMessage}
		<div
			class="fixed bottom-6 left-1/2 z-50 w-[min(92vw,460px)] -translate-x-1/2 rounded-card border border-[var(--border)] bg-white/95 p-4 shadow-soft"
			role="status"
			aria-live="polite"
		>
			<p class="text-sm text-[var(--accent)]">{errorMessage}</p>
		</div>
	{/if}
</div>
