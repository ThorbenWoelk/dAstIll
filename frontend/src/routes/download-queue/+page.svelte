<script lang="ts">
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import {
    getChannelSnapshot,
    getChannelSyncDepth,
    getWorkspaceBootstrapWhenAvailable,
    isAiAvailable,
    listVideos,
    refreshChannel,
    updateChannel,
  } from "$lib/api";
  import { resolveAiIndicatorPresentation } from "$lib/ai-status";
  import AiStatusIndicator from "$lib/components/AiStatusIndicator.svelte";
  import {
    applySavedChannelOrder,
    resolveInitialChannelSelection,
    WORKSPACE_STATE_KEY,
    type WorkspaceStateSnapshot,
  } from "$lib/channel-workspace";
  import defaultChannelIcon from "$lib/assets/channel-default.svg";
  import ChannelCard from "$lib/components/ChannelCard.svelte";
  import ConfirmationModal from "$lib/components/ConfirmationModal.svelte";
  import type {
    AiStatus,
    Channel,
    ChannelSnapshot,
    ContentStatus,
    Video,
  } from "$lib/types";

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
  let aiStatus = $state<AiStatus | null>(null);
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
  let aiIndicator = $derived(
    aiStatus ? resolveAiIndicatorPresentation(aiStatus) : null,
  );

  const MAX_RETRIES = 3;
  const CHANNEL_REFRESH_TTL_MS = 5 * 60 * 1000;
  const channelLastRefreshedAt = new Map<string, number>();

  $effect(() => {
    const timer = setInterval(() => {
      void isAiAvailable()
        .then((status) => {
          aiAvailable = status.available;
          aiStatus = status.status;
        })
        .catch(() => {
          aiAvailable = false;
          aiStatus = "offline";
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
      : (syncDepth?.derived_earliest_ready_date ??
          selectedChannel?.earliest_sync_date),
  );

  const queuedVideos = $derived(
    videos.filter(
      (video) =>
        video.transcript_status !== "ready" || video.summary_status !== "ready",
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
          detail: "Transcript extraction is running now.",
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
            : "Transcript extraction failed. Automatic retry is queued.",
        };
      }

      return {
        kind: "queued",
        label: "DISTILLING KNOWLEDGE - QUEUED FOR TRANSCRIPT",
        detail: "Waiting in queue to start transcript extraction.",
      };
    }

    if (video.summary_status === "loading") {
      return {
        kind: "processing",
        label: "DISTILLING KNOWLEDGE - PROCESSING SUMMARY",
        detail: "Summary generation is running now.",
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
          : "Summary generation failed. Automatic retry is queued.",
      };
    }

    return {
      kind: "queued",
      label: "DISTILLING KNOWLEDGE - QUEUED FOR SUMMARY",
      detail: "Transcript is ready. Waiting in queue for summary generation.",
    };
  }

  const queueStats = $derived({
    total: queuedVideos.length,
    loading: queuedVideos.filter((video) => getQueueState(video) === "loading")
      .length,
    pending: queuedVideos.filter((video) => getQueueState(video) === "pending")
      .length,
    failed: queuedVideos.filter((video) => getQueueState(video) === "failed")
      .length,
  });
  const queuedVideosWithDistillationStatus = $derived(
    queuedVideos.map((video) => ({
      video,
      distillationStatus: getDistillationStatusCopy(video),
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
        video.transcript_status !== "ready" || video.summary_status !== "ready",
    ).length;

    if (previousQueuedTotal === null) {
      queueDeltaSinceLastSync = null;
    } else {
      queueDeltaSinceLastSync = queuedCount - previousQueuedTotal;
    }

    previousQueuedTotal = queuedCount;
    lastSyncedAt = new Date();
  }

  async function applyChannelSnapshot(
    channelId: string,
    snapshot: ChannelSnapshot,
    silent = false,
  ) {
    if (!silent) {
      loadingVideos = true;
      errorMessage = null;
    }

    try {
      if (selectedChannelId !== channelId) {
        return;
      }

      syncDepth = snapshot.sync_depth;
      videos = snapshot.videos;
      offset = snapshot.videos.length;
      hasMore = snapshot.videos.length === limit;
      setSyncSnapshot(snapshot.videos);
    } catch (error) {
      if (!silent || !errorMessage) {
        errorMessage = (error as Error).message;
      }
    } finally {
      if (!silent) {
        loadingVideos = false;
      }
    }
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
    if (!workspaceStateHydrated || typeof localStorage === "undefined") return;

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
    const effective = selectedChannel.earliest_sync_date_user_set
      ? selectedChannel.earliest_sync_date
      : (syncDepth?.derived_earliest_ready_date ??
        selectedChannel.earliest_sync_date);
    if (effective) {
      earliestSyncDateInput = new Date(effective).toISOString().split("T")[0];
    } else {
      earliestSyncDateInput = "";
    }
  });

  $effect(() => {
    if (!selectedChannelId) {
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

    try {
      const bootstrap = retryUntilBackendReachable
        ? await getWorkspaceBootstrapWhenAvailable({
            selectedChannelId,
            limit,
            offset: 0,
            queueOnly: true,
          })
        : await getWorkspaceBootstrapWhenAvailable({
            selectedChannelId,
            limit,
            offset: 0,
            queueOnly: true,
            retryDelayMs: 0,
          });

      aiAvailable = bootstrap.ai_available;
      aiStatus = bootstrap.ai_status;
      waitingForBackend = false;
      channels = applySavedChannelOrder(bootstrap.channels, channelOrder);
      channelOrder = channels.map((c) => c.id);

      const initialChannelId = resolveInitialChannelSelection(
        channels,
        selectedChannelId,
        bootstrap.selected_channel_id,
      );
      if (!initialChannelId) {
        selectedChannelId = null;
        videos = [];
        syncDepth = null;
      } else {
        selectedChannelId = initialChannelId;
        if (
          bootstrap.snapshot &&
          bootstrap.snapshot.channel_id === initialChannelId
        ) {
          await applyChannelSnapshot(initialChannelId, bootstrap.snapshot);
        }
      }
    } catch (error) {
      waitingForBackend = false;
      errorMessage = (error as Error).message;
    } finally {
      loadingChannels = false;
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
    const snapshot = await getChannelSnapshot(channelId, {
      limit,
      offset: 0,
      queueOnly: true,
    });
    await applyChannelSnapshot(channelId, snapshot);

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
      // After refresh, silently reload the queue snapshot.
      if (selectedChannelId === channelId) {
        const refreshedSnapshot = await getChannelSnapshot(channelId, {
          limit,
          offset: 0,
          queueOnly: true,
        });
        await applyChannelSnapshot(channelId, refreshedSnapshot, true);
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
      if (!silent || !errorMessage) errorMessage = (error as Error).message;
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

  <header
    class="mx-auto flex w-full max-w-[1440px] items-center justify-between gap-3 px-4 sm:px-2 pb-2 mb-0"
  >
    <div class="flex items-center gap-3">
      <a
        href="/"
        class="text-xl sm:text-2xl font-bold tracking-tighter text-[var(--foreground)] transition-opacity hover:opacity-80 focus-visible:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)] focus-visible:ring-offset-2 focus-visible:ring-offset-[var(--surface)]"
        aria-label="Go to dAstIll home"
      >
        DASTILL
      </a>
      {#if aiIndicator}
        <AiStatusIndicator
          detail={aiIndicator.detail}
          dotClass={aiIndicator.dotClass}
          title={aiIndicator.title}
        />
      {/if}
    </div>

    <nav class="flex items-center gap-0.5" aria-label="Workspace sections">
      <a
        href="/"
        class="rounded-full px-3.5 py-1.5 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50 transition-all hover:opacity-100"
      >
        Workspace
      </a>
      <a
        href="/download-queue"
        class="rounded-full px-3.5 py-1.5 text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--foreground)] bg-[var(--muted)] transition-all"
      >
        Queue
      </a>
    </nav>
  </header>

  {#if waitingForBackend && channels.length === 0}
    <main
      id="main-content"
      class="mx-auto flex min-h-[60vh] w-full max-w-[1440px] flex-col items-center justify-center text-center fade-in px-6"
      role="status"
      aria-live="polite"
    >
      <div
        class="mb-6 h-6 w-6 animate-spin rounded-full border-2 border-[var(--muted)] border-t-[var(--accent)]"
      ></div>
      <p
        class="text-[11px] font-bold uppercase tracking-[0.25em] text-[var(--accent)]"
      >
        Connecting
      </p>
      <p
        class="mt-2 max-w-[260px] text-[14px] font-medium text-[var(--soft-foreground)] opacity-60"
      >
        Waiting for the distillation engine.
      </p>
    </main>
  {:else}
    <main
      id="main-content"
      class="mx-auto mt-4 grid w-full max-w-[1440px] gap-0 lg:grid-cols-[260px_minmax(0,1fr)] xl:grid-cols-[280px_minmax(0,1fr)] items-start max-lg:mt-0"
    >
      <aside
        class="flex h-fit flex-col gap-4 border-0 lg:pr-5 lg:border-r lg:border-[var(--border-soft)] lg:pl-2 lg:sticky lg:top-4 fade-in stagger-1 {mobileTab !==
        'channels'
          ? 'hidden lg:flex'
          : 'h-[calc(100dvh-4rem)] p-4 gap-4'}"
      >
        <div class="flex items-center justify-between gap-2">
          <h2
            class="text-base font-bold tracking-tight text-[var(--soft-foreground)]"
          >
            Channels
          </h2>
          <button
            type="button"
            class="inline-flex h-7 w-7 items-center justify-center rounded-full transition-colors {manageChannels
              ? 'text-red-500'
              : 'text-[var(--soft-foreground)] opacity-40 hover:opacity-80'}"
            data-tooltip={manageChannels
              ? "Exit manage mode"
              : "Manage channels"}
            onclick={() => {
              manageChannels = !manageChannels;
            }}
            aria-label={manageChannels ? "Exit manage mode" : "Manage channels"}
          >
            <svg
              width="13"
              height="13"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              ><path d="M3 6h18"></path><path
                d="M19 6v14c0 1-1 2-2 2H7c-1 0-2-1-2-2V6"
              ></path><path d="M8 6V4c0-1 1-2 2-2h4c1 0 2 1 2 2v2"></path></svg
            >
          </button>
        </div>

        <div
          class="flex flex-1 min-h-0 flex-col gap-1.5 overflow-y-auto pr-1 custom-scrollbar lg:max-h-[60vh]"
          aria-busy={loadingChannels}
        >
          {#if loadingChannels}
            <div class="space-y-4" role="status" aria-live="polite">
              {#each Array.from({ length: 4 }) as _, index (index)}
                <div class="flex animate-pulse items-center gap-4 px-3 py-3">
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
                onDragOver={(event) => handleChannelDragOver(channel.id, event)}
                onDrop={(event) => handleChannelDrop(channel.id, event)}
                onDragEnd={handleChannelDragEnd}
                onDelete={() => handleDeleteChannel(channel.id)}
              />
            {/each}
          {/if}
        </div>
      </aside>

      <section
        class="flex min-w-0 flex-col gap-6 overflow-hidden border-0 lg:pl-6 fade-in stagger-2 {mobileTab !==
        'details'
          ? 'hidden lg:flex'
          : 'h-[calc(100dvh-4rem)] p-4 pt-4'}"
      >
        <div
          class="flex flex-wrap items-center justify-between gap-4 pb-4 border-b border-[var(--border-soft)]"
        >
          <div class="flex items-center gap-3 min-w-0">
            <button
              onclick={() => (mobileTab = "channels")}
              class="lg:hidden inline-flex items-center gap-2 text-[13px] font-semibold text-[var(--foreground)] transition-transform active:scale-95"
            >
              <div
                class="h-6 w-6 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
              >
                <img
                  src={selectedChannel?.thumbnail_url || defaultChannelIcon}
                  alt=""
                  class="h-full w-full object-cover"
                />
              </div>
              <span class="truncate"
                >{selectedChannel ? selectedChannel.name : "None"}</span
              >
              <svg
                class="h-3 w-3 opacity-30 shrink-0"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="3"
                stroke-linecap="round"
                stroke-linejoin="round"><path d="m6 9 6 6 6-6" /></svg
              >
            </button>
            <span
              class="max-lg:hidden flex items-center gap-2 text-[13px] font-semibold text-[var(--foreground)]"
            >
              <div
                class="h-6 w-6 shrink-0 overflow-hidden rounded-full bg-[var(--muted)]"
              >
                <img
                  src={selectedChannel?.thumbnail_url || defaultChannelIcon}
                  alt=""
                  class="h-full w-full object-cover"
                />
              </div>
              {selectedChannel ? selectedChannel.name : "None"}
            </span>
            <span
              class="hidden sm:block text-[11px] text-[var(--soft-foreground)] opacity-40"
            >
              {#if lastSyncedAt}
                {syncTimeFormatter.format(lastSyncedAt)}
              {/if}
            </span>
          </div>
          <div
            class="flex items-center gap-4 text-[11px] font-bold tabular-nums"
          >
            <span class="text-slate-500" data-tooltip="Queued"
              >{queueStats.pending} queued</span
            >
            {#if queueStats.loading > 0}
              <span class="text-amber-600 flex items-center gap-1.5">
                <span
                  class="h-1.5 w-1.5 rounded-full bg-amber-500 animate-pulse"
                ></span>
                {queueStats.loading} active
              </span>
            {/if}
            {#if queueStats.failed > 0}
              <span class="text-rose-600">{queueStats.failed} failed</span>
            {/if}
          </div>
        </div>

        {#if selectedChannel}
          <div class="flex flex-wrap items-center gap-4 py-2">
            <div class="flex items-center gap-2 min-w-0">
              <p
                class="text-[11px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-50"
              >
                Sync from
              </p>
              <p class="text-[13px] font-semibold text-[var(--foreground)]">
                {formatSyncDate(effectiveEarliestSyncDate)}
              </p>
            </div>
            <div class="flex items-center gap-2 ml-auto">
              <input
                type="date"
                class="rounded-[var(--radius-sm)] border border-[var(--border-soft)] bg-white px-2.5 py-1.5 text-[12px] font-medium focus:outline-none focus:border-[var(--accent)]/40 transition-colors"
                bind:value={earliestSyncDateInput}
                disabled={savingSyncDate}
              />
              <button
                type="button"
                class="rounded-[var(--radius-sm)] bg-[var(--foreground)] px-3 py-1.5 text-[10px] font-bold uppercase tracking-[0.08em] text-white transition-all hover:bg-[var(--accent-strong)] disabled:opacity-30"
                onclick={saveEarliestSyncDate}
                disabled={!earliestSyncDateInput || savingSyncDate}
              >
                {savingSyncDate ? "..." : "Set"}
              </button>
            </div>
          </div>
        {/if}

        <div class="flex-1 overflow-y-auto custom-scrollbar">
          {#if !selectedChannelId}
            <div
              class="flex flex-col items-center justify-center py-20 text-center"
            >
              <p class="text-[15px] text-[var(--soft-foreground)] opacity-30">
                Select a channel to view its queue.
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
            <div
              class="flex flex-col items-center justify-center py-20 text-center"
            >
              <svg
                width="20"
                height="20"
                viewBox="0 0 24 24"
                fill="none"
                stroke="currentColor"
                stroke-width="2.5"
                stroke-linecap="round"
                stroke-linejoin="round"
                class="text-emerald-500 mb-3"
                ><polyline points="20 6 9 17 4 12" /></svg
              >
              <p class="text-[14px] text-[var(--soft-foreground)] opacity-50">
                Queue is clear.
              </p>
            </div>
          {:else}
            <ul class="flex flex-col mt-2 pb-20">
              {#each queuedVideosWithDistillationStatus as item}
                {@const video = item.video}
                <li
                  class="border-b border-[var(--border-soft)] last:border-b-0"
                >
                  <button
                    type="button"
                    class="group w-full cursor-pointer py-4 px-1 text-left focus:outline-none focus-visible:ring-2 focus-visible:ring-[var(--accent)]/40 max-lg:py-3"
                    onclick={() => openVideoTranscriptInWorkspace(video)}
                    aria-label={`Open transcript workspace for ${video.title}`}
                  >
                    <p
                      class="line-clamp-2 text-[14px] font-semibold text-[var(--foreground)] leading-[1.4] tracking-tight group-hover:text-[var(--accent-strong)] transition-colors"
                    >
                      {video.title}
                    </p>
                    <div class="mt-2 flex flex-wrap items-center gap-3">
                      <span
                        class="text-[11px] font-medium text-[var(--soft-foreground)] opacity-50"
                      >
                        {formatDate(video.published_at)}
                      </span>
                      {#if item.distillationStatus.kind === "processing"}
                        <span
                          class="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--accent)]"
                        >
                          <span class="relative flex h-1.5 w-1.5">
                            <span
                              class="animate-ping absolute inline-flex h-full w-full rounded-full bg-[var(--accent)] opacity-75"
                            ></span>
                            <span
                              class="relative inline-flex rounded-full h-1.5 w-1.5 bg-[var(--accent)]"
                            ></span>
                          </span>
                          Processing
                        </span>
                      {:else if item.distillationStatus.kind === "failed"}
                        <span
                          class="inline-flex items-center gap-1.5 text-[10px] font-bold uppercase tracking-[0.1em] text-rose-600"
                        >
                          <svg
                            width="10"
                            height="10"
                            viewBox="0 0 24 24"
                            fill="none"
                            stroke="currentColor"
                            stroke-width="3"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                            ><circle cx="12" cy="12" r="10" /><line
                              x1="12"
                              y1="8"
                              x2="12"
                              y2="12"
                            /><line x1="12" y1="16" x2="12.01" y2="16" /></svg
                          >
                          Failed{#if video.retry_count !== undefined && video.retry_count > 0}
                            ({video.retry_count}/{MAX_RETRIES}){/if}
                        </span>
                      {:else}
                        <span
                          class="text-[10px] font-bold uppercase tracking-[0.1em] text-[var(--soft-foreground)] opacity-40"
                        >
                          Queued
                        </span>
                      {/if}
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
                class="w-full rounded-[var(--radius-sm)] border border-[var(--border-soft)] py-2.5 text-[11px] font-bold uppercase tracking-[0.15em] text-[var(--soft-foreground)] transition-all hover:border-[var(--accent)]/40 hover:text-[var(--foreground)] disabled:opacity-30"
                onclick={() => loadVideos(false)}
                disabled={loadingVideos}
              >
                {loadingVideos ? "Loading..." : "Load More"}
              </button>
            </div>
          {/if}
        </div>
      </section>
    </main>

    <nav class="mobile-tab-bar lg:hidden" aria-label="Panel navigation">
      <button
        type="button"
        class="mobile-tab-item {mobileTab === 'channels'
          ? 'mobile-tab-item--active'
          : ''}"
        onclick={() => (mobileTab = "channels")}
        aria-current={mobileTab === "channels" ? "page" : undefined}
      >
        <svg
          class="h-5 w-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <rect x="3" y="3" width="6" height="18" rx="1.5" />
          <rect x="15" y="3" width="6" height="18" rx="1.5" />
          <rect x="9" y="3" width="6" height="18" rx="1.5" />
        </svg>
        <span>Channels</span>
      </button>
      <button
        type="button"
        class="mobile-tab-item {mobileTab === 'details'
          ? 'mobile-tab-item--active'
          : ''}"
        onclick={() => (mobileTab = "details")}
        aria-current={mobileTab === "details" ? "page" : undefined}
      >
        <svg
          class="h-5 w-5"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="1.5"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
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
      class="fixed bottom-6 max-lg:bottom-[calc(3.5rem+env(safe-area-inset-bottom)+1rem)] left-1/2 z-50 w-[min(90vw,420px)] -translate-x-1/2 rounded-[var(--radius-md)] bg-white border border-rose-200 px-4 py-3 shadow-lg fade-in"
      role="status"
      aria-live="polite"
    >
      <div class="flex items-start gap-3">
        <p class="text-[13px] font-medium text-rose-600 flex-1">
          {errorMessage}
        </p>
        <button
          onclick={() => (errorMessage = null)}
          class="shrink-0 text-[var(--soft-foreground)] opacity-40 hover:opacity-80"
          aria-label="Dismiss"
        >
          <svg
            width="14"
            height="14"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2.5"
            stroke-linecap="round"
            stroke-linejoin="round"
            ><line x1="18" y1="6" x2="6" y2="18"></line><line
              x1="6"
              y1="6"
              x2="18"
              y2="18"
            ></line></svg
          >
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
