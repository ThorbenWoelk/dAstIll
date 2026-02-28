<script lang="ts">
	import { onMount } from "svelte";
	import {
		addChannel,
		backfillChannelVideos,
		cleanTranscriptFormatting,
		getVideoInfo,
		getSummary,
		getTranscript,
		listChannels,
		listVideos,
		refreshChannel,
		updateSummary,
		updateTranscript,
		updateAcknowledged,
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
		prioritizeChannelOrder,
		resolveInitialChannelSelection,
	} from "$lib/channel-workspace";
	import {
		normalizeTranscriptForRender,
		renderMarkdown,
	} from "$lib/utils/markdown";

	const secondaryButtonClass =
		"inline-flex items-center justify-center rounded-full border border-[var(--border)] px-5 py-3 text-xs font-semibold uppercase tracking-[0.2em] text-[var(--foreground)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent)] disabled:cursor-not-allowed disabled:opacity-60 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]";

	const channelSubmitButtonClass =
		"inline-flex h-9 w-9 shrink-0 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--surface)] text-xl leading-none text-[var(--accent)] transition-colors hover:border-[var(--accent)] hover:text-[var(--accent-strong)] disabled:cursor-not-allowed disabled:opacity-50 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]";
	const WORKSPACE_STATE_KEY = "dastill.workspace.state.v1";

	type AcknowledgedFilter = "all" | "unack" | "ack";

	type WorkspaceStateSnapshot = {
		selectedChannelId: string | null;
		selectedVideoId: string | null;
		contentMode: "transcript" | "summary" | "info";
		videoTypeFilter: VideoTypeFilter;
		hideShorts?: boolean;
		acknowledgedFilter?: AcknowledgedFilter;
		channelOrder?: string[];
	};

	let channels = $state<Channel[]>([]);
	let channelOrder = $state<string[]>([]);
	let videos = $state<Video[]>([]);
	let selectedChannelId = $state<string | null>(null);
	let selectedVideoId = $state<string | null>(null);
	let draggedChannelId = $state<string | null>(null);
	let dragOverChannelId = $state<string | null>(null);

	let channelInput = $state("");
	let loadingChannels = $state(false);
	let loadingVideos = $state(false);
	let loadingContent = $state(false);
	let addingChannel = $state(false);
	let errorMessage = $state<string | null>(null);
	let summaryQualityScore = $state<number | null>(null);
	let summaryQualityNote = $state<string | null>(null);
	let videoInfo = $state<VideoInfoPayload | null>(null);

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
	let formattingRequestSeq = 0;
	let activeFormattingRequest = $state(0);
	let contentRequestSeq = 0;
	let activeContentRequestId = 0;

	let offset = $state(0);
	const limit = 12;
	let hasMore = $state(true);
	let historyExhausted = $state(false);
	let backfillingHistory = $state(false);
	let videoTypeFilter = $state<VideoTypeFilter>("all");
	let acknowledgedFilter = $state<AcknowledgedFilter>("all");
	let workspaceStateHydrated = $state(false);
	let filterMenuOpen = $state(false);
	let filterMenuContainer: HTMLDivElement | null = null;
	let filterMenuLabel = $derived(
		videoTypeFilter === "all"
			? "Open video filter menu."
			: `Video type filter set to ${videoTypeFilter}. Open filter menu.`,
	);

	const selectedChannel = $derived(
		channels.find((channel) => channel.id === selectedChannelId) ?? null,
	);
	const selectedVideoYoutubeUrl = $derived(
		selectedVideoId
			? `https://www.youtube.com/watch?v=${selectedVideoId}`
			: null,
	);
	const selectedOriginalTranscript = $derived(
		selectedVideoId
			? originalTranscriptByVideoId[selectedVideoId] ?? null
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

	function applySavedChannelOrder(nextChannels: Channel[]) {
		if (channelOrder.length === 0) return nextChannels;
		const byId = new Map(nextChannels.map((channel) => [channel.id, channel]));
		const ordered: Channel[] = [];
		const seen = new Set<string>();

		for (const id of channelOrder) {
			const channel = byId.get(id);
			if (!channel) continue;
			ordered.push(channel);
			seen.add(id);
		}

		for (const channel of nextChannels) {
			if (!seen.has(channel.id)) {
				ordered.push(channel);
			}
		}

		return ordered;
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
		void loadChannels();

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

	async function loadChannels(preferredChannelId: string | null = null) {
		loadingChannels = true;
		errorMessage = null;
		let initialChannelId: string | null = null;
		let preferredVideoId: string | null = null;

		try {
			const fetchedChannels = await listChannels();
			channels = applySavedChannelOrder(fetchedChannels);
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
		formattingNotice = null;
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
			return;
		}

		backfillingHistory = true;
		errorMessage = null;

		try {
			const result = await backfillChannelVideos(
				selectedChannelId,
				limit,
			);
			if (result.fetched_count === 0 || result.videos_added === 0) {
				historyExhausted = true;
				return;
			}

			await loadVideos(false);
			if (result.fetched_count < limit && !hasMore) {
				historyExhausted = true;
			}
		} catch (error) {
			errorMessage = (error as Error).message;
		} finally {
			backfillingHistory = false;
		}
	}

	async function selectVideo(videoId: string) {
		selectedVideoId = videoId;
		resetSummaryQuality();
		resetVideoInfo();
		editing = false;
		formattingNotice = null;
		formattingNoticeVideoId = null;
		await loadContent();
	}

	async function setMode(mode: "transcript" | "summary" | "info") {
		if (contentMode === mode) return;
		contentMode = mode;
		resetSummaryQuality();
		resetVideoInfo();
		editing = false;
		formattingNotice = null;
		formattingNoticeVideoId = null;
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
		formattingNotice = "Formatting transcript with Ollama…";
		formattingNoticeVideoId = targetVideoId;
		formattingNoticeTone = "info";

		try {
			const result = await cleanTranscriptFormatting(
				targetVideoId,
				source,
			);
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
						? "No formatting changes."
						: "Formatting applied to draft. Save to persist.";
				formattingNoticeVideoId = targetVideoId;
			} else {
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
				formattingNotice =
					result.content === source
						? "No formatting changes."
						: "Formatting applied and saved.";
				formattingNoticeVideoId = targetVideoId;
			}
			formattingNoticeTone = "success";
			if (!result.preserved_text) {
				errorMessage =
					"Formatting changed transcript words. Original transcript text was kept.";
				formattingNotice =
					"Safety guard kept original wording. Only spacing changes are allowed.";
				formattingNoticeVideoId = targetVideoId;
				formattingNoticeTone = "warning";
			}
		} catch (error) {
			errorMessage = (error as Error).message;
			formattingNotice = null;
			formattingNoticeVideoId = null;
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
				const transcript = await updateTranscript(targetVideoId, original);
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
			formattingNotice = null;
			formattingNoticeVideoId = null;
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
				formattingNotice = null;
				formattingNoticeVideoId = null;
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

	async function refreshSummaryQuality() {
		if (!selectedVideoId || contentMode !== "summary" || editing || loadingContent)
			return;
		const targetVideoId = selectedVideoId;
		try {
			const summary = await getSummary(targetVideoId);
			if (selectedVideoId !== targetVideoId || contentMode !== "summary" || editing)
				return;
			applySummaryQuality(summary);
		} catch {
			// Keep previous quality state if background refresh fails.
		}
	}

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
</script>

<svelte:window onkeydown={handleWindowKeydown} />

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
					class="text-xs font-semibold uppercase tracking-[0.35em] text-[var(--accent)]"
				>
					DASTILL v1.0
				</p>
				<h1
					class="text-balance text-3xl font-bold tracking-tight sm:text-4xl text-[var(--foreground)]"
				>
					Follow Channels &amp; Distill Knowledge
				</h1>
				<p
					class="max-w-2xl font-serif text-[17px] text-[var(--soft-foreground)]"
				>
					Track new uploads, extract transcripts, and refine summaries
					from one focused workspace.
				</p>
			</div>
		</div>

		<nav
			class="flex flex-wrap items-center gap-2 rounded-3xl border border-[var(--border)] bg-[var(--surface)] p-1.5 px-2"
			aria-label="Workspace sections"
		>
			<a
				href="#workspace"
				class="rounded-full bg-[var(--muted)]/50 px-4 py-2 text-xs font-semibold uppercase tracking-[0.15em] text-[var(--foreground)] transition-colors hover:text-[var(--accent)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
			>
				Workspace
			</a>
			<a
				href="/download-queue"
				class="rounded-full px-4 py-2 text-xs font-semibold uppercase tracking-[0.15em] text-[var(--soft-foreground)] transition-colors hover:bg-[var(--muted)]/30 hover:text-[var(--foreground)] focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
			>
				Download Queue
			</a>
		</nav>
	</header>

	<main
		id="main-content"
		class="mx-auto mt-6 grid w-full max-w-[1440px] items-start gap-8 lg:grid-cols-[280px_320px_minmax(0,1fr)] xl:grid-cols-[280px_360px_minmax(0,1fr)]"
	>
		<aside
			class="flex h-fit flex-col gap-4 rounded-3xl bg-[var(--surface)] border border-[var(--border)] p-4 lg:sticky lg:top-6"
			id="workspace"
		>
			<div class="flex items-center justify-between gap-2 px-1">
				<h2 class="text-lg font-semibold tracking-tight">Channels</h2>
			</div>

			<form
				class="grid gap-3"
				onsubmit={handleChannelSubmit}
				aria-label="Follow channel"
			>
				<div
					class="flex min-w-0 items-center gap-2 rounded-full border border-[var(--border)] bg-[var(--background)] pl-4 pr-1 transition-shadow focus-within:ring-2 focus-within:ring-[var(--accent)] focus-within:ring-offset-2 focus-within:ring-offset-[var(--surface)]"
				>
					<label for="channel-input" class="sr-only"
						>Add Channel</label
					>
					<input
						id="channel-input"
						name="channel"
						autocomplete="off"
						spellcheck={false}
						class="min-w-0 flex-1 bg-transparent py-2.5 text-sm placeholder:text-[var(--soft-foreground)] focus-visible:outline-none"
						placeholder="Handle, URL..."
						bind:value={channelInput}
					/>
					<button
						type="submit"
						class={channelSubmitButtonClass}
						disabled={!channelInput.trim() || addingChannel}
						aria-label="Follow channel"
					>
						+
					</button>
				</div>
			</form>

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
						Add a channel to get started.
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
							onDrop={(event) => handleChannelDrop(channel.id, event)}
							onDragEnd={handleChannelDragEnd}
						/>
					{/each}
				{/if}
			</div>
		</aside>

		<aside
			class="flex h-fit min-w-0 flex-col gap-5 rounded-3xl bg-[var(--surface)] border border-[var(--border)] p-4 lg:sticky lg:top-6"
			id="videos"
		>
			<div class="flex flex-wrap items-end justify-between gap-3 px-1">
				<div class="min-w-0">
					<h2 class="text-lg font-semibold tracking-tight">
						Latest Videos
						{#if refreshingChannel}
							<span
								class="ml-2 text-[10px] uppercase tracking-widest text-[var(--soft-foreground)] animate-pulse inline-flex items-center align-middle rounded-full border border-[var(--border)] px-2 py-0.5"
								>Syncing</span
							>
						{/if}
					</h2>
				</div>
				<div class="relative" bind:this={filterMenuContainer}>
					<button
						type="button"
						class={`group flex items-center justify-center rounded-full p-2 transition-all duration-200 ${videoTypeFilter !== "all" || filterMenuOpen ? "bg-[var(--accent)] text-white shadow-sm" : "text-[var(--soft-foreground)] hover:bg-white/60 hover:text-[var(--foreground)] border border-transparent hover:border-[var(--border)]"} focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] disabled:opacity-50 disabled:cursor-not-allowed`}
						onclick={toggleFilterMenu}
						disabled={!selectedChannelId || loadingVideos}
						aria-label={filterMenuLabel}
						aria-haspopup="menu"
						aria-expanded={filterMenuOpen}
						aria-controls="video-filter-menu"
					>
						<svg
							width="18"
							height="18"
							viewBox="0 0 24 24"
							fill="none"
							stroke="currentColor"
							stroke-width="2"
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
							class="absolute right-0 top-full z-20 mt-2 w-56 rounded-2xl border border-[var(--border)] bg-[var(--surface)] p-3 shadow-xl"
						>
							<p
								class="text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--soft-foreground)]"
							>
								Video Type
							</p>
							<div class="mt-2 grid gap-1">
								<button
									type="button"
									role="menuitemradio"
									aria-checked={videoTypeFilter === "all"}
									class={`flex w-full items-center justify-between rounded-xl px-3 py-2 text-left text-sm transition-colors ${videoTypeFilter === "all" ? "bg-[var(--muted)] text-[var(--foreground)]" : "text-[var(--soft-foreground)] hover:bg-[var(--muted)]/50 hover:text-[var(--foreground)]"}`}
									onclick={() => setVideoTypeFilter("all")}
								>
									<span>All videos</span>
									{#if videoTypeFilter === "all"}
										<span aria-hidden="true">•</span>
									{/if}
								</button>
								<button
									type="button"
									role="menuitemradio"
									aria-checked={videoTypeFilter === "long"}
									class={`flex w-full items-center justify-between rounded-xl px-3 py-2 text-left text-sm transition-colors ${videoTypeFilter === "long" ? "bg-[var(--muted)] text-[var(--foreground)]" : "text-[var(--soft-foreground)] hover:bg-[var(--muted)]/50 hover:text-[var(--foreground)]"}`}
									onclick={() => setVideoTypeFilter("long")}
								>
									<span>Long videos only</span>
									{#if videoTypeFilter === "long"}
										<span aria-hidden="true">•</span>
									{/if}
								</button>
								<button
									type="button"
									role="menuitemradio"
									aria-checked={videoTypeFilter === "short"}
									class={`flex w-full items-center justify-between rounded-xl px-3 py-2 text-left text-sm transition-colors ${videoTypeFilter === "short" ? "bg-[var(--muted)] text-[var(--foreground)]" : "text-[var(--soft-foreground)] hover:bg-[var(--muted)]/50 hover:text-[var(--foreground)]"}`}
									onclick={() => setVideoTypeFilter("short")}
								>
									<span>Shorts only</span>
									{#if videoTypeFilter === "short"}
										<span aria-hidden="true">•</span>
									{/if}
								</button>
							</div>

							<p
								class="text-[10px] font-semibold uppercase tracking-[0.2em] text-[var(--soft-foreground)] mt-4 mb-2"
							>
								Status
							</p>
							<div class="grid gap-1">
								<button
									type="button"
									role="menuitemradio"
									aria-checked={acknowledgedFilter === "all"}
									class={`flex w-full items-center justify-between rounded-xl px-3 py-2 text-left text-sm transition-colors ${acknowledgedFilter === "all" ? "bg-[var(--muted)] text-[var(--foreground)]" : "text-[var(--soft-foreground)] hover:bg-[var(--muted)]/50 hover:text-[var(--foreground)]"}`}
									onclick={() => setAcknowledgedFilter("all")}
								>
									<span>All statuses</span>
									{#if acknowledgedFilter === "all"}
										<span aria-hidden="true">•</span>
									{/if}
								</button>
								<button
									type="button"
									role="menuitemradio"
									aria-checked={acknowledgedFilter ===
										"unack"}
									class={`flex w-full items-center justify-between rounded-xl px-3 py-2 text-left text-sm transition-colors ${acknowledgedFilter === "unack" ? "bg-[var(--muted)] text-[var(--foreground)]" : "text-[var(--soft-foreground)] hover:bg-[var(--muted)]/50 hover:text-[var(--foreground)]"}`}
									onclick={() =>
										setAcknowledgedFilter("unack")}
								>
									<span>Unacknowledged</span>
									{#if acknowledgedFilter === "unack"}
										<span aria-hidden="true">•</span>
									{/if}
								</button>
								<button
									type="button"
									role="menuitemradio"
									aria-checked={acknowledgedFilter === "ack"}
									class={`flex w-full items-center justify-between rounded-xl px-3 py-2 text-left text-sm transition-colors ${acknowledgedFilter === "ack" ? "bg-[var(--muted)] text-[var(--foreground)]" : "text-[var(--soft-foreground)] hover:bg-[var(--muted)]/50 hover:text-[var(--foreground)]"}`}
									onclick={() => setAcknowledgedFilter("ack")}
								>
									<span>Acknowledged</span>
									{#if acknowledgedFilter === "ack"}
										<span aria-hidden="true">•</span>
									{/if}
								</button>
							</div>
						</div>
					{/if}
				</div>
			</div>

			<div
				class="grid max-h-[70vh] gap-3 overflow-y-auto pr-1"
				aria-busy={loadingVideos}
			>
				{#if loadingVideos && videos.length === 0}
					{#each Array.from({ length: 4 }) as _, index (index)}
						<article
							class="flex min-h-[12rem] flex-col gap-3 rounded-2xl p-3 animate-pulse"
						>
							<div
								class="aspect-video rounded-xl bg-[var(--muted)]"
							></div>
							<div
								class="h-3.5 w-11/12 rounded-full bg-[var(--muted)] mt-1"
							></div>
							<div
								class="h-2.5 w-2/5 rounded-full bg-[var(--muted)]/80"
							></div>
						</article>
					{/each}
				{:else if videos.length === 0}
					<p class="px-1 text-sm text-[var(--soft-foreground)]">
						No videos yet for this channel.
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

			{#if selectedChannelId && (hasMore || !historyExhausted)}
				<div class="flex justify-center pt-2">
					<button
						type="button"
						class={secondaryButtonClass}
						onclick={loadMoreVideos}
						disabled={loadingVideos || backfillingHistory}
					>
						{#if loadingVideos || backfillingHistory}
							Loading…
						{:else if hasMore}
							Load More
						{:else}
							Load Older
						{/if}
					</button>
				</div>
			{/if}
		</aside>

		<section
			class="flex min-h-0 min-w-0 flex-col gap-6 rounded-3xl bg-[var(--surface)] border border-[var(--border)] py-6 md:py-10 font-serif lg:sticky lg:top-6 lg:h-[calc(100vh-3rem)] lg:overflow-hidden"
			id="content-view"
		>
			<div
				class="flex flex-wrap items-center justify-between gap-4 pb-2 px-6 md:px-10"
			>
				<div class="flex items-center gap-4">
					<h2 class="sr-only">Content Mode</h2>
					<Toggle
						options={["transcript", "summary", "info"]}
						value={contentMode}
						onChange={(value) =>
							setMode(value as "transcript" | "summary" | "info")}
					/>
				</div>

				{#if selectedVideoId &&
					!loadingContent &&
					!editing &&
					contentMode !== "info"}
					<div class="flex justify-end font-sans">
						<ContentEditor
							editing={false}
							busy={loadingContent}
							formatting={formattingContent &&
								formattingVideoId === selectedVideoId}
							reverting={revertingContent &&
								revertingVideoId === selectedVideoId}
							showFormatAction={contentMode === "transcript"}
							showRevertAction={contentMode === "transcript"}
							canRevert={canRevertTranscript}
							youtubeUrl={
								contentMode === "transcript"
									? selectedVideoYoutubeUrl
									: null
							}
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

				<div class="w-full min-h-0 flex-1 overflow-y-auto px-6 md:px-10">
					{#if contentMode === "transcript" && selectedVideoId && ((formattingContent && formattingVideoId === selectedVideoId) || (formattingNotice && formattingNoticeVideoId === selectedVideoId))}
						<p
						class={`mb-3 font-sans text-xs ${
							formattingNoticeTone === "warning"
								? "text-[var(--accent)]"
								: "text-[var(--soft-foreground)]"
						}`}
						role="status"
						aria-live="polite"
					>
							{formattingContent &&
							formattingVideoId === selectedVideoId
								? "Formatting transcript with Ollama…"
								: formattingNotice}
						</p>
					{/if}
					{#if contentMode === "summary" && selectedVideoId && !loadingContent}
						<p
							class="mb-3 font-sans text-[11px] text-[var(--soft-foreground)]"
							role="status"
							aria-live="polite"
						>
							{#if summaryQualityScore !== null}
								Quality {summaryQualityScore}/10
								{#if summaryQualityNote}
									- {summaryQualityNote}
								{/if}
							{:else if summaryQualityNote}
								{summaryQualityNote}
							{:else}
								Quality check in progress…
							{/if}
						</p>
					{/if}
					{#if !selectedVideoId}
						<p
							class="text-[var(--soft-foreground)] font-sans text-base"
						>
						Select a video to start reading.
					</p>
				{:else if loadingContent}
					<div
						class="space-y-4 animate-pulse mt-4"
						role="status"
						aria-live="polite"
					>
						<div class="h-6 w-2/5 rounded bg-[var(--muted)]"></div>
						<div
							class="h-4 w-full rounded bg-[var(--muted)]/85 mt-6"
						></div>
						<div
							class="h-4 w-11/12 rounded bg-[var(--muted)]/85"
						></div>
						<div
							class="h-4 w-10/12 rounded bg-[var(--muted)]/85"
						></div>
						<div
							class="h-4 w-3/4 rounded bg-[var(--muted)]/85"
						></div>
						<p
							class="pt-4 font-sans text-xs uppercase tracking-[0.2em] text-[var(--soft-foreground)]"
						>
							Preparing {contentMode}…
						</p>
					</div>
				{:else if contentMode === "info"}
					<div class="font-sans space-y-5 text-sm leading-relaxed">
						<div class="space-y-1">
							<p class="text-xs uppercase tracking-[0.15em] text-[var(--soft-foreground)]">
								Title
							</p>
							<p class="text-[var(--foreground)]">
								{videoInfo?.title || "Unknown"}
							</p>
						</div>

						<div class="grid gap-4 sm:grid-cols-2">
							<div class="space-y-1">
								<p class="text-xs uppercase tracking-[0.15em] text-[var(--soft-foreground)]">
									Published
								</p>
								<p>{formatPublishedAt(videoInfo?.published_at)}</p>
							</div>
							<div class="space-y-1">
								<p class="text-xs uppercase tracking-[0.15em] text-[var(--soft-foreground)]">
									Views
								</p>
								<p>{formatCount(videoInfo?.view_count)}</p>
							</div>
							<div class="space-y-1">
								<p class="text-xs uppercase tracking-[0.15em] text-[var(--soft-foreground)]">
									Duration
								</p>
								<p>
									{formatDuration(
										videoInfo?.duration_seconds,
										videoInfo?.duration_iso8601,
									)}
								</p>
							</div>
							<div class="space-y-1">
								<p class="text-xs uppercase tracking-[0.15em] text-[var(--soft-foreground)]">
									Channel
								</p>
								<p>{videoInfo?.channel_name || videoInfo?.channel_id || "Unknown"}</p>
							</div>
						</div>

						<div class="space-y-1">
							<p class="text-xs uppercase tracking-[0.15em] text-[var(--soft-foreground)]">
								Video URL
							</p>
							{#if videoInfo?.watch_url}
								<a
									href={videoInfo.watch_url}
									target="_blank"
									rel="noopener noreferrer"
									class="break-all text-[var(--accent)] underline underline-offset-4 hover:text-[var(--accent-strong)]"
								>
									{videoInfo.watch_url}
								</a>
							{:else}
								<p>Unknown</p>
							{/if}
						</div>

						<div class="space-y-1">
							<p class="text-xs uppercase tracking-[0.15em] text-[var(--soft-foreground)]">
								Description
							</p>
							<p class="whitespace-pre-wrap text-[var(--foreground)]">
								{videoInfo?.description || "Description unavailable."}
							</p>
						</div>
					</div>
				{:else if editing}
					<div class="font-sans">
						<ContentEditor
							editing
							busy={loadingContent}
							formatting={formattingContent &&
								formattingVideoId === selectedVideoId}
							reverting={revertingContent &&
								revertingVideoId === selectedVideoId}
							showFormatAction={contentMode === "transcript"}
							showRevertAction={contentMode === "transcript"}
							canRevert={canRevertTranscript}
							youtubeUrl={
								contentMode === "transcript"
									? selectedVideoYoutubeUrl
									: null
							}
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
					<TranscriptView
						html={contentHtml}
						formatting={contentMode === "transcript" &&
							formattingContent &&
							formattingVideoId === selectedVideoId}
					/>
				{/if}
			</div>
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
