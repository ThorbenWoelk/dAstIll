<script lang="ts">
  import { onDestroy } from "svelte";
  import {
    generateSummaryAudio,
    markSummaryAudioPlaybackStopped,
    readSummaryAudioSession,
    resetSummaryAudioPlayback,
    resolveSummaryAudioTimelineState,
    setSummaryAudioStatus,
    subscribeToSummaryAudioSession,
    syncSummaryAudioDebugState,
    updateSummaryAudioCurrentTime,
    updateSummaryAudioDuration,
    updateSummaryAudioPlaybackRate,
    type SummaryAudioStatus,
  } from "$lib/workspace/summary-audio-session";

  let {
    videoId,
    summaryReady = true,
  }: { videoId: string | null; summaryReady?: boolean } = $props();

  let status = $state<SummaryAudioStatus>("missing");
  let summaryAudioError = $state<string | null>(null);
  let audioPlayer = $state<HTMLAudioElement | null>(null);
  let audioSrc = $state<string | null>(null);
  let currentTime = $state(0);
  let duration = $state(0);
  let playbackRate = $state(1);
  let summaryWordCount = $state<number | null>(null);
  let estimatedSecs = $state<number | null>(null);

  const playbackRates = [1, 1.25, 1.5, 2, 2.5, 3, 0.75];
  const timelineState = $derived(
    resolveSummaryAudioTimelineState(currentTime, duration),
  );

  let unsubscribeSession: (() => void) | null = null;

  function applySession(videoIdValue: string) {
    const session = readSummaryAudioSession(videoIdValue);
    status = session.status;
    summaryAudioError = session.summaryAudioError;
    audioSrc = session.audioSrc;
    currentTime = session.currentTime;
    duration = session.duration;
    playbackRate = session.playbackRate;
    summaryWordCount = session.summaryWordCount;
    estimatedSecs = session.estimatedSecs;
  }

  function cyclePlaybackRate() {
    const currentIndex = playbackRates.indexOf(playbackRate);
    playbackRate = playbackRates[(currentIndex + 1) % playbackRates.length];
    if (audioPlayer) {
      audioPlayer.playbackRate = playbackRate;
    }
    if (videoId) {
      updateSummaryAudioPlaybackRate(videoId, playbackRate);
    }
  }

  function handleKeydown(e: KeyboardEvent) {
    if (status === "missing" || status === "generating") return;

    // Ignore if typing in an input
    if (
      e.target instanceof HTMLInputElement ||
      e.target instanceof HTMLTextAreaElement
    )
      return;

    if (e.code === "Space") {
      e.preventDefault();
      togglePlay();
    } else if (e.code === "ArrowLeft") {
      e.preventDefault();
      skip(-10);
    } else if (e.code === "ArrowRight") {
      e.preventDefault();
      skip(10);
    }
  }

  async function checkAudioStatus() {
    if (!videoId) return;
    try {
      const resp = await fetch(`/api/videos/${videoId}/summary/audio/debug`);
      if (resp.ok) {
        const data = await resp.json();
        syncSummaryAudioDebugState(videoId, data);
      }
    } catch (e) {
      console.error("Failed to check audio status", e);
    }
  }

  async function generateAudio() {
    if (!videoId) return;
    await generateSummaryAudio(videoId, () =>
      fetch(`/api/videos/${videoId}/summary/audio`, {
        method: "POST",
      }),
    );
  }

  function togglePlay() {
    if (!audioPlayer || !videoId) return;
    if (audioPlayer.paused) {
      audioPlayer.play();
      setSummaryAudioStatus(videoId, "playing");
    } else {
      audioPlayer.pause();
      setSummaryAudioStatus(videoId, "ready");
    }
  }

  function skip(seconds: number) {
    if (!audioPlayer) return;
    audioPlayer.currentTime = Math.max(
      0,
      Math.min(audioPlayer.duration, audioPlayer.currentTime + seconds),
    );
  }

  function onTimeUpdate() {
    if (audioPlayer && videoId) {
      updateSummaryAudioCurrentTime(videoId, audioPlayer.currentTime);
    }
  }

  function syncKnownDuration() {
    if (audioPlayer && videoId) {
      updateSummaryAudioDuration(videoId, audioPlayer.duration);
    }
  }

  function onEnded() {
    if (!videoId) return;
    resetSummaryAudioPlayback(videoId);
    if (audioPlayer) {
      audioPlayer.currentTime = 0;
    }
  }

  function onPlay() {
    if (videoId) {
      setSummaryAudioStatus(videoId, "playing");
    }
  }

  function onPause() {
    if (videoId && status === "playing") {
      setSummaryAudioStatus(videoId, "ready");
    }
  }

  function onWaiting() {
    if (videoId) {
      setSummaryAudioStatus(videoId, "loading");
    }
  }

  function onCanPlay() {
    syncKnownDuration();
    if (audioPlayer) {
      audioPlayer.playbackRate = playbackRate;
      if (videoId) {
        updateSummaryAudioPlaybackRate(videoId, playbackRate);
      }
      if (!audioPlayer.paused) {
        if (videoId) {
          setSummaryAudioStatus(videoId, "playing");
        }
      } else {
        if (videoId) {
          setSummaryAudioStatus(videoId, "ready");
        }
      }
    }
  }

  function handleScrub(e: Event) {
    const target = e.target as HTMLInputElement;
    if (audioPlayer) {
      audioPlayer.currentTime = parseFloat(target.value);
    }
  }

  $effect(() => {
    const activeVideoId = videoId;
    unsubscribeSession?.();
    unsubscribeSession = null;

    if (!activeVideoId) {
      status = "missing";
      audioSrc = null;
      summaryAudioError = null;
      currentTime = 0;
      duration = 0;
      summaryWordCount = null;
      estimatedSecs = null;
      return;
    }

    applySession(activeVideoId);
    unsubscribeSession = subscribeToSummaryAudioSession(activeVideoId, () => {
      applySession(activeVideoId);
    });
    void checkAudioStatus();

    return () => {
      if (audioPlayer) {
        audioPlayer.pause();
      }
      markSummaryAudioPlaybackStopped(activeVideoId);
      unsubscribeSession?.();
      unsubscribeSession = null;
    };
  });

  onDestroy(() => {
    unsubscribeSession?.();
  });
