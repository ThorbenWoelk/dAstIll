<script lang="ts">
	import { onMount, tick } from "svelte";
	import {
		addChannel,
		backfillChannelVideos,
		cleanTranscriptFormatting,
		deleteChannel,
		getChannelSyncDepth,
		getVideoInfo,
		getSummary,
		getTranscript,
		isAiAvailable,
		listChannels,
		listChannelsWhenAvailable,
		listVideos,
		refreshChannel,
		updateSummary,
		updateTranscript,
		updateAcknowledged,
		updateChannel,
	} from "$lib/api";
	import ChannelCard from "$lib/components/ChannelCard.svelte";
	import ContentEditor from "$lib/components/ContentEditor.svelte";
	import Toggle from "$lib/components/Toggle.svelte";
	import TranscriptView from "$lib/components/TranscriptView.svelte";
	import VideoCard from "$lib/components/VideoCard.svelte";
	import type {
		Channel,
		Summary as SummaryPayload,
		VideoInfo as VideoInfoPayload,
		Video,
		VideoTypeFilter,
	} from "$lib/types";
	import {
		applySavedChannelOrder,
		prioritizeChannelOrder,
		resolveInitialChannelSelection,
		WORKSPACE_STATE_KEY,
		type WorkspaceStateSnapshot,
	} from "$lib/channel-workspace";
	import {
		normalizeTranscriptForRender,
		renderMarkdown,
	} from "$lib/utils/markdown";

	const secondaryButtonClass =
		"inline-flex items-center justify-center rounded-full border border-[var(--border)] px-5 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-[var(--foreground)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]";

	const channelSubmitButtonClass =
		"inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--surface)] text-xl leading-none text-[var(--accent)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent-strong)] disabled:cursor-not-allowed disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]";
	const FORMAT_MAX_TURNS = 5;
	const FORMAT_HARD_TIMEOUT_MINUTES = 5;

	type AcknowledgedFilter = "all" | "unack" | "ack";

	let channels = $state<Channel[]>([]);
	let channelOrder = $state<string[]>([]);
	let videos = $state<Video[]>([]);
	let selectedChannelId = $state<string | null>(null);
	let selectedVideoId = $state<string | null>(null);
	let draggedChannelId = $state<string | null>(null);
	let dragOverChannelId = $state<string | null>(null);

	let channelInput = $state("");
	let loadingChannels = $state(false);
	let aiAvailable = $state<boolean | null>(null);
	let loadingVideos = $state(false);
	let loadingContent = $state(false);
	let waitingForBackend = $state(false);
	let addingChannel = $state(false);
	let errorMessage = $state<string | null>(null);
	let summaryQualityScore = $state<number | null>(null);
	let summaryQualityNote = $state<string | null>(null);
	let videoInfo = $state<VideoInfoPayload | null>(null);
	let syncDepth = $state<{
		earliest_sync_date: string | null;
		earliest_sync_date_user_set: boolean;
		derived_earliest_ready_date: string | null;
	} | null>(null);

	let contentMode = $state<"transcript" | "summary" | "info">("transcript");
	let contentText = $state("");
	let contentRenderText = $derived(
		contentMode === "transcript"
			? normalizeTranscriptForRender(contentText)
			: contentText,
	);
	let contentHtml = $derived(renderMarkdown(contentRenderText));
	let editing = $state(false);
	let draft = $state("");
	let formattingContent = $state(false);
	let formattingVideoId = $state<string | null>(null);
	let revertingContent = $state(false);
	let revertingVideoId = $state<string | null>(null);
	let originalTranscriptByVideoId = $state<Record<string, string>>({});
	let formattingNotice = $state<string | null>(null);
	let formattingNoticeVideoId = $state<string | null>(null);
	let formattingNoticeTone = $state<"info" | "success" | "warning">("info");
	let formattingAttemptsUsed = $state<number | null>(null);
	let formattingAttemptsMax = $state<number | null>(null);
	let formattingAttemptsVideoId = $state<string | null>(null);
	let formattingRequestSeq = 0;
	let activeFormattingRequest = $state(0);
	let contentRequestSeq = 0;
	let activeContentRequestId = 0;

	let offset = $state(0);
	const limit = 20;
	let hasMore = $state(true);
	let historyExhausted = $state(false);
	let backfillingHistory = $state(false);

	let videoTypeFilter = $state<VideoTypeFilter>("all");
	let acknowledgedFilter = $state<AcknowledgedFilter>("all");
	let workspaceStateHydrated = $state(false);
	let filterMenuOpen = $state(false);
	let filterMenuContainer = $state<HTMLDivElement | null>(null);
	let videoListContainer = $state<HTMLDivElement | null>(null);
	let atVideoListBottom = $state(false);
	let filterMenuLabel = $derived(
		videoTypeFilter === "all"
			? "Open video filter menu."
			: `Video type filter set to ${videoTypeFilter}. Open filter menu.`,
	);

	const selectedChannel = $derived(
		channels.find((channel) => channel.id === selectedChannelId) ?? null,
	);

	function resolveOldestLoadedVideoDate(): Date | null {
		let oldest: Date | null = null;
		for (const video of videos) {
			if (
				video.transcript_status !== "ready" ||
				video.summary_status !== "ready"
			)
				continue;
			const parsed = new Date(video.published_at);
			if (Number.isNaN(parsed.getTime())) continue;
			if (!oldest || parsed < oldest) {
				oldest = parsed;
			}
		}
		return oldest;
	}

	function resolveDisplayedSyncDepthIso(): string | null {
		const oldestLoaded = resolveOldestLoadedVideoDate();
		if (oldestLoaded) {
			return oldestLoaded.toISOString();
		}
		if (selectedChannel?.earliest_sync_date_user_set) {
			return selectedChannel.earliest_sync_date ?? null;
		}
		return (
			syncDepth?.derived_earliest_ready_date ??
			selectedChannel?.earliest_sync_date ??
			null
		);
	}

	async function loadSyncDepth() {
		if (!selectedChannelId) {
			syncDepth = null;
			return;
		}
		try {
			syncDepth = await getChannelSyncDepth(selectedChannelId);
		} catch {
			syncDepth = null;
		}
	}

	async function syncEarliestDateFromLoadedVideos() {
		if (!selectedChannelId || !selectedChannel) return;
		if (selectedChannel.earliest_sync_date_user_set) return;

		const oldest = resolveOldestLoadedVideoDate();
		if (!oldest) return;

		const currentEarliest = selectedChannel.earliest_sync_date
			? new Date(selectedChannel.earliest_sync_date)
			: null;
		const shouldPushBack =
			!currentEarliest ||
			Number.isNaN(currentEarliest.getTime()) ||
			oldest < currentEarliest;
		if (!shouldPushBack) return;

		const updated = await updateChannel(selectedChannelId, {
			earliest_sync_date: oldest.toISOString(),
		});
		channels = channels.map((channel) =>
			channel.id === selectedChannelId ? updated : channel,
		);
		void loadSyncDepth();
	}
	const selectedVideoYoutubeUrl = $derived(
		selectedVideoId
			? `https://www.youtube.com/watch?v=${selectedVideoId}`
			: null,
	);
	const selectedOriginalTranscript = $derived(
		selectedVideoId
			? (originalTranscriptByVideoId[selectedVideoId] ?? null)
			: null,
	);
	const canRevertTranscript = $derived(
		contentMode === "transcript" &&
			selectedOriginalTranscript !== null &&
			(editing
				? draft !== selectedOriginalTranscript
				: contentText !== selectedOriginalTranscript),
	);

	function isContentMode(
		value: unknown,
	): value is "transcript" | "summary" | "info" {
		return (
			value === "transcript" || value === "summary" || value === "info"
		);
	}

	function isVideoTypeFilter(value: unknown): value is VideoTypeFilter {
		return value === "all" || value === "long" || value === "short";
	}

	function stripPrefix(text: string): string {
		return text.replace(/^(?:Transcript|Summary):\s*/i, "").trimStart();
	}

	function syncChannelOrderFromList() {
		channelOrder = channels.map((channel) => channel.id);
	}

	function applySummaryQuality(summary: SummaryPayload) {
		summaryQualityScore =
			typeof summary.quality_score === "number"
				? Math.max(0, Math.min(10, Math.round(summary.quality_score)))
				: null;
		summaryQualityNote = summary.quality_note?.trim() || null;
	}

	function resetSummaryQuality() {
		summaryQualityScore = null;
		summaryQualityNote = null;
	}

	function resetVideoInfo() {
		videoInfo = null;
	}

	function clearFormattingFeedback() {
		formattingNotice = null;
		formattingNoticeVideoId = null;
		formattingAttemptsUsed = null;
		formattingAttemptsMax = null;
		formattingAttemptsVideoId = null;
	}

	function isCurrentContentRequest(
		requestId: number,
		targetVideoId: string,
		targetMode: "transcript" | "summary" | "info",
	) {
		return (
			activeContentRequestId === requestId &&
			selectedVideoId === targetVideoId &&
			contentMode === targetMode
		);
	}

	function formatPublishedAt(value: string | null | undefined) {
		if (!value) return "Unknown";
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return value;
		return new Intl.DateTimeFormat(undefined, {
			dateStyle: "long",
			timeStyle: "short",
		}).format(date);
	}

	function formatSyncDate(value: string | null | undefined) {
		if (!value) return "Unknown";
		const date = new Date(value);
		if (Number.isNaN(date.getTime())) return "Unknown";
		return new Intl.DateTimeFormat(undefined, {
			dateStyle: "long",
		}).format(date);
	}

	function formatCount(value: number | null | undefined) {
		if (value === null || value === undefined) return "Unknown";
		return new Intl.NumberFormat().format(value);
	}

	function formatDuration(
		seconds: number | null | undefined,
		iso8601: string | null | undefined,
	) {
		if (seconds !== null && seconds !== undefined && seconds >= 0) {
			const hrs = Math.floor(seconds / 3600);
			const mins = Math.floor((seconds % 3600) / 60);
			const secs = seconds % 60;
			if (hrs > 0) {
				return `${hrs}h ${mins}m ${secs}s`;
			}
			return `${mins}m ${secs}s`;
		}
		if (iso8601) return iso8601;
		return "Unknown";
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
			if (
				typeof snapshot.selectedVideoId === "string" ||
				snapshot.selectedVideoId === null
			) {
				selectedVideoId = snapshot.selectedVideoId;
			}
			if (isContentMode(snapshot.contentMode)) {
				contentMode = snapshot.contentMode;
			}
			if (isVideoTypeFilter(snapshot.videoTypeFilter)) {
				videoTypeFilter = snapshot.videoTypeFilter;
			} else if (typeof snapshot.hideShorts === "boolean") {
				videoTypeFilter = snapshot.hideShorts ? "long" : "all";
			}
			if (
				snapshot.acknowledgedFilter &&
				["all", "unack", "ack"].includes(snapshot.acknowledgedFilter)
			) {
				acknowledgedFilter = snapshot.acknowledgedFilter;
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
		const snapshot: WorkspaceStateSnapshot = {
			selectedChannelId,
			selectedVideoId,
			contentMode,
			videoTypeFilter,
			acknowledgedFilter,
			channelOrder,
		};
		localStorage.setItem(WORKSPACE_STATE_KEY, JSON.stringify(snapshot));
	}

	$effect(() => {
		persistWorkspaceState();
	});

	onMount(() => {
		restoreWorkspaceState();
		workspaceStateHydrated = true;
		waitingForBackend = true;
		void loadChannels(null, true);

		const handlePointerDown = (event: PointerEvent) => {
			if (!filterMenuOpen || !filterMenuContainer) return;
			if (!filterMenuContainer.contains(event.target as Node)) {
				filterMenuOpen = false;
			}
		};

		document.addEventListener("pointerdown", handlePointerDown);
		return () => {
			document.removeEventListener("pointerdown", handlePointerDown);
		};
	});

	async function loadChannels(
		preferredChannelId: string | null = null,
		retryUntilBackendReachable = false,
	) {
		loadingChannels = true;
		errorMessage = null;
		let initialChannelId: string | null = null;
		let preferredVideoId: string | null = null;

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
			syncChannelOrderFromList();
			initialChannelId = resolveInitialChannelSelection(
				channels,
				selectedChannelId,
				preferredChannelId,
			);
			if (!initialChannelId) {
				selectedChannelId = null;
				selectedVideoId = null;
			} else {
				preferredVideoId =
					initialChannelId === selectedChannelId
						? selectedVideoId
						: null;
			}
		} catch (error) {
			waitingForBackend = false;
			errorMessage = (error as Error).message;
		} finally {
			loadingChannels = false;
		}

		if (initialChannelId) {
			await selectChannel(initialChannelId, preferredVideoId);
		}
	}

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

	async function handleAddChannel(input: string) {
		if (!input.trim()) return;

		addingChannel = true;
		errorMessage = null;

		try {
			const channel = await addChannel(input.trim());
			channelInput = "";
			channelOrder = prioritizeChannelOrder(channelOrder, channel.id);
			await loadChannels(channel.id);
		} catch (error) {
			errorMessage = (error as Error).message;
		} finally {
			addingChannel = false;
		}
	}

	function handleChannelSubmit(event: SubmitEvent) {
		event.preventDefault();
		handleAddChannel(channelInput);
	}

	async function handleDeleteChannel(channelId: string) {
		if (
			!confirm(
				"Are you sure you want to delete this channel? All its downloaded data will be removed.",
			)
		)
			return;
		try {
			await deleteChannel(channelId);
			channelOrder = channelOrder.filter((id) => id !== channelId);
			if (selectedChannelId === channelId) {
				selectedChannelId = null;
				selectedVideoId = null;
			}
			await loadChannels(selectedChannelId);
		} catch (error) {
			errorMessage = (error as Error).message;
		}
	}

	$effect(() => {
		if (selectedChannelId) {
			void loadSyncDepth();
		} else {
			syncDepth = null;
		}
	});

	async function selectChannel(
		channelId: string,
		preferredVideoId: string | null = null,
	) {
		selectedChannelId = channelId;
		selectedVideoId = preferredVideoId;
		contentText = "";
		draft = "";
		resetSummaryQuality();
		resetVideoInfo();
		editing = false;
		clearFormattingFeedback();
		videos = [];
		offset = 0;
		hasMore = true;
		historyExhausted = false;
		backfillingHistory = false;
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
			const isAck =
				acknowledgedFilter === "ack"
					? true
					: acknowledgedFilter === "unack"
						? false
						: undefined;
			const list = await listVideos(
				selectedChannelId,
				limit,
				reset ? 0 : offset,
				videoTypeFilter,
				isAck,
			);
			videos = reset ? list : [...videos, ...list];
			offset = (reset ? 0 : offset) + list.length;
			hasMore = list.length === limit;

			if (reset) {
				if (videos.length === 0) {
					selectedVideoId = null;
					contentText = "";
					draft = "";
					resetSummaryQuality();
					resetVideoInfo();
					return;
				}

				if (!selectedVideoId) {
					void selectVideo(videos[0].id);
					return;
				}

				const hasSelectedVideo = videos.some(
					(video) => video.id === selectedVideoId,
				);
				if (!hasSelectedVideo) {
					void selectVideo(videos[0].id);
					return;
				}

				if (!loadingContent && contentText.trim().length === 0) {
					void loadContent();
				}
			}
		} catch (error) {
			if (!silent || !errorMessage)
				errorMessage = (error as Error).message;
		} finally {
			if (!silent) loadingVideos = false;
		}
	}

	async function loadMoreVideos() {
		if (!selectedChannelId || loadingVideos || backfillingHistory) return;

		if (hasMore) {
			await loadVideos(false);
			await syncEarliestDateFromLoadedVideos();
			return;
		}

		backfillingHistory = true;
		errorMessage = null;

		try {
			// Try to backfill a batch of 50
			const result = await backfillChannelVideos(selectedChannelId, 50);

			// Use the explicit flag from backend to know if we hit the actual end of YouTube results
			if (result.exhausted) {
				historyExhausted = true;
			}

			// Load the newly added videos (if any) or just try to see if we can find more older ones
			await loadVideos(false);
			await syncEarliestDateFromLoadedVideos();
		} catch (error) {
			errorMessage = (error as Error).message;
		} finally {
			backfillingHistory = false;
		}
	}

	async function selectVideo(videoId: string) {
		if (videoId === selectedVideoId) return;
		selectedVideoId = videoId;
		resetSummaryQuality();
		resetVideoInfo();
		editing = false;
		clearFormattingFeedback();
		await loadContent();
	}

	async function setMode(mode: "transcript" | "summary" | "info") {
		if (contentMode === mode) return;
		contentMode = mode;
		resetSummaryQuality();
		resetVideoInfo();
		editing = false;
		clearFormattingFeedback();
		await loadContent();
	}

	async function loadContent() {
		if (!selectedVideoId) return;
		const targetVideoId = selectedVideoId;
		const targetMode = contentMode;
		const requestId = ++contentRequestSeq;
		activeContentRequestId = requestId;

		loadingContent = true;
		errorMessage = null;

		try {
			if (targetMode === "transcript") {
				const transcript = await getTranscript(targetVideoId);
				if (
					!isCurrentContentRequest(
						requestId,
						targetVideoId,
						targetMode,
					)
				)
					return;
				const originalTranscript = stripPrefix(
					transcript.raw_text ||
						transcript.formatted_markdown ||
						"Transcript unavailable.",
				);
				contentText = stripPrefix(
					transcript.formatted_markdown ||
						transcript.raw_text ||
						"Transcript unavailable.",
				);
				if (!(targetVideoId in originalTranscriptByVideoId)) {
					originalTranscriptByVideoId = {
						...originalTranscriptByVideoId,
						[targetVideoId]: originalTranscript,
					};
				}
				resetSummaryQuality();
				resetVideoInfo();
			} else {
				if (targetMode === "summary") {
					const summary = await getSummary(targetVideoId);
					if (
						!isCurrentContentRequest(
							requestId,
							targetVideoId,
							targetMode,
						)
					)
						return;
					contentText = stripPrefix(
						summary.content || "Summary unavailable.",
					);
					applySummaryQuality(summary);
					resetVideoInfo();
				} else {
					const info = await getVideoInfo(targetVideoId);
					if (
						!isCurrentContentRequest(
							requestId,
							targetVideoId,
							targetMode,
						)
					)
						return;
					videoInfo = info;
					contentText = "";
					resetSummaryQuality();
				}
			}
			if (!isCurrentContentRequest(requestId, targetVideoId, targetMode))
				return;
			draft = contentText;
		} catch (error) {
			if (activeContentRequestId === requestId) {
				errorMessage = (error as Error).message;
			}
		} finally {
			if (activeContentRequestId === requestId) {
				loadingContent = false;
				activeContentRequestId = 0;
			}
		}
	}

	function startEdit() {
		editing = true;
		draft = contentText;
	}

	function cancelEdit() {
		editing = false;
		draft = contentText;
	}

	async function saveEdit() {
		if (!selectedVideoId) return;
		if (contentMode === "info") return;

		loadingContent = true;
		errorMessage = null;

		try {
			if (contentMode === "transcript") {
				const transcript = await updateTranscript(
					selectedVideoId,
					draft,
				);
				contentText = stripPrefix(
					transcript.formatted_markdown ||
						transcript.raw_text ||
						"Transcript unavailable.",
				);
				resetSummaryQuality();
				resetVideoInfo();
			} else {
				const summary = await updateSummary(selectedVideoId, draft);
				contentText = stripPrefix(
					summary.content || "Summary unavailable.",
				);
				applySummaryQuality(summary);
				resetVideoInfo();
			}
			editing = false;
		} catch (error) {
			errorMessage = (error as Error).message;
		} finally {
			loadingContent = false;
		}
	}

	async function cleanFormatting() {
		if (!selectedVideoId || contentMode !== "transcript") return;
		const targetVideoId = selectedVideoId;
		const startedInEditMode = editing;
		const source = startedInEditMode ? draft : contentText;
		const requestId = ++formattingRequestSeq;

		activeFormattingRequest = requestId;
		formattingContent = true;
		formattingVideoId = targetVideoId;
		errorMessage = null;
		formattingNotice = `Formatting transcript with Ollama… (up to ${FORMAT_MAX_TURNS} tries, ${FORMAT_HARD_TIMEOUT_MINUTES} minute cutoff)`;
		formattingNoticeVideoId = targetVideoId;
		formattingNoticeTone = "info";
		formattingAttemptsUsed = 0;
		formattingAttemptsMax = FORMAT_MAX_TURNS;
		formattingAttemptsVideoId = targetVideoId;

		try {
			const result = await cleanTranscriptFormatting(
				targetVideoId,
				source,
			);
			const attemptsSummary = `Attempts ${result.attempts_used}/${result.max_attempts}.`;
			formattingAttemptsUsed = result.attempts_used;
			formattingAttemptsMax = result.max_attempts;
			formattingAttemptsVideoId = targetVideoId;
			if (startedInEditMode) {
				if (
					activeFormattingRequest === requestId &&
					selectedVideoId === targetVideoId &&
					editing
				) {
					draft = result.content;
				}
				formattingNotice =
					result.content === source
						? `No formatting changes. ${attemptsSummary}`
						: `Formatting applied to draft. Save to persist. ${attemptsSummary}`;
				formattingNoticeVideoId = targetVideoId;
			} else {
				if (result.content !== source) {
					const transcript = await updateTranscript(
						targetVideoId,
						result.content,
					);
					if (
						activeFormattingRequest === requestId &&
						selectedVideoId === targetVideoId &&
						!editing
					) {
						contentText = stripPrefix(
							transcript.formatted_markdown ||
								transcript.raw_text ||
								"Transcript unavailable.",
						);
						draft = contentText;
					}
				}
				formattingNotice =
					result.content === source
						? `No formatting changes. ${attemptsSummary}`
						: `Formatting applied and saved. ${attemptsSummary}`;
				formattingNoticeVideoId = targetVideoId;
			}
			formattingNoticeTone = "success";
			if (result.timed_out) {
				formattingNotice = `Formatting stopped after ${FORMAT_HARD_TIMEOUT_MINUTES} minutes. Current transcript was kept. ${attemptsSummary}`;
				formattingNoticeVideoId = targetVideoId;
				formattingNoticeTone = "warning";
			} else if (!result.preserved_text) {
				errorMessage =
					"Formatting changed transcript words. Original transcript text was kept.";
				formattingNotice = `Safety guard kept original wording. Only spacing changes are allowed. ${attemptsSummary}`;
				formattingNoticeVideoId = targetVideoId;
				formattingNoticeTone = "warning";
			}
		} catch (error) {
			errorMessage = (error as Error).message;
			clearFormattingFeedback();
		} finally {
			if (activeFormattingRequest === requestId) {
				formattingContent = false;
				formattingVideoId = null;
			}
		}
	}

	async function revertToOriginalTranscript() {
		if (!selectedVideoId || contentMode !== "transcript") return;
		const targetVideoId = selectedVideoId;
		const original = originalTranscriptByVideoId[targetVideoId];
		if (!original) return;

		const startedInEditMode = editing;
		const source = startedInEditMode ? draft : contentText;
		if (source === original) {
			formattingNotice = "Already showing the original transcript.";
			formattingNoticeVideoId = targetVideoId;
			formattingNoticeTone = "info";
			formattingAttemptsUsed = null;
			formattingAttemptsMax = null;
			formattingAttemptsVideoId = null;
			return;
		}

		revertingContent = true;
		revertingVideoId = targetVideoId;
		errorMessage = null;
		formattingNotice = startedInEditMode
			? "Reverting draft to original transcript…"
			: "Reverting transcript to original…";
		formattingNoticeVideoId = targetVideoId;
		formattingNoticeTone = "info";

		try {
			if (startedInEditMode) {
				if (selectedVideoId === targetVideoId && editing) {
					draft = original;
				}
				formattingNotice =
					"Draft reset to original transcript. Save to persist.";
			} else {
				const transcript = await updateTranscript(
					targetVideoId,
					original,
				);
				if (selectedVideoId === targetVideoId && !editing) {
					contentText = stripPrefix(
						transcript.formatted_markdown ||
							transcript.raw_text ||
							"Transcript unavailable.",
					);
					draft = contentText;
				}
				formattingNotice = "Original transcript restored.";
			}
			formattingNoticeVideoId = targetVideoId;
			formattingNoticeTone = "success";
		} catch (error) {
			errorMessage = (error as Error).message;
			clearFormattingFeedback();
		} finally {
			revertingContent = false;
			revertingVideoId = null;
		}
	}

	async function setVideoTypeFilter(nextValue: VideoTypeFilter) {
		filterMenuOpen = false;
		if (videoTypeFilter === nextValue) return;
		videoTypeFilter = nextValue;
		await loadVideos(true);
	}

	async function setAcknowledgedFilter(nextValue: AcknowledgedFilter) {
		filterMenuOpen = false;
		if (acknowledgedFilter === nextValue) return;
		acknowledgedFilter = nextValue;
		await loadVideos(true);
	}

	function matchesAcknowledgedFilter(video: Video) {
		if (acknowledgedFilter === "ack") return video.acknowledged;
		if (acknowledgedFilter === "unack") return !video.acknowledged;
		return true;
	}

	async function toggleAcknowledge() {
		if (!selectedVideoId) return;
		const video = videos.find((v) => v.id === selectedVideoId);
		if (!video) return;

		loadingContent = true;
		errorMessage = null;

		try {
			const updated = await updateAcknowledged(
				selectedVideoId,
				!video.acknowledged,
			);
			videos = videos
				.map((v) => (v.id === updated.id ? updated : v))
				.filter(matchesAcknowledgedFilter);

			const stillSelected = videos.some((v) => v.id === selectedVideoId);
			if (!stillSelected) {
				editing = false;
				clearFormattingFeedback();
				if (videos.length === 0) {
					selectedVideoId = null;
					contentText = "";
					draft = "";
				} else {
					await selectVideo(videos[0].id);
				}
			}
		} catch (error) {
			errorMessage = (error as Error).message;
		} finally {
			loadingContent = false;
		}
	}

	function toggleFilterMenu() {
		filterMenuOpen = !filterMenuOpen;
	}

	function handleWindowKeydown(event: KeyboardEvent) {
		if (event.key === "Escape") {
			filterMenuOpen = false;
		}
	}

	function updateVideoListBottomState() {
		if (!videoListContainer) {
			atVideoListBottom = false;
			return;
		}
		const thresholdPx = 12;
		atVideoListBottom =
			videoListContainer.scrollTop + videoListContainer.clientHeight >=
			videoListContainer.scrollHeight - thresholdPx;
	}

	async function refreshSummaryQuality() {
		if (
			!selectedVideoId ||
			contentMode !== "summary" ||
			editing ||
			loadingContent
		)
			return;
		const targetVideoId = selectedVideoId;
		try {
			const summary = await getSummary(targetVideoId);
			if (
				selectedVideoId !== targetVideoId ||
				contentMode !== "summary" ||
				editing
			)
				return;
			applySummaryQuality(summary);
		} catch {
			// Keep previous quality state if background refresh fails.
		}
	}

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

	$effect(() => {
		if (
			contentMode !== "summary" ||
			!selectedVideoId ||
			editing ||
			loadingContent ||
			summaryQualityScore !== null ||
			summaryQualityNote !== null
		) {
			return;
		}

		const timer = setInterval(() => {
			void refreshSummaryQuality();
		}, 7000);
		return () => clearInterval(timer);
	});

	$effect(() => {
		selectedChannelId;
		videos.length;
		loadingVideos;
		void tick().then(updateVideoListBottomState);
	});
