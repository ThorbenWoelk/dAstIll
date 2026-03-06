<script lang="ts">
	import { goto } from "$app/navigation";
	import { onMount } from "svelte";
	import {
		getChannelSyncDepth,
		isAiAvailable,
		listChannels,
		listChannelsWhenAvailable,
		listVideos,
		refreshChannel,
		updateChannel,
	} from "$lib/api";
	import {
		applySavedChannelOrder,
		resolveInitialChannelSelection,
		WORKSPACE_STATE_KEY,
		type WorkspaceStateSnapshot,
	} from "$lib/channel-workspace";
	import defaultChannelIcon from "$lib/assets/channel-default.svg";
	import ChannelCard from "$lib/components/ChannelCard.svelte";
	import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
	import type { Channel, ContentStatus, Video } from "$lib/types";

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
	let channelOrder = $state<string[]>([]);
	let videos = $state<Video[]>([]);
	let selectedChannelId = $state<string | null>(null);
	let draggedChannelId = $state<string | null>(null);
	let dragOverChannelId = $state<string | null>(null);
	let loadingChannels = $state(false);
	let aiAvailable = $state<boolean | null>(null);
	let loadingVideos = $state(false);
	let waitingForBackend = $state(false);
	let errorMessage = $state<string | null>(null);
	let showDeleteConfirmation = $state(false);
	let channelIdToDelete = $state<string | null>(null);
	let workspaceStateHydrated = $state(false);

	let offset = $state(0);
	const limit = 20;
	let hasMore = $state(true);
	let lastSyncedAt = $state<Date | null>(null);
	let queueDeltaSinceLastSync = $state<number | null>(null);
	let previousQueuedTotal = $state<number | null>(null);
	let earliestSyncDateInput = $state("");
	let savingSyncDate = $state(false);
	let syncDepth = $state<{
		earliest_sync_date: string | null;
		earliest_sync_date_user_set: boolean;
		derived_earliest_ready_date: string | null;
	} | null>(null);

	const MAX_RETRIES = 3;
	const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;
	const channelLastRefreshedAt = new Map<string, number>();

	$effect(() => {
		const timer = setInterval(() => {
			void isAiAvailable()
				.then((status) => {
					aiAvailable = status.available;
				})
				.catch(() => {
					aiAvailable = false;
				});
		}, 30000);
		return () => clearInterval(timer);
	});

	let mobileTab = $state<"channels" | "details">("details");
	let manageChannels = $state(false);

	function reorderChannels(dragId: string, targetId: string) {
		if (dragId === targetId) return;
		const ids = channels.map((channel) => channel.id);
		const fromIndex = ids.indexOf(dragId);
		const toIndex = ids.indexOf(targetId);
		if (fromIndex < 0 || toIndex < 0) return;

		ids.splice(fromIndex, 1);
		ids.splice(toIndex, 0, dragId);
		const byId = new Map(channels.map((channel) => [channel.id, channel]));
		channels = ids
			.map((id) => byId.get(id))
			.filter((channel): channel is Channel => !!channel);
		channelOrder = ids;
	}

	function handleChannelDragStart(channelId: string, event: DragEvent) {
		draggedChannelId = channelId;
		dragOverChannelId = channelId;
		if (!event.dataTransfer) return;
		event.dataTransfer.effectAllowed = "move";
		event.dataTransfer.setData("text/plain", channelId);
	}

	function handleChannelDragOver(channelId: string, event: DragEvent) {
		event.preventDefault();
		if (dragOverChannelId !== channelId) {
			dragOverChannelId = channelId;
		}
	}

	function handleChannelDrop(channelId: string, event: DragEvent) {
		event.preventDefault();
		const fallbackId = event.dataTransfer?.getData("text/plain") || null;
		const sourceId = draggedChannelId || fallbackId;
		if (sourceId) {
			reorderChannels(sourceId, channelId);
		}
		draggedChannelId = null;
		dragOverChannelId = null;
	}

	function handleChannelDragEnd() {
		draggedChannelId = null;
		dragOverChannelId = null;
	}

	const selectedChannel = $derived(
		channels.find((channel) => channel.id === selectedChannelId) ?? null,
	);

	const effectiveEarliestSyncDate = $derived(
		selectedChannel?.earliest_sync_date_user_set
			? selectedChannel.earliest_sync_date
			: (syncDepth?.derived_earliest_ready_date ?? selectedChannel?.earliest_sync_date),
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

	type DistillationStatusKind = "processing" | "queued" | "failed";

	interface DistillationStatusCopy {
		kind: DistillationStatusKind;
		label: string;
		detail: string;
	}

	function getDistillationStatusCopy(video: Video): DistillationStatusCopy {
		const retries = video.retry_count ?? 0;
		const permanentlyFailed = retries >= MAX_RETRIES;

		if (video.transcript_status !== "ready") {
			if (video.transcript_status === "loading") {
				return {
					kind: "processing",
					label: "DISTILLING KNOWLEDGE - PROCESSING TRANSCRIPT",
					detail: "Transcript extraction is running now."
				};
			}

			if (video.transcript_status === "failed") {
				return {
					kind: "failed",
					label: permanentlyFailed
						? "DISTILLATION FAILED - TRANSCRIPT (PERMANENT)"
						: "DISTILLATION FAILED - TRANSCRIPT (RETRYING)",
					detail: permanentlyFailed
						? "Automatic retries are exhausted."
						: "Transcript extraction failed. Automatic retry is queued."
				};
			}

			return {
				kind: "queued",
				label: "DISTILLING KNOWLEDGE - QUEUED FOR TRANSCRIPT",
				detail: "Waiting in queue to start transcript extraction."
			};
		}

		if (video.summary_status === "loading") {
			return {
				kind: "processing",
				label: "DISTILLING KNOWLEDGE - PROCESSING SUMMARY",
				detail: "Summary generation is running now."
			};
		}

		if (video.summary_status === "failed") {
			return {
				kind: "failed",
				label: permanentlyFailed
					? "DISTILLATION FAILED - SUMMARY (PERMANENT)"
					: "DISTILLATION FAILED - SUMMARY (RETRYING)",
				detail: permanentlyFailed
					? "Automatic retries are exhausted."
					: "Summary generation failed. Automatic retry is queued."
			};
		}

		return {
			kind: "queued",
			label: "DISTILLING KNOWLEDGE - QUEUED FOR SUMMARY",
			detail: "Transcript is ready. Waiting in queue for summary generation."
		};
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
	const queuedVideosWithDistillationStatus = $derived(
		queuedVideos.map((video) => ({
			video,
			distillationStatus: getDistillationStatusCopy(video)
		})),
	);

	function formatDate(value: string) {
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return "Date unavailable";
		return dateFormatter.format(date);
	}

	function formatSyncDate(value: string | null | undefined) {
		if (!value) return "Not set";
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return "Not set";
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

	function restoreWorkspaceState() {
		if (typeof localStorage === "undefined") return;
		const raw = localStorage.getItem(WORKSPACE_STATE_KEY);
		if (!raw) return;

		try {
			const snapshot = JSON.parse(raw) as Partial<WorkspaceStateSnapshot>;
			if (
				typeof snapshot.selectedChannelId === "string" ||
				snapshot.selectedChannelId === null
			) {
				selectedChannelId = snapshot.selectedChannelId;
			}
			if (Array.isArray(snapshot.channelOrder)) {
				channelOrder = snapshot.channelOrder.filter(
					(id): id is string => typeof id === "string",
				);
			}
		} catch {
			localStorage.removeItem(WORKSPACE_STATE_KEY);
		}
	}

	function persistWorkspaceState() {
		if (!workspaceStateHydrated || typeof localStorage === "undefined")
			return;

		const raw = localStorage.getItem(WORKSPACE_STATE_KEY);
		let snapshot: Partial<WorkspaceStateSnapshot> = {};
		if (raw) {
			try {
				snapshot = JSON.parse(raw);
			} catch {
				// Ignore
			}
		}

		snapshot.selectedChannelId = selectedChannelId;
		snapshot.channelOrder = channelOrder;

		localStorage.setItem(WORKSPACE_STATE_KEY, JSON.stringify(snapshot));
	}

	async function openVideoTranscriptInWorkspace(video: Video) {
		if (typeof localStorage !== "undefined") {
			const raw = localStorage.getItem(WORKSPACE_STATE_KEY);
			let snapshot: Partial<WorkspaceStateSnapshot> = {};
			if (raw) {
				try {
					snapshot = JSON.parse(raw);
				} catch {
					// Ignore malformed workspace snapshot
				}
			}

			snapshot.selectedChannelId = video.channel_id;
			snapshot.selectedVideoId = video.id;
			snapshot.contentMode = "transcript";
			snapshot.videoTypeFilter = "all";
			snapshot.acknowledgedFilter = "all";

			localStorage.setItem(WORKSPACE_STATE_KEY, JSON.stringify(snapshot));
		}

		await goto("/");
	}

	$effect(() => {
		persistWorkspaceState();
	});

	$effect(() => {
		if (!selectedChannel) {
			earliestSyncDateInput = "";
			return;
		}
		const effective =
			selectedChannel.earliest_sync_date_user_set
				? selectedChannel.earliest_sync_date
				: (syncDepth?.derived_earliest_ready_date ?? selectedChannel.earliest_sync_date);
		if (effective) {
			earliestSyncDateInput = new Date(effective).toISOString().split("T")[0];
		} else {
			earliestSyncDateInput = "";
		}
	});

	$effect(() => {
		if (selectedChannelId) {
			void getChannelSyncDepth(selectedChannelId).then((d) => (syncDepth = d));
		} else {
			syncDepth = null;
		}
	});

	onMount(() => {
		restoreWorkspaceState();
		workspaceStateHydrated = true;
		waitingForBackend = true;
		void loadChannels(true);
	});

	async function loadChannels(retryUntilBackendReachable = false) {
		loadingChannels = true;
		errorMessage = null;
		let initialChannelId: string | null = null;

		try {
			// Load AI status in parallel
			const aiStatusPromise = isAiAvailable().catch(() => ({ available: false }));
			const fetchedChannelsPromise = retryUntilBackendReachable
				? listChannelsWhenAvailable()
				: listChannels();

			const [aiStatus, fetchedChannels] = await Promise.all([
				aiStatusPromise,
				fetchedChannelsPromise,
			]);

			aiAvailable = aiStatus.available;
			waitingForBackend = false;
			channels = applySavedChannelOrder(fetchedChannels, channelOrder);
			channelOrder = channels.map((c) => c.id);

			initialChannelId = resolveInitialChannelSelection(
				channels,
				selectedChannelId,
				null,
			);
		} catch (error) {
			waitingForBackend = false;
			errorMessage = (error as Error).message;
		} finally {
			loadingChannels = false;
		}

		if (initialChannelId && initialChannelId !== selectedChannelId) {
			await selectChannel(initialChannelId);
		} else if (selectedChannelId) {
			await refreshAndLoadVideos(selectedChannelId);
		}
	}

	async function selectChannel(channelId: string) {
		selectedChannelId = channelId;
		mobileTab = "details";
		videos = [];
		offset = 0;
		hasMore = true;
		lastSyncedAt = null;
		queueDeltaSinceLastSync = null;
		previousQueuedTotal = null;
		await refreshAndLoadVideos(channelId);
	}

	async function handleDeleteChannel(channelId: string) {
		channelIdToDelete = channelId;
		showDeleteConfirmation = true;
	}

	async function confirmDeleteChannel() {
		if (!channelIdToDelete) return;
		const channelId = channelIdToDelete;
		showDeleteConfirmation = false;
		channelIdToDelete = null;

		try {
			const { deleteChannel } = await import("$lib/api");
			await deleteChannel(channelId);
			channels = channels.filter((c) => c.id !== channelId);
			channelOrder = channelOrder.filter((id) => id !== channelId);
			if (selectedChannelId === channelId) {
				const nextChannel = channels.length > 0 ? channels[0] : null;
				if (nextChannel) {
					await selectChannel(nextChannel.id);
				} else {
					selectedChannelId = null;
					videos = [];
				}
			}
		} catch (error) {
			errorMessage = (error as Error).message;
		}
	}

	function cancelDeleteChannel() {
		showDeleteConfirmation = false;
		channelIdToDelete = null;
	}

	let refreshingChannel = $state(false);

	async function refreshAndLoadVideos(channelId: string) {
		// Instantly load existing videos
		await loadVideos(true);

		// Skip YouTube refresh if channel was refreshed recently
		const lastRefresh = channelLastRefreshedAt.get(channelId);
		if (lastRefresh && Date.now() - lastRefresh < CHANNEL_REFRESH_TTL_MS) {
			return;
		}

		// Lazy load/refresh the channel in the background
		refreshingChannel = true;
		try {
			await refreshChannel(channelId);
			channelLastRefreshedAt.set(channelId, Date.now());
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
				undefined,
				true,
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

	async function saveEarliestSyncDate() {
		if (!selectedChannelId || !earliestSyncDateInput || savingSyncDate) return;
		errorMessage = null;
		savingSyncDate = true;
		try {
			const updated = await updateChannel(selectedChannelId, {
				earliest_sync_date: new Date(earliestSyncDateInput).toISOString(),
				earliest_sync_date_user_set: true,
			});
			channels = channels.map((channel) =>
				channel.id === selectedChannelId ? updated : channel,
			);
			syncDepth = await getChannelSyncDepth(selectedChannelId);
		} catch (error) {
			errorMessage = (error as Error).message;
		} finally {
			savingSyncDate = false;
		}
	}
</script>

<div class="page-shell min-h-screen px-4 pb-12 pt-8 sm:px-8 max-lg:px-0">
	<a
		href="#main-content"
		class="skip-link absolute left-4 top-4 z-50 rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-white shadow-lg shadow-[var(--accent)]/20"
	>
		Skip to Main Content
	</a>

	<header class="mx-auto flex w-full max-w-[1440px] items-center justify-between gap-4 px-4 sm:px-2 fade-in border-b border-[var(--border-soft)] pb-3 mb-1">
		<div class="flex items-center gap-4">
			<h1 class="text-2xl sm:text-3xl font-bold tracking-tighter text-[var(--foreground)]">
				DASTILL
			</h1>
			<span class="hidden sm:block h-5 w-px bg-[var(--border-soft)]"></span>
			<p class="hidden sm:block text-[10px] font-bold uppercase tracking-[0.3em] text-[var(--accent)] opacity-60">
				Observatory
			</p>
			{#if aiAvailable !== null}
				<span class="hidden sm:block h-3 w-px bg-[var(--border-soft)]"></span>
				<div 
					class="hidden sm:block h-2 w-2 rounded-full {aiAvailable ? 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.4)]' : 'bg-[var(--accent)] shadow-[0_0_8px_rgba(212,100,81,0.4)]'}"
					data-tooltip={aiAvailable ? "AI Engine: Ready" : "AI Engine: Offline"}
				></div>
			{/if}
		</div>

		<nav
			class="flex items-center gap-1 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] p-1 shadow-sm"
			aria-label="Workspace sections"
		>
			<a
				href="/"
				class="rounded-[var(--radius-sm)] px-3 py-1.5 sm:px-5 sm:py-2 text-[10px] font-bold uppercase tracking-[0.15em] text-[var(--soft-foreground)] opacity-60 transition-all hover:opacity-100 hover:bg-[var(--muted)]/30"
			>
				Workspace
			</a>
			<a
				href="/download-queue"
				class="rounded-[var(--radius-sm)] bg-[var(--muted)]/60 px-3 py-1.5 sm:px-5 sm:py-2 text-[10px] font-bold uppercase tracking-[0.15em] text-[var(--foreground)] transition-all"
			>
				Details
			</a>
		</nav>
	</header>

	{#if waitingForBackend && channels.length === 0}
		<main
			id="main-content"
			class="mx-auto mt-12 grid w-full max-w-[1440px]"
		>
			<section
				class="flex min-h-[480px] flex-col items-center justify-center gap-8 rounded-[var(--radius-lg)] border border-[var(--border-soft)] bg-[var(--surface)] p-12 text-center fade-in shadow-sm"
				role="status"
				aria-live="polite"
			>
				<div class="relative flex h-12 w-12 items-center justify-center">
					<div class="absolute h-full w-full animate-ping rounded-full bg-[var(--accent)] opacity-10"></div>
					<div class="h-8 w-8 animate-spin rounded-full border-2 border-[var(--muted)] border-t-[var(--accent)]"></div>
				</div>
				<div class="space-y-3">
					<p class="text-[11px] font-bold uppercase tracking-[0.3em] text-[var(--accent)]">
						Establishing Link
					</p>
					<p class="max-w-xs text-[15px] font-medium text-[var(--soft-foreground)] opacity-70">
						Waiting for the observatory to connect with the distillation engine.
					</p>
				</div>
			</section>
		</main>
	{:else}
		<main
			id="main-content"
			class="mx-auto mt-10 grid w-full max-w-[1440px] gap-10 lg:grid-cols-[280px_minmax(0,1fr)] xl:grid-cols-[320px_minmax(0,1fr)] items-start max-lg:mt-0 max-lg:gap-0"
		>
		<aside
			class="flex h-fit flex-col gap-6 border-0 lg:rounded-[var(--radius-lg)] lg:bg-[var(--surface)] lg:border lg:border-[var(--border-soft)] lg:p-6 lg:sticky lg:top-8 fade-in stagger-1 lg:shadow-sm {mobileTab !== 'channels' ? 'hidden lg:flex' : 'h-[calc(100dvh-4rem)] p-4 gap-4'}"
		>
			<div class="flex items-center justify-between gap-2">
				<h2 class="text-xl font-bold tracking-tight">Channels</h2>
				<div class="flex items-center gap-1">
					<button
						type="button"
						class="inline-flex h-7 w-7 items-center justify-center rounded-[var(--radius-sm)] border border-transparent transition-colors hover:border-[var(--border-soft)] {manageChannels ? 'text-red-500' : 'text-[var(--soft-foreground)] opacity-50'}"
						data-tooltip={manageChannels ? "Exit manage mode" : "Manage channels"}
						onclick={() => {
							manageChannels = !manageChannels;
						}}
						aria-label={manageChannels ? "Exit manage mode" : "Manage channels"}
					>
						<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6h18"></path><path d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg>
					</button>
				</div>
			</div>
			<p
				class="px-1 text-[10px] font-bold uppercase tracking-[0.2em] text-[var(--soft-foreground)] opacity-40 leading-relaxed"
			>
				Select a channel to inspect its specific queue.
			</p>

			<div
				class="flex flex-1 min-h-0 flex-col gap-1.5 overflow-y-auto pr-1 custom-scrollbar lg:max-h-[60vh]"
				aria-busy={loadingChannels}
			>
				{#if loadingChannels}
					<div class="space-y-4" role="status" aria-live="polite">
						{#each Array.from({ length: 4 }) as _, index (index)}
							<div
								class="flex animate-pulse items-center gap-4 px-3 py-3"
							>
								<div
									class="h-10 w-10 shrink-0 rounded-full bg-[var(--muted)] opacity-60"
								></div>
								<div class="min-w-0 flex-1 space-y-2">
									<div
										class="h-3 w-3/4 rounded-full bg-[var(--muted)] opacity-60"
									></div>
									<div
										class="h-2 w-1/2 rounded-full bg-[var(--muted)] opacity-40"
									></div>
								</div>
							</div>
						{/each}
					</div>
				{:else if channels.length === 0}
					<p class="px-1 text-[14px] font-medium text-[var(--soft-foreground)] opacity-50 italic">
						No channels followed.
					</p>
				{:else}
					{#each channels as channel}
						<ChannelCard
							{channel}
							active={selectedChannelId === channel.id}
							showDelete={manageChannels}
							draggableEnabled
							loading={channel.id.startsWith("temp-")}
							dragging={draggedChannelId === channel.id}
							dragOver={dragOverChannelId === channel.id &&
								draggedChannelId !== channel.id}
							onSelect={() => selectChannel(channel.id)}
							onDragStart={(event) =>
								handleChannelDragStart(channel.id, event)}
							onDragOver={(event) =>
								handleChannelDragOver(channel.id, event)}
							onDrop={(event) => handleChannelDrop(channel.id, event)}
							onDragEnd={handleChannelDragEnd}
							onDelete={() => handleDeleteChannel(channel.id)}
						/>
					{/each}
				{/if}
			</div>
		</aside>

		<section
			class="flex min-w-0 flex-col gap-8 overflow-hidden border-0 lg:rounded-[var(--radius-lg)] lg:bg-[var(--surface)] lg:border lg:border-[var(--border-soft)] lg:p-8 lg:md:p-12 fade-in stagger-2 lg:shadow-sm {mobileTab !== 'details' ? 'hidden lg:flex' : 'h-[calc(100dvh-4rem)] p-4 pt-6'}"
		>
			<div
				class="flex flex-wrap items-center justify-between gap-8 border-b border-[var(--border-soft)]/50 pb-8 max-lg:gap-4 max-lg:pb-4"
			>
				<div class="space-y-2">
					<h2 class="text-2xl font-bold tracking-tight">
						Channel Details
					</h2>
					<div class="flex items-center gap-3">
						<button 
							onclick={() => mobileTab = 'channels'}
							class="lg:hidden inline-flex items-center rounded-full bg-[var(--muted)] border border-[var(--border)] pl-1.5 pr-3 py-1 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] transition-transform active:scale-95 shadow-sm"
						>
							<div class="mr-2 h-5 w-5 shrink-0 overflow-hidden rounded-full border border-[var(--border-soft)] bg-white">
								<img
									src={selectedChannel?.thumbnail_url || defaultChannelIcon}
									alt=""
									class="h-full w-full object-cover"
								/>
							</div>
							{selectedChannel ? selectedChannel.name : "None"}
							<svg class="ml-1.5 h-2.5 w-2.5 opacity-40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="4" stroke-linecap="round" stroke-linejoin="round"><path d="m6 9 6 6 6-6"/></svg>
						</button>
						<span class="max-lg:hidden inline-flex items-center rounded-full bg-[var(--muted)] border border-[var(--border)] pl-1.5 pr-4 py-1 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)]">
							<div class="mr-2 h-5 w-5 shrink-0 overflow-hidden rounded-full border border-[var(--border-soft)] bg-white">
								<img
									src={selectedChannel?.thumbnail_url || defaultChannelIcon}
									alt=""
									class="h-full w-full object-cover"
								/>
							</div>
							{selectedChannel ? selectedChannel.name : "None"}
						</span>
						<p class="text-[12px] font-medium text-[var(--soft-foreground)] opacity-60">
							{#if lastSyncedAt}
								Pulse captured at {syncTimeFormatter.format(lastSyncedAt)}
							{:else}
								Establishing initial sync…
							{/if}
						</p>
					</div>
				</div>
				<div
					class="flex flex-wrap items-center gap-3"
				>
					<div class="flex items-center gap-2 px-4 py-2 rounded-full bg-[var(--muted)]/30 border border-[var(--border-soft)] max-lg:px-3 max-lg:py-1">
						<div class="h-1.5 w-1.5 rounded-full bg-slate-400"></div>
						<span class="text-[10px] font-bold uppercase tracking-[0.1em] text-slate-600">{queueStats.pending}</span>
					</div>
					<div class="flex items-center gap-2 px-4 py-2 rounded-full bg-amber-50 border border-amber-200 max-lg:px-3 max-lg:py-1">
						<div class="h-1.5 w-1.5 rounded-full bg-amber-500 animate-pulse"></div>
						<span class="text-[10px] font-bold uppercase tracking-[0.1em] text-amber-700">{queueStats.loading}</span>
					</div>
					<div class="flex items-center gap-2 px-4 py-2 rounded-full bg-rose-50 border border-rose-200 max-lg:px-3 max-lg:py-1">
						<div class="h-1.5 w-1.5 rounded-full bg-rose-500"></div>
						<span class="text-[10px] font-bold uppercase tracking-[0.1em] text-rose-700">{queueStats.failed}</span>
					</div>
					
					</div>
				</div>

			{#if selectedChannel}
				<div class="grid gap-4 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--background)] p-5 sm:grid-cols-[minmax(0,1fr)_auto] sm:items-end max-lg:bg-transparent max-lg:border-0 max-lg:p-0">
					<div class="space-y-2">
						<p class="text-[10px] font-bold uppercase tracking-[0.2em] text-[var(--soft-foreground)] opacity-60">
							Oldest Sync Date
						</p>
						<p class="text-[14px] font-semibold text-[var(--foreground)]">
							{formatSyncDate(effectiveEarliestSyncDate)}
						</p>
						<p class="text-[11px] text-[var(--soft-foreground)] opacity-60 max-lg:hidden">
							This channel-level value defines how far back sync should go.
						</p>
					</div>
					<div class="flex flex-wrap items-center gap-2">
						<input
							type="date"
							class="min-w-[13rem] rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-white px-3 py-2 text-[12px] font-medium focus:outline-none focus:border-[var(--accent)]/40 transition-colors max-lg:flex-1"
							bind:value={earliestSyncDateInput}
							disabled={savingSyncDate}
						/>
						<button
							type="button"
							class="inline-flex items-center justify-center rounded-[var(--radius-sm)] bg-[var(--foreground)] px-4 py-2 text-[10px] font-bold uppercase tracking-[0.1em] text-white transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
							onclick={saveEarliestSyncDate}
							disabled={!earliestSyncDateInput || savingSyncDate}
							>
								{savingSyncDate ? "Saving..." : "Save"}
							</button>
						</div>
					</div>
				{/if}

			<div class="flex-1 overflow-y-auto custom-scrollbar">
				{#if !selectedChannelId}
					<div class="flex flex-col items-center justify-center py-24 opacity-30 text-center">
						<svg width="48" height="48" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1" stroke-linecap="round" stroke-linejoin="round" class="mb-6"><circle cx="12" cy="12" r="10"/><path d="M12 16v-4"/><path d="M12 8h.01"/></svg>
						<p class="text-[20px] font-serif italic text-[var(--soft-foreground)]">
							Choose a channel to inspect its heartbeat.
						</p>
					</div>
				{:else if loadingVideos && videos.length === 0}
					<div class="space-y-4 mt-4" role="status" aria-live="polite">
						{#each Array.from({ length: 4 }) as _, index (index)}
							<div
								class="animate-pulse rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--background)] p-6"
							>
								<div
									class="h-4 w-3/4 rounded-full bg-[var(--muted)] opacity-60"
								></div>
								<div
									class="mt-4 h-3 w-1/4 rounded-full bg-[var(--muted)] opacity-40"
								></div>
							</div>
						{/each}
					</div>
				{:else if queueStats.total === 0}
					<div class="flex flex-col items-center justify-center py-24 text-center">
						<div class="h-16 w-16 rounded-full bg-emerald-50 border border-emerald-100 flex items-center justify-center text-emerald-600 mb-6">
							<svg width="32" height="32" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round"><polyline points="20 6 9 17 4 12"/></svg>
						</div>
						<p class="text-[20px] font-serif italic text-[var(--soft-foreground)] opacity-60">
							The queue is clear. Every fragment has been distilled.
						</p>
					</div>
				{:else}
					<ul class="flex flex-col gap-4 mt-4 pb-20">
						{#each queuedVideosWithDistillationStatus as item}
							{@const video = item.video}
							<li
								class="rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-white transition-all duration-300 hover:border-[var(--accent)]/30 hover:shadow-lg hover:shadow-[var(--accent)]/5 max-lg:border-x-0 max-lg:rounded-none"
							>
								<button
									type="button"
									class="group w-full cursor-pointer p-6 text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 max-lg:p-4"
									onclick={() => openVideoTranscriptInWorkspace(video)}
									aria-label={`Open transcript workspace for ${video.title}`}
								>
									<div
										class="flex flex-wrap items-start justify-between gap-6 max-lg:gap-3"
									>
										<div class="min-w-0 flex-1">
											<p
												class="line-clamp-2 text-[16px] font-bold text-[var(--foreground)] leading-[1.4] tracking-tight group-hover:text-[var(--accent-strong)] transition-colors"
											>
												{video.title}
											</p>
											<p
												class="mt-2 text-[12px] font-medium text-[var(--soft-foreground)] opacity-50"
											>
												Published {formatDate(video.published_at)}
											</p>
											<div
												class="mt-6 flex items-center gap-4 max-lg:mt-4"
											>
												{#if item.distillationStatus.kind === "processing"}
													<div class="flex flex-col gap-1">
														<span
															class="inline-flex items-center gap-2 text-[10px] font-bold uppercase tracking-[0.2em] text-[var(--accent)]"
														>
															<span class="relative flex h-2 w-2">
																<span class="animate-ping absolute inline-flex h-full w-full rounded-full bg-[var(--accent)] opacity-75"></span>
																<span class="relative inline-flex rounded-full h-2 w-2 bg-[var(--accent)]"></span>
															</span>
															{item.distillationStatus.label}
														</span>
														<p class="text-[9px] font-bold text-[var(--accent-strong)]/70 uppercase tracking-widest ml-5">
															{item.distillationStatus.detail}
														</p>
													</div>
												{:else if item.distillationStatus.kind === "failed"}
													<div class="flex flex-col gap-1">
														<span
															class="inline-flex items-center gap-2 text-[10px] font-bold uppercase tracking-[0.2em] text-rose-600"
														>
															<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
															{item.distillationStatus.label}
														</span>
														<p class="text-[9px] font-bold text-rose-600/60 uppercase tracking-widest ml-5">
															{item.distillationStatus.detail}
														</p>
														{#if video.retry_count !== undefined && video.retry_count > 0}
															<p class="text-[9px] font-bold text-rose-600/60 uppercase tracking-widest ml-5">
																Attempt {video.retry_count} of {MAX_RETRIES}
															</p>
														{/if}
													</div>
												{:else}
													<div class="flex flex-col gap-1">
														<span
															class="inline-flex items-center gap-2 text-[10px] font-bold uppercase tracking-[0.2em] text-[var(--soft-foreground)] opacity-60"
														>
															<svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><polyline points="12 6 12 12 16 14"/></svg>
															{item.distillationStatus.label}
														</span>
														<p class="text-[9px] font-bold text-[var(--soft-foreground)]/60 uppercase tracking-widest ml-5">
															{item.distillationStatus.detail}
														</p>
													</div>
													{/if}
											</div>
										</div>
									</div>
								</button>
							</li>
						{/each}
					</ul>
				{/if}

				{#if hasMore && selectedChannelId}
					<div class="flex justify-center mt-4 max-lg:mb-20">
						<button
							type="button"
							class="inline-flex items-center justify-center rounded-full border border-[var(--border-soft)] bg-[var(--background)] px-10 py-3.5 text-[10px] font-bold uppercase tracking-[0.3em] text-[var(--soft-foreground)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--foreground)] hover:shadow-md disabled:opacity-30"
							onclick={() => loadVideos(false)}
							disabled={loadingVideos}
						>
							{loadingVideos ? "Retrieving…" : "Extend Library"}
						</button>
					</div>
				{/if}
			</div>
		</section>
		</main>

		<nav class="mobile-tab-bar lg:hidden" aria-label="Panel navigation">
			<button
				type="button"
				class="mobile-tab-item {mobileTab === 'channels' ? 'mobile-tab-item--active' : ''}"
				onclick={() => (mobileTab = 'channels')}
				aria-current={mobileTab === 'channels' ? 'page' : undefined}
			>
				<svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<rect x="3" y="3" width="6" height="18" rx="1.5" />
					<rect x="15" y="3" width="6" height="18" rx="1.5" />
					<rect x="9" y="3" width="6" height="18" rx="1.5" />
				</svg>
				<span>Channels</span>
			</button>
			<button
				type="button"
				class="mobile-tab-item {mobileTab === 'details' ? 'mobile-tab-item--active' : ''}"
				onclick={() => (mobileTab = 'details')}
				aria-current={mobileTab === 'details' ? 'page' : undefined}
			>
				<svg class="h-5 w-5" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1.5" stroke-linecap="round" stroke-linejoin="round">
					<circle cx="12" cy="12" r="10" />
					<line x1="12" y1="16" x2="12" y2="12" />
					<line x1="12" y1="8" x2="12.01" y2="8" />
				</svg>
				<span>Details</span>
			</button>
		</nav>
	{/if}

	{#if errorMessage}
		<div
			class="fixed bottom-8 left-1/2 z-50 w-[min(92vw,480px)] -translate-x-1/2 rounded-[var(--radius-md)] border-2 border-[var(--accent)]/10 bg-white p-5 shadow-2xl animate-fade-in"
			role="status"
			aria-live="polite"
		>
			<div class="flex items-start gap-4">
				<div class="flex h-6 w-6 shrink-0 items-center justify-center rounded-full bg-rose-50 text-rose-600">
					<svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"><circle cx="12" cy="12" r="10"/><line x1="12" y1="8" x2="12" y2="12"/><line x1="12" y1="16" x2="12.01" y2="16"/></svg>
				</div>
				<p class="text-[14px] font-bold text-rose-600 leading-tight pr-8">{errorMessage}</p>
				<button 
					onclick={() => errorMessage = null}
					class="absolute top-4 right-4 text-[var(--soft-foreground)] hover:text-[var(--foreground)] opacity-40"
					aria-label="Dismiss error"
				>
					<svg width="18" height="18" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2.5" stroke-linecap="round" stroke-linejoin="round"><line x1="18" y1="6" x2="6" y2="18"></line><line x1="6" y1="6" x2="18" y2="18"></line></svg>
				</button>
			</div>
		</div>
	{/if}

	<ConfirmationModal
		show={showDeleteConfirmation}
		title="Remove Channel?"
		message="Are you sure you want to remove this channel? All its downloaded transcripts and summaries will be permanently deleted."
		confirmLabel="Delete"
		cancelLabel="Keep"
		tone="danger"
		onConfirm={confirmDeleteChannel}
		onCancel={cancelDeleteChannel}
	/>
</div>