</script>

<svelte:window onkeydown={handleKeydown} />

<div class="mb-4 flex flex-col gap-1.5">
  <div class="flex items-center gap-3 py-1">
    <div class="flex items-center gap-1">
      {#if status === "generating" || status === "loading"}
        <div
          class="flex h-10 w-10 items-center justify-center rounded-full bg-[var(--accent-soft)]/20 text-[var(--accent-strong)]"
        >
          <span
            class="h-4 w-4 animate-spin rounded-full border-2 border-[var(--soft-foreground)]/30 border-t-[var(--accent)]"
          ></span>
        </div>
      {:else if status === "missing"}
        <button
          onclick={generateAudio}
          disabled={!summaryReady}
          class="group flex h-10 w-10 items-center justify-center rounded-full bg-[var(--accent-soft)]/40 text-[var(--accent-strong)] transition-all hover:bg-[var(--accent-wash)] hover:scale-105 active:scale-95 disabled:pointer-events-none disabled:opacity-40"
          title={summaryReady
            ? "Generate Summary Audio"
            : "Summary not yet available"}
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4" />
            <polyline points="7 10 12 15 17 10" />
            <line x1="12" x2="12" y1="3" y2="15" />
          </svg>
        </button>
      {:else}
        <button
          onclick={() => skip(-10)}
          class="relative flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100 active:scale-95"
          title="Back 10s (Arrow Left)"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M3 12a9 9 0 1 0 9-9 9.75 9.75 0 0 0-6.74 2.74L3 8" />
            <path d="M3 3v5h5" />
          </svg>
          <span
            class="absolute bottom-0.5 right-0.5 text-[8px] font-bold leading-none"
            style="letter-spacing: -0.03em">10</span
          >
        </button>

        <button
          onclick={togglePlay}
          class="group flex h-10 w-10 items-center justify-center rounded-full bg-[var(--accent-soft)]/40 text-[var(--accent-strong)] transition-all hover:bg-[var(--accent-wash)] hover:scale-105 active:scale-95"
          title={status === "playing" ? "Pause (Space)" : "Play (Space)"}
        >
          {#if status === "playing"}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect x="6" y="4" width="3" height="16" />
              <rect x="15" y="4" width="3" height="16" />
            </svg>
          {:else}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="18"
              height="18"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="ml-0.5"
            >
              <polygon points="5 3 19 12 5 21 5 3" />
            </svg>
          {/if}
        </button>

        <button
          onclick={() => skip(10)}
          class="relative flex h-8 w-8 items-center justify-center rounded-full text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100 active:scale-95"
          title="Forward 10s (Arrow Right)"
        >
          <svg
            xmlns="http://www.w3.org/2000/svg"
            width="16"
            height="16"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
          >
            <path d="M21 12a9 9 0 1 1-9-9 9.75 9.75 0 0 1 6.74 2.74L21 8" />
            <path d="M21 3v5h-5" />
          </svg>
          <span
            class="absolute bottom-0.5 left-0.5 text-[8px] font-bold leading-none"
            style="letter-spacing: -0.03em">10</span
          >
        </button>

        <button
          onclick={cyclePlaybackRate}
          class="ml-1 flex h-8 min-w-[32px] items-center justify-center rounded-full px-1.5 text-[10px] font-bold text-[var(--soft-foreground)] opacity-60 transition-all hover:bg-[var(--accent-wash)] hover:opacity-100 active:scale-95"
          title="Cycle Playback Speed"
        >
          {playbackRate}x
        </button>
      {/if}
    </div>

    <div class="flex flex-1 flex-col gap-1.5 min-w-0">
      <div class="flex items-center justify-between">
        {#if status === "generating" || status === "loading"}
          <span
            class="text-[10px] font-bold uppercase tracking-[0.08em] text-[var(--soft-foreground)] opacity-70"
          >
            {status === "generating" ? "Generating audio" : "Loading"}
          </span>
        {:else if status === "missing" && summaryWordCount !== null}
          {@const speakMins = Math.round(summaryWordCount / 140) || 1}
          <span class="text-[10px] text-[var(--soft-foreground)] opacity-50">
            {summaryWordCount} words &middot; ~{speakMins} min audio
            {#if estimatedSecs !== null}
              &middot; ~{Math.round(estimatedSecs)}s to generate
            {/if}
          </span>
        {:else}
          <div></div>
        {/if}
        {#if currentTime > 0 || duration > 0}
          {@const knownDuration = isFinite(duration) && duration > 0}
          <span
            class="text-[10px] tabular-nums text-[var(--soft-foreground)] opacity-50"
          >
            {Math.floor(currentTime / 60)}:{(currentTime % 60)
              .toFixed(0)
              .padStart(2, "0")}{#if knownDuration}
              / {Math.floor(duration / 60)}:{(duration % 60)
                .toFixed(0)
                .padStart(2, "0")}{/if}
          </span>
        {/if}
      </div>
      <div class="group relative flex h-4 items-center">
        <input
          type="range"
          min="0"
          max={timelineState.sliderMax}
          step="0.1"
          value={timelineState.sliderValue}
          oninput={handleScrub}
          disabled={status === "missing" ||
            status === "generating" ||
            !timelineState.knownDuration}
          class="timeline-slider w-full cursor-pointer appearance-none bg-transparent"
        />
        <div
          class="pointer-events-none absolute h-1 w-full rounded-full bg-[var(--soft-foreground)]/10"
          aria-hidden="true"
        >
          <div
            class="h-full rounded-full bg-[var(--accent)] transition-all duration-75"
            style="width: {timelineState.progressPercent}%"
          ></div>
        </div>
      </div>
    </div>
  </div>

  {#if summaryAudioError}
    <span class="px-1 text-[10px] font-medium text-[var(--danger)]"
      >{summaryAudioError}</span
    >
  {/if}

  {#if audioSrc}
    <audio
      bind:this={audioPlayer}
      src={audioSrc}
      ontimeupdate={onTimeUpdate}
      onloadedmetadata={syncKnownDuration}
      ondurationchange={syncKnownDuration}
      onloadeddata={syncKnownDuration}
      onended={onEnded}
      onplay={onPlay}
      onpause={onPause}
      onwaiting={onWaiting}
      oncanplay={onCanPlay}
      preload="metadata"
      class="hidden"
    ></audio>
  {/if}
</div>

<style>
  .timeline-slider {
    z-index: 10;
  }

  .timeline-slider::-webkit-slider-thumb {
    appearance: none;
    height: 12px;
    width: 12px;
    border-radius: 50%;
    background: var(--accent-strong);
    cursor: pointer;
    border: 2px solid var(--panel-surface);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .group:hover .timeline-slider::-webkit-slider-thumb {
    opacity: 1;
  }

  .timeline-slider:focus::-webkit-slider-thumb {
    opacity: 1;
    box-shadow: 0 0 0 3px var(--accent-soft);
  }

  /* Firefox */
  .timeline-slider::-moz-range-thumb {
    height: 12px;
    width: 12px;
    border-radius: 50%;
    background: var(--accent-strong);
    cursor: pointer;
    border: 2px solid var(--panel-surface);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
    opacity: 0;
    transition: opacity 0.15s ease;
  }

  .group:hover .timeline-slider::-moz-range-thumb {
    opacity: 1;
  }
</style>