</script>

<svelte:window onkeydown={handleWindowKeydown} />

<div class="page-shell min-h-screen px-4 py-6 sm:px-8">
	<a
		href="#main-content"
		class="skip-link absolute left-4 top-4 z-50 rounded-full bg-[var(--accent)] px-4 py-2 text-sm font-semibold text-white"
	>
		Skip to Main Content
	</a>

	<header
		class="mx-auto flex w-full max-w-[1440px] items-center justify-between gap-6 px-4 sm:px-2 fade-in border-b border-[var(--border-soft)] pb-4 mb-2"
	>
		<div class="flex items-center gap-4">
			<h1
				class="text-3xl font-bold tracking-tighter text-[var(--foreground)]"
			>
				DASTILL
			</h1>
			<span class="hidden sm:block h-5 w-px bg-[var(--border-soft)]"
			></span>
			<p
				class="hidden sm:block text-[10px] font-bold uppercase tracking-[0.3em] text-[var(--accent)] opacity-60"
			>
				v1.0.0
			</p>
			{#if aiAvailable === false}
				<span class="hidden sm:block h-3 w-px bg-[var(--border-soft)]"
				></span>
				<p
					class="hidden sm:block text-[9px] font-bold uppercase tracking-[0.2em] text-[var(--accent)]"
				>
					Ollama Offline
				</p>
			{:else if aiAvailable === true}
				<span class="hidden sm:block h-3 w-px bg-[var(--border-soft)]"
				></span>
				<p
					class="hidden sm:block text-[9px] font-bold uppercase tracking-[0.2em] text-[var(--soft-foreground)] opacity-40"
				>
					Ollama Ready
				</p>
			{/if}
		</div>

		<nav
			class="flex items-center gap-1 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] p-1 shadow-sm"
			aria-label="Workspace sections"
		>
			<a
				href="#workspace"
				class="rounded-[var(--radius-sm)] bg-[var(--muted)]/60 px-5 py-2 text-[10px] font-bold uppercase tracking-[0.15em] text-[var(--foreground)] transition-all"
			>
				Workspace
			</a>
			<a
				href="/download-queue"
				class="rounded-[var(--radius-sm)] px-5 py-2 text-[10px] font-bold uppercase tracking-[0.15em] text-[var(--soft-foreground)] opacity-60 transition-all hover:opacity-100 hover:bg-[var(--muted)]/30"
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
				<div
					class="relative flex h-12 w-12 items-center justify-center"
				>
					<div
						class="absolute h-full w-full animate-ping rounded-full bg-[var(--accent)] opacity-10"
					></div>
					<div
						class="h-8 w-8 animate-spin rounded-full border-2 border-[var(--muted)] border-t-[var(--accent)]"
					></div>
				</div>
				<div class="space-y-3">
					<p
						class="text-[11px] font-bold uppercase tracking-[0.3em] text-[var(--accent)]"
					>
						Establishing Link
					</p>
					<p
						class="max-w-xs text-[15px] font-medium text-[var(--soft-foreground)] opacity-70"
					>
						Waiting for the distillation engine to become reachable.
					</p>
				</div>
			</section>
		</main>
	{:else}
		<main
			id="main-content"
			class="mx-auto mt-10 grid w-full max-w-[1440px] items-start gap-10 lg:grid-cols-[280px_320px_minmax(0,1fr)] xl:grid-cols-[280px_380px_minmax(0,1fr)]"
		>
			<aside
				class="flex h-fit flex-col gap-6 rounded-[var(--radius-lg)] bg-[var(--surface)] border border-[var(--border-soft)] p-6 lg:sticky lg:top-8 fade-in stagger-1 shadow-sm"
				id="workspace"
			>
				<div class="flex items-center justify-between gap-2">
					<h2 class="text-xl font-bold tracking-tight">Channels</h2>
				</div>

				<form
					class="grid gap-3"
					onsubmit={handleChannelSubmit}
					aria-label="Follow channel"
				>
					<div
						class="flex min-w-0 items-center gap-3 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--background)] pl-5 pr-1.5 transition-all focus-within:ring-2 focus-within:ring-[var(--accent)]/20 focus-within:border-[var(--accent)]/40"
					>
						<label for="channel-input" class="sr-only"
							>Add Channel</label
						>
						<input
							id="channel-input"
							name="channel"
							autocomplete="off"
							spellcheck={false}
							class="min-w-0 flex-1 bg-transparent py-3 text-[14px] font-medium placeholder:text-[var(--soft-foreground)] placeholder:opacity-40 focus-visible:outline-none"
							placeholder="Channel URL or Handle"
							bind:value={channelInput}
						/>
						<button
							type="submit"
							class="inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-[var(--radius-sm)] bg-[var(--foreground)] text-white transition-all hover:bg-[var(--accent-strong)] disabled:opacity-20 disabled:grayscale"
							disabled={!channelInput.trim() || addingChannel}
							aria-label="Follow channel"
						>
							<svg
								width="18"
								height="18"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2.5"
								stroke-linecap="round"
								stroke-linejoin="round"
								><line x1="12" y1="5" x2="12" y2="19"
								></line><line x1="5" y1="12" x2="19" y2="12"
								></line></svg
							>
						</button>
					</div>
				</form>

				<div
					class="flex max-h-[60vh] flex-col gap-1.5 overflow-y-auto pr-1 custom-scrollbar"
					aria-busy={loadingChannels}
				>
					{#if loadingChannels}
						<div class="space-y-4" role="status" aria-live="polite">
							{#each Array.from( { length: 4 }, ) as _, index (index)}
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
						<p
							class="px-1 text-[14px] font-medium text-[var(--soft-foreground)] opacity-50 italic"
						>
							Start by following a channel.
						</p>
					{:else}
						{#each channels as channel}
							<ChannelCard
								{channel}
								active={selectedChannelId === channel.id}
								draggableEnabled
								dragging={draggedChannelId === channel.id}
								dragOver={dragOverChannelId === channel.id &&
									draggedChannelId !== channel.id}
								onSelect={() => selectChannel(channel.id)}
								onDragStart={(event) =>
									handleChannelDragStart(channel.id, event)}
								onDragOver={(event) =>
									handleChannelDragOver(channel.id, event)}
								onDrop={(event) =>
									handleChannelDrop(channel.id, event)}
								onDragEnd={handleChannelDragEnd}
								onDelete={() => handleDeleteChannel(channel.id)}
							/>
						{/each}
					{/if}
				</div>
			</aside>

			<aside
				class="flex h-fit min-w-0 flex-col gap-6 rounded-[var(--radius-lg)] bg-[var(--surface)] border border-[var(--border-soft)] p-6 lg:sticky lg:top-8 fade-in stagger-2 shadow-sm"
				id="videos"
			>
				<div class="flex flex-wrap items-center justify-between gap-4">
					<div class="min-w-0">
						<h2 class="text-xl font-bold tracking-tight">
							Knowledge Library
							{#if refreshingChannel}
								<span
									class="ml-3 inline-flex h-5 w-5 items-center justify-center align-middle rounded-full border border-[var(--border-soft)] bg-[var(--background)]"
									role="status"
									aria-label="Syncing channel"
									title="Syncing channel"
								>
									<span
										class="h-2.5 w-2.5 animate-spin rounded-full border border-[var(--accent)]/20 border-t-[var(--accent)]"
									></span>
									<span class="sr-only">Syncing</span>
								</span>
							{/if}
						</h2>
					</div>
					<div class="relative" bind:this={filterMenuContainer}>
						<button
							type="button"
							class={`group flex h-9 w-9 items-center justify-center rounded-[var(--radius-sm)] transition-all duration-300 ${videoTypeFilter !== "all" || filterMenuOpen ? "bg-[var(--accent)] text-white shadow-lg shadow-[var(--accent)]/20" : "text-[var(--soft-foreground)] hover:bg-[var(--muted)] border border-[var(--border-soft)] bg-[var(--background)]"} focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 disabled:opacity-20`}
							onclick={toggleFilterMenu}
							disabled={!selectedChannelId || loadingVideos}
							aria-label={filterMenuLabel}
							aria-haspopup="menu"
							aria-expanded={filterMenuOpen}
							aria-controls="video-filter-menu"
						>
							<svg
								width="16"
								height="16"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="2.5"
								stroke-linecap="round"
								stroke-linejoin="round"
							>
								<line x1="3" y1="6" x2="21" y2="6"></line>
								<line x1="7" y1="12" x2="17" y2="12"></line>
								<line x1="10" y1="18" x2="14" y2="18"></line>
							</svg>
						</button>
						{#if filterMenuOpen}
							<div
								id="video-filter-menu"
								role="menu"
								aria-label="Video filters"
								class="absolute right-0 top-full z-20 mt-3 w-64 overflow-hidden rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--surface)] shadow-2xl animate-fade-in"
							>
								<div class="bg-[var(--muted)]/30 px-4 py-2.5">
									<p
										class="text-[9px] font-bold uppercase tracking-[0.3em] text-[var(--soft-foreground)]"
									>
										Refine View
									</p>
								</div>
								<div class="p-2 space-y-4">
									<div class="grid gap-1">
										<p
											class="px-2 pb-1 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50"
										>
											TYPE
										</p>
										<button
											type="button"
											role="menuitemradio"
											aria-checked={videoTypeFilter ===
												"all"}
											class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "all" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
											onclick={() =>
												setVideoTypeFilter("all")}
										>
											<span>All Content</span>
											{#if videoTypeFilter === "all"}
												<svg
													width="12"
													height="12"
													viewBox="0 0 24 24"
													fill="none"
													stroke="currentColor"
													stroke-width="3"
													stroke-linecap="round"
													stroke-linejoin="round"
													><polyline
														points="20 6 9 17 4 12"
													/></svg
												>
											{/if}
										</button>
										<button
											type="button"
											role="menuitemradio"
											aria-checked={videoTypeFilter ===
												"long"}
											class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "long" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
											onclick={() =>
												setVideoTypeFilter("long")}
										>
											<span>Full Videos</span>
											{#if videoTypeFilter === "long"}
												<svg
													width="12"
													height="12"
													viewBox="0 0 24 24"
													fill="none"
													stroke="currentColor"
													stroke-width="3"
													stroke-linecap="round"
													stroke-linejoin="round"
													><polyline
														points="20 6 9 17 4 12"
													/></svg
												>
											{/if}
										</button>
										<button
											type="button"
											role="menuitemradio"
											aria-checked={videoTypeFilter ===
												"short"}
											class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${videoTypeFilter === "short" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
											onclick={() =>
												setVideoTypeFilter("short")}
										>
											<span>Shorts</span>
											{#if videoTypeFilter === "short"}
												<svg
													width="12"
													height="12"
													viewBox="0 0 24 24"
													fill="none"
													stroke="currentColor"
													stroke-width="3"
													stroke-linecap="round"
													stroke-linejoin="round"
													><polyline
														points="20 6 9 17 4 12"
													/></svg
												>
											{/if}
										</button>
									</div>

									<div class="grid gap-1">
										<p
											class="px-2 pb-1 text-[10px] font-bold text-[var(--soft-foreground)] opacity-50"
										>
											STATUS
										</p>
										<button
											type="button"
											role="menuitemradio"
											aria-checked={acknowledgedFilter ===
												"all"}
											class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "all" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
											onclick={() =>
												setAcknowledgedFilter("all")}
										>
											<span>All Statuses</span>
											{#if acknowledgedFilter === "all"}
												<svg
													width="12"
													height="12"
													viewBox="0 0 24 24"
													fill="none"
													stroke="currentColor"
													stroke-width="3"
													stroke-linecap="round"
													stroke-linejoin="round"
													><polyline
														points="20 6 9 17 4 12"
													/></svg
												>
											{/if}
										</button>
										<button
											type="button"
											role="menuitemradio"
											aria-checked={acknowledgedFilter ===
												"unack"}
											class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "unack" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
											onclick={() =>
												setAcknowledgedFilter("unack")}
										>
											<span>Unread</span>
											{#if acknowledgedFilter === "unack"}
												<svg
													width="12"
													height="12"
													viewBox="0 0 24 24"
													fill="none"
													stroke="currentColor"
													stroke-width="3"
													stroke-linecap="round"
													stroke-linejoin="round"
													><polyline
														points="20 6 9 17 4 12"
													/></svg
												>
											{/if}
										</button>
										<button
											type="button"
											role="menuitemradio"
											aria-checked={acknowledgedFilter ===
												"ack"}
											class={`flex w-full items-center justify-between rounded-[var(--radius-sm)] px-3 py-2 text-left text-[13px] font-medium transition-colors ${acknowledgedFilter === "ack" ? "bg-[var(--accent-soft)] text-[var(--accent-strong)]" : "text-[var(--foreground)] hover:bg-[var(--muted)]/50"}`}
											onclick={() =>
												setAcknowledgedFilter("ack")}
										>
											<span>Read</span>
											{#if acknowledgedFilter === "ack"}
												<svg
													width="12"
													height="12"
													viewBox="0 0 24 24"
													fill="none"
													stroke="currentColor"
													stroke-width="3"
													stroke-linecap="round"
													stroke-linejoin="round"
													><polyline
														points="20 6 9 17 4 12"
													/></svg
												>
											{/if}
										</button>
									</div>
								</div>
							</div>
						{/if}
					</div>
				</div>

				<div
					class="grid max-h-[65vh] gap-4 overflow-y-auto pr-1 custom-scrollbar"
					bind:this={videoListContainer}
					onscroll={updateVideoListBottomState}
					aria-busy={loadingVideos}
				>
					{#if loadingVideos && videos.length === 0}
						{#each Array.from({ length: 3 }) as _, index (index)}
							<article
								class="flex min-h-[14rem] flex-col gap-4 rounded-[var(--radius-md)] p-4 animate-pulse bg-[var(--muted)]/30"
							>
								<div
									class="aspect-video rounded-[var(--radius-sm)] bg-[var(--muted)] opacity-60"
								></div>
								<div
									class="h-4 w-11/12 rounded-full bg-[var(--muted)] opacity-60"
								></div>
								<div
									class="h-3 w-2/5 rounded-full bg-[var(--muted)] opacity-40"
								></div>
							</article>
						{/each}
					{:else if videos.length === 0}
						<p
							class="px-1 text-[14px] font-medium text-[var(--soft-foreground)] opacity-50 italic"
						>
							Waiting for the library to fill.
						</p>
					{:else}
						{#each videos as video}
							<VideoCard
								{video}
								active={selectedVideoId === video.id}
								onSelect={() => selectVideo(video.id)}
							/>
						{/each}
					{/if}
				</div>

				{#if selectedChannelId}
					<div
						class="flex flex-col gap-6 pt-6 border-t border-[var(--border-soft)]/50 mt-4"
					>
						{#if hasMore || !historyExhausted}
							<div class="flex justify-center">
								<button
									type="button"
									class="inline-flex items-center justify-center rounded-full border border-[var(--border-soft)] bg-[var(--background)] px-8 py-3 text-[10px] font-bold uppercase tracking-[0.25em] text-[var(--soft-foreground)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--foreground)] hover:shadow-sm disabled:opacity-30"
									onclick={loadMoreVideos}
									disabled={loadingVideos ||
										backfillingHistory}
								>
									{#if loadingVideos || backfillingHistory}
										Retrieving…
									{:else if hasMore}
										Load More
									{:else}
										Explore History
									{/if}
								</button>
							</div>
						{/if}

						{#if atVideoListBottom && videos.length > 0}
							<div class="space-y-1 px-1">
								<p
									class="text-[9px] font-bold uppercase tracking-[0.3em] text-[var(--soft-foreground)] opacity-40"
								>
									Bottom Reached
								</p>
								<p
									class="text-[12px] font-semibold text-[var(--foreground)]"
								>
									Sync depth: {formatSyncDate(
										resolveDisplayedSyncDepthIso(),
									)}
								</p>
							</div>
						{/if}
					</div>
				{/if}
			</aside>

			<section
				class="flex min-h-[600px] min-w-0 flex-col gap-8 rounded-[var(--radius-lg)] bg-[var(--surface)] border border-[var(--border-soft)] py-10 fade-in stagger-3 shadow-sm lg:sticky lg:top-8 lg:h-[calc(100vh-6rem)] lg:overflow-hidden"
				id="content-view"
			>
				<div
					class="flex flex-wrap items-center justify-between gap-6 px-8 md:px-12 border-b border-[var(--border-soft)]/50"
				>
					<div class="flex items-center gap-6">
						<h2 class="sr-only">Display Content</h2>
						<Toggle
							options={["transcript", "summary", "info"]}
							value={contentMode}
							onChange={(value) =>
								setMode(
									value as "transcript" | "summary" | "info",
								)}
						/>
					</div>

					{#if selectedVideoId && !loadingContent && !editing && contentMode !== "info"}
						<div class="flex items-center justify-end h-10">
							<ContentEditor
								editing={false}
								busy={loadingContent}
								aiAvailable={aiAvailable ?? false}
								formatting={formattingContent &&
									formattingVideoId === selectedVideoId}
								reverting={revertingContent &&
									revertingVideoId === selectedVideoId}
								showFormatAction={contentMode === "transcript"}
								showRevertAction={contentMode === "transcript"}
								canRevert={canRevertTranscript}
								youtubeUrl={contentMode === "transcript"
									? selectedVideoYoutubeUrl
									: null}
								value={draft}
								acknowledged={videos.find(
									(v) => v.id === selectedVideoId,
								)?.acknowledged ?? false}
								onEdit={startEdit}
								onCancel={cancelEdit}
								onSave={saveEdit}
								onFormat={cleanFormatting}
								onRevert={revertToOriginalTranscript}
								onChange={(value) => (draft = value)}
								onAcknowledgeToggle={toggleAcknowledge}
							/>
						</div>
					{/if}
				</div>

				<div
					class="w-full min-h-0 flex-1 overflow-y-auto px-8 md:px-12 custom-scrollbar"
				>
					{#if aiAvailable === false && (contentMode === "transcript" || contentMode === "summary")}
						<div
							class="mb-8 p-6 rounded-[var(--radius-lg)] border border-[var(--accent)]/10 bg-[var(--accent-soft)]/20 flex flex-col gap-3 shadow-sm"
							role="alert"
						>
							<div class="flex items-center gap-3">
								<svg
									width="16"
									height="16"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="3"
									stroke-linecap="round"
									stroke-linejoin="round"
									class="text-[var(--accent)]"
									><circle cx="12" cy="12" r="10" /><line
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
								<p
									class="text-[11px] font-bold tracking-[0.2em] uppercase text-[var(--accent)]"
								>
									Ollama Offline
								</p>
							</div>
							<p
								class="text-[13px] font-medium leading-relaxed text-[var(--accent-strong)] opacity-80"
							>
								Summarization and transcript refining are disabled because the local distillation engine is unreachable from this environment.
							</p>
						</div>
					{/if}
					{#if contentMode === "transcript" && selectedVideoId && ((formattingContent && formattingVideoId === selectedVideoId) || (formattingNotice && formattingNoticeVideoId === selectedVideoId))}
						<div
							class={`mb-8 p-4 rounded-[var(--radius-md)] border flex items-center gap-3 transition-all duration-500 ${
								formattingNoticeTone === "warning"
									? "border-[var(--accent)]/20 bg-[var(--accent-soft)]/50 text-[var(--accent-strong)]"
									: "border-[var(--border-soft)] bg-[var(--muted)]/30 text-[var(--soft-foreground)]"
							}`}
							role="status"
							aria-live="polite"
						>
							{#if formattingContent && formattingVideoId === selectedVideoId}
								<span class="relative flex h-2 w-2">
									<span
										class="animate-ping absolute inline-flex h-full w-full rounded-full bg-current opacity-75"
									></span>
									<span
										class="relative inline-flex rounded-full h-2 w-2 bg-current"
									></span>
								</span>
							{:else}
								<svg
									width="14"
									height="14"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="3"
									stroke-linecap="round"
									stroke-linejoin="round"
									><circle cx="12" cy="12" r="10" /><polyline
										points="12 6 12 12 16 14"
									/></svg
								>
							{/if}
							<p
								class="text-[12px] font-bold tracking-wide uppercase"
							>
								{formattingContent &&
								formattingVideoId === selectedVideoId
									? formattingNotice ||
										"Refining transcript with Ollama…"
									: formattingNotice}
							</p>
						</div>
					{/if}
					{#if contentMode === "summary" && selectedVideoId && !loadingContent}
						<div
							class="mb-8 p-4 rounded-[var(--radius-md)] border border-[var(--border-soft)] bg-[var(--muted)]/20 flex items-center justify-between"
							role="status"
							aria-live="polite"
						>
							<div class="flex items-center gap-3">
								<svg
									width="14"
									height="14"
									viewBox="0 0 24 24"
									fill="none"
									stroke="currentColor"
									stroke-width="3"
									stroke-linecap="round"
									stroke-linejoin="round"
									class="text-[var(--accent)]"
									><polygon
										points="12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2"
									/></svg
								>
								<p
									class="text-[12px] font-bold tracking-[0.1em] uppercase text-[var(--soft-foreground)]"
								>
									{#if summaryQualityScore !== null}
										Quality Analysis: {summaryQualityScore}/10
									{:else}
										Evaluating quality…
									{/if}
								</p>
							</div>
							{#if summaryQualityNote}
								<p
									class="text-[12px] font-medium text-[var(--soft-foreground)] opacity-60 italic"
								>
									"{summaryQualityNote}"
								</p>
							{/if}
						</div>
					{/if}

					{#if !selectedVideoId}
						<div
							class="flex flex-col items-center justify-center h-full text-center opacity-40 py-20"
						>
							<svg
								width="48"
								height="48"
								viewBox="0 0 24 24"
								fill="none"
								stroke="currentColor"
								stroke-width="1"
								stroke-linecap="round"
								stroke-linejoin="round"
								class="mb-6"
								><path
									d="M2 3h6a4 4 0 0 1 4 4v14a3 3 0 0 0-3-3H2z"
								/><path
									d="M22 3h-6a4 4 0 0 0-4 4v14a3 3 0 0 1 3-3h7z"
								/></svg
							>
							<p
								class="text-[20px] font-serif italic text-[var(--soft-foreground)]"
							>
								Select a video to begin distillation.
							</p>
						</div>
					{:else if loadingContent}
						<div
							class="space-y-8 animate-pulse mt-4"
							role="status"
							aria-live="polite"
						>
							<div
								class="h-10 w-3/5 rounded-[var(--radius-sm)] bg-[var(--muted)]/60"
							></div>
							<div class="space-y-4 pt-4">
								<div
									class="h-4 w-full rounded-full bg-[var(--muted)]/50"
								></div>
								<div
									class="h-4 w-11/12 rounded-full bg-[var(--muted)]/50"
								></div>
								<div
									class="h-4 w-10/12 rounded-full bg-[var(--muted)]/50"
								></div>
								<div
									class="h-4 w-full rounded-full bg-[var(--muted)]/50"
								></div>
								<div
									class="h-4 w-3/4 rounded-full bg-[var(--muted)]/50"
								></div>
							</div>
							<p
								class="pt-10 text-[10px] font-bold uppercase tracking-[0.4em] text-[var(--accent)] text-center"
							>
								Processing {contentMode}…
							</p>
						</div>
					{:else if contentMode === "info"}
						<div
							class="space-y-10 text-[15px] leading-relaxed pb-20"
						>
							<div class="space-y-3">
								<p
									class="text-[10px] font-bold uppercase tracking-[0.3em] text-[var(--accent)] opacity-60"
								>
									PRIMARY TITLE
								</p>
								<p
									class="text-[24px] font-bold font-serif leading-tight text-[var(--foreground)]"
								>
									{videoInfo?.title || "Untitled Fragment"}
								</p>
							</div>

							<div
								class="grid gap-10 sm:grid-cols-2 lg:grid-cols-4 border-y border-[var(--border-soft)]/50 py-10"
							>
								<div class="space-y-2">
									<p
										class="text-[10px] font-bold uppercase tracking-[0.2em] text-[var(--soft-foreground)] opacity-50"
									>
										PUBLISHED
									</p>
									<p class="font-bold text-[14px]">
										{formatPublishedAt(
											videoInfo?.published_at,
										)}
									</p>
								</div>
								<div class="space-y-2">
									<p
										class="text-[10px] font-bold uppercase tracking-[0.2em] text-[var(--soft-foreground)] opacity-50"
									>
										ENGAGEMENT
									</p>
									<p class="font-bold text-[14px]">
										{formatCount(videoInfo?.view_count)} Views
									</p>
								</div>
								<div class="space-y-2">
									<p
										class="text-[10px] font-bold uppercase tracking-[0.2em] text-[var(--soft-foreground)] opacity-50"
									>
										TEMPORAL
									</p>
									<p class="font-bold text-[14px]">
										{formatDuration(
											videoInfo?.duration_seconds,
											videoInfo?.duration_iso8601,
										)}
									</p>
								</div>
								<div class="space-y-2">
									<p
										class="text-[10px] font-bold uppercase tracking-[0.2em] text-[var(--soft-foreground)] opacity-50"
									>
										ORIGIN
									</p>
									<p class="font-bold text-[14px] truncate">
										{videoInfo?.channel_name ||
											"Unknown Source"}
									</p>
								</div>
							</div>

							<div class="space-y-3">
								<p
									class="text-[10px] font-bold uppercase tracking-[0.3em] text-[var(--soft-foreground)] opacity-50"
								>
									SOURCE ACCESS
								</p>
								{#if videoInfo?.watch_url}
									<a
										href={videoInfo.watch_url}
										target="_blank"
										rel="noopener noreferrer"
										class="inline-flex items-center gap-2 group text-[14px] font-bold text-[var(--accent)] hover:text-[var(--accent-strong)]"
									>
										<span>{videoInfo.watch_url}</span>
										<svg
											width="14"
											height="14"
											viewBox="0 0 24 24"
											fill="none"
											stroke="currentColor"
											stroke-width="2.5"
											stroke-linecap="round"
											stroke-linejoin="round"
											class="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5"
											><line
												x1="7"
												y1="17"
												x2="17"
												y2="7"
											/><polyline
												points="7 7 17 7 17 17"
											/></svg
										>
									</a>
								{:else}
									<p class="font-bold opacity-40">
										Direct URL unavailable.
									</p>
								{/if}
							</div>

							<div class="space-y-4">
								<p
									class="text-[10px] font-bold uppercase tracking-[0.3em] text-[var(--soft-foreground)] opacity-50"
								>
									FULL DESCRIPTION
								</p>
								<div
									class="p-6 rounded-[var(--radius-md)] bg-[var(--background)] border border-[var(--border-soft)]"
								>
									<p
										class="whitespace-pre-wrap text-[14px] font-medium leading-relaxed text-[var(--foreground)] opacity-80"
									>
										{videoInfo?.description ||
											"Source description is empty or unavailable."}
									</p>
								</div>
							</div>
						</div>
					{:else if editing}
						<div class="pb-20">
							<ContentEditor
								editing
								busy={loadingContent}
								aiAvailable={aiAvailable ?? false}
								formatting={formattingContent &&
									formattingVideoId === selectedVideoId}
								reverting={revertingContent &&
									revertingVideoId === selectedVideoId}
								showFormatAction={contentMode === "transcript"}
								showRevertAction={contentMode === "transcript"}
								canRevert={canRevertTranscript}
								youtubeUrl={contentMode === "transcript"
									? selectedVideoYoutubeUrl
									: null}
								value={draft}
								acknowledged={videos.find(
									(v) => v.id === selectedVideoId,
								)?.acknowledged ?? false}
								onEdit={startEdit}
								onCancel={cancelEdit}
								onSave={saveEdit}
								onFormat={cleanFormatting}
								onRevert={revertToOriginalTranscript}
								onChange={(value) => (draft = value)}
								onAcknowledgeToggle={toggleAcknowledge}
							/>
						</div>
					{:else}
						<div class="pb-32">
							<TranscriptView
								html={contentHtml}
								formatting={contentMode === "transcript" &&
									formattingContent &&
									formattingVideoId === selectedVideoId}
							/>
						</div>
					{/if}
				</div>
			</section>
		</main>
	{/if}

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
