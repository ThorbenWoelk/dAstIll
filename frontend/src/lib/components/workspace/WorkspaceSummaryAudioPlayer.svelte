<script lang="ts">
  import { onDestroy } from "svelte";
  import { createApiRequestInit, resolveApiUrl } from "$lib/api-client";
  import {
    generateSummaryAudio,
    markSummaryAudioPlaybackStopped,
    readSummaryAudioSession,
    resetSummaryAudioPlayback,
    resolveSummaryAudioTimelineState,
    setSummaryAudioUnavailable,
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
  let audioRequested = $state(false);
  let audioPlayer = $state<HTMLAudioElement | null>(null);
  let audioSrc = $state<string | null>(null);
  let currentTime = $state(0);
  let duration = $state(0);
  let playbackRate = $state(1);
  let summaryWordCount = $state<number | null>(null);
  let estimatedSecs = $state<number | null>(null);
  let waveformContainer = $state<HTMLDivElement | null>(null);

  const playbackRates = [1, 1.25, 1.5, 2, 2.5, 3, 0.75];
  const BAR_COUNT = 80;
  const timelineState = $derived(
    resolveSummaryAudioTimelineState(currentTime, duration),
  );
  const unavailableMessage = $derived(
    summaryAudioError || "Text-to-speech is currently unavailable.",
  );

  let unsubscribeSession: (() => void) | null = null;

  // Generate a deterministic waveform pattern from the videoId
  function generateWaveformBars(id: string | null): number[] {
    if (!id) return Array(BAR_COUNT).fill(0.3);
    let seed = 0;
    for (let i = 0; i < id.length; i++) {
      seed = ((seed << 5) - seed + id.charCodeAt(i)) | 0;
    }
    const bars: number[] = [];
    for (let i = 0; i < BAR_COUNT; i++) {
      seed = (seed * 16807 + 0) % 2147483647;
      const base = (seed & 0xffff) / 0xffff;
      // Shape: gentle arc with variation
      const position = i / BAR_COUNT;
      const envelope =
        0.3 + 0.7 * Math.sin(position * Math.PI) * (0.5 + 0.5 * base);
      bars.push(Math.max(0.08, Math.min(1, envelope)));
    }
    return bars;
  }

  const waveformBars = $derived(generateWaveformBars(videoId));

  function applySession(videoIdValue: string) {
    const session = readSummaryAudioSession(videoIdValue);
    status = session.status;
    summaryAudioError = session.summaryAudioError;
    audioRequested = session.audioRequested;
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
    if (
      status === "missing" ||
      status === "unavailable" ||
      status === "generating"
    )
      return;

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
      const resp = await fetch(
        resolveApiUrl(`/api/videos/${videoId}/summary/audio/debug`),
        await createApiRequestInit(undefined, {
          includeJsonContentType: false,
        }),
      );
      if (resp.ok) {
        const data = await resp.json();
        syncSummaryAudioDebugState(videoId, data);
        return;
      }
      if (resp.status === 503) {
        if (readSummaryAudioSession(videoId).audioRequested) {
          const message =
            (await resp.text()) || "Text-to-speech is currently unavailable.";
          setSummaryAudioUnavailable(videoId, message);
        }
      }
    } catch (e) {
      console.error("Failed to check audio status", e);
    }
  }

  async function generateAudio() {
    if (!videoId) return;
    await generateSummaryAudio(videoId, async () =>
      fetch(
        resolveApiUrl(`/api/videos/${videoId}/summary/audio`),
        await createApiRequestInit(
          {
            method: "POST",
          },
          {
            includeJsonContentType: false,
          },
        ),
      ),
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

  function handleWaveformClick(e: MouseEvent) {
    if (
      !waveformContainer ||
      !audioPlayer ||
      !timelineState.knownDuration ||
      status === "missing" ||
      status === "unavailable" ||
      status === "generating"
    )
      return;
    const rect = waveformContainer.getBoundingClientRect();
    const fraction = Math.max(
      0,
      Math.min(1, (e.clientX - rect.left) / rect.width),
    );
    audioPlayer.currentTime = fraction * duration;
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

<div class="waveform-player">
  {#if status === "unavailable" && audioRequested}
    <div class="waveform-area waveform-area-unavailable" role="status">
      <div class="waveform-bars waveform-bars-idle" aria-hidden="true">
        {#each waveformBars as height}
          <div class="waveform-bar" style="height: {height * 100}%"></div>
        {/each}
      </div>
      <div class="waveform-unavailable-copy">
        <span class="waveform-status-label">Audio unavailable</span>
        <span class="waveform-unavailable-text">{unavailableMessage}</span>
      </div>
    </div>
  {:else if status === "missing"}
    <!-- Generate prompt with waveform preview -->
    <button
      class="waveform-generate-btn"
      onclick={generateAudio}
      disabled={!summaryReady}
      title={summaryReady ? "Generate audio" : "Summary not yet available"}
    >
      <div class="waveform-bars waveform-bars-idle" aria-hidden="true">
        {#each waveformBars as height}
          <div class="waveform-bar" style="height: {height * 100}%"></div>
        {/each}
      </div>
      <div class="waveform-generate-overlay">
        <svg
          xmlns="http://www.w3.org/2000/svg"
          width="14"
          height="14"
          viewBox="0 0 24 24"
          fill="none"
          stroke="currentColor"
          stroke-width="2"
          stroke-linecap="round"
          stroke-linejoin="round"
        >
          <polygon points="5 3 19 12 5 21 5 3" />
        </svg>
        <span class="waveform-generate-label">
          {#if summaryWordCount !== null}
            {@const speakMins = Math.round(summaryWordCount / 140) || 1}
            Generate ~{speakMins}m audio
          {:else}
            Generate audio
          {/if}
        </span>
      </div>
    </button>
  {:else if status === "generating" || status === "loading"}
    <!-- Generating state with animated waveform -->
    <div class="waveform-area">
      <div class="waveform-bars waveform-bars-generating" aria-hidden="true">
        {#each waveformBars as height, i}
          <div
            class="waveform-bar"
            style="height: {height * 100}%; animation-delay: {i * 20}ms"
          ></div>
        {/each}
      </div>
      <span class="waveform-status-label">
        {status === "generating" ? "Generating audio..." : "Loading..."}
      </span>
    </div>
  {:else}
    <!-- Active player with interactive waveform -->
    <div class="waveform-area waveform-area-active">
      <div class="waveform-controls">
        <button
          class="waveform-play-btn"
          onclick={togglePlay}
          title={status === "playing" ? "Pause (Space)" : "Play (Space)"}
        >
          {#if status === "playing"}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
            >
              <rect x="6" y="4" width="3" height="16" />
              <rect x="15" y="4" width="3" height="16" />
            </svg>
          {:else}
            <svg
              xmlns="http://www.w3.org/2000/svg"
              width="12"
              height="12"
              viewBox="0 0 24 24"
              fill="none"
              stroke="currentColor"
              stroke-width="2.5"
              stroke-linecap="round"
              stroke-linejoin="round"
              class="ml-px"
            >
              <polygon points="5 3 19 12 5 21 5 3" />
            </svg>
          {/if}
        </button>

        <button
          class="waveform-rate-btn"
          onclick={cyclePlaybackRate}
          title="Playback speed"
        >
          {playbackRate}x
        </button>
      </div>

      <!-- svelte-ignore a11y_click_events_have_key_events -->
      <!-- svelte-ignore a11y_no_static_element_interactions -->
      <div
        class="waveform-bars waveform-bars-interactive"
        bind:this={waveformContainer}
        onclick={handleWaveformClick}
        role="slider"
        tabindex="0"
        aria-label="Audio progress"
        aria-valuenow={Math.round(currentTime)}
        aria-valuemin={0}
        aria-valuemax={Math.round(duration)}
      >
        {#each waveformBars as height, i}
          {@const barProgress = i / BAR_COUNT}
          <div
            class="waveform-bar {barProgress <=
            timelineState.progressPercent / 100
              ? 'waveform-bar-played'
              : 'waveform-bar-unplayed'}"
            style="height: {height * 100}%"
          ></div>
        {/each}
      </div>

      <div class="waveform-time">
        {#if currentTime > 0 || duration > 0}
          {@const knownDuration = isFinite(duration) && duration > 0}
          <span class="waveform-time-text">
            {Math.floor(currentTime / 60)}:{(currentTime % 60)
              .toFixed(0)
              .padStart(2, "0")}{#if knownDuration}
              / {Math.floor(duration / 60)}:{(duration % 60)
                .toFixed(0)
                .padStart(2, "0")}{/if}
          </span>
        {/if}
      </div>
    </div>
  {/if}

  {#if summaryAudioError && status !== "unavailable"}
    <span class="waveform-error">{summaryAudioError}</span>
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
  .waveform-player {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
    margin-bottom: 1.5rem;
  }

  /* Waveform bars container */
  .waveform-bars {
    display: flex;
    align-items: flex-end;
    gap: 1.5px;
    height: 40px;
    width: 100%;
  }

  .waveform-bar {
    flex: 1;
    min-width: 0;
    border-radius: 1px;
    background: var(--accent);
    opacity: 0.25;
    transition:
      opacity 0.1s ease,
      background 0.1s ease;
  }

  /* Idle / generate state */
  .waveform-generate-btn {
    position: relative;
    display: flex;
    flex-direction: column;
    align-items: stretch;
    background: none;
    border: none;
    padding: 0;
    cursor: pointer;
    font-family: inherit;
    color: inherit;
    border-radius: var(--radius-sm);
    overflow: hidden;
    transition: opacity 0.15s ease;
  }

  .waveform-generate-btn:disabled {
    pointer-events: none;
    opacity: 0.3;
  }

  .waveform-generate-btn:hover .waveform-bars-idle .waveform-bar {
    opacity: 0.4;
  }

  .waveform-generate-overlay {
    position: absolute;
    inset: 0;
    display: flex;
    align-items: center;
    justify-content: center;
    gap: 0.5rem;
    opacity: 1;
    transition:
      opacity 0.15s ease,
      transform 0.15s ease;
    color: var(--accent-strong);
    background: linear-gradient(
      180deg,
      color-mix(in srgb, var(--surface) 58%, transparent) 0%,
      color-mix(in srgb, var(--surface) 72%, transparent) 100%
    );
  }

  .waveform-generate-label {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
  }

  .waveform-generate-btn:hover .waveform-generate-overlay {
    transform: translateY(-1px);
  }

  /* Generating animation */
  .waveform-bars-generating .waveform-bar {
    animation: wave-pulse 1.2s ease-in-out infinite alternate;
  }

  @keyframes wave-pulse {
    0% {
      opacity: 0.15;
      transform: scaleY(0.6);
    }
    100% {
      opacity: 0.5;
      transform: scaleY(1);
    }
  }

  /* Active/interactive waveform */
  .waveform-area {
    display: flex;
    flex-direction: column;
    gap: 0.375rem;
  }

  .waveform-area-active {
    display: grid;
    grid-template-columns: auto 1fr auto;
    grid-template-rows: auto;
    align-items: center;
    gap: 0.75rem;
  }

  .waveform-area-unavailable {
    gap: 0.55rem;
    opacity: 0.72;
  }

  .waveform-unavailable-copy {
    display: flex;
    flex-direction: column;
    gap: 0.12rem;
  }

  .waveform-unavailable-text {
    font-size: 11px;
    line-height: 1.4;
    color: var(--soft-foreground);
  }

  .waveform-controls {
    display: flex;
    align-items: center;
    gap: 0.25rem;
  }

  .waveform-play-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: var(--accent-soft);
    border: none;
    color: var(--accent-strong);
    cursor: pointer;
    transition: all 0.15s ease;
    padding: 0;
  }

  .waveform-play-btn:hover {
    background: var(--accent-wash);
    transform: scale(1.05);
  }

  .waveform-play-btn:active {
    transform: scale(0.95);
  }

  .waveform-rate-btn {
    display: flex;
    align-items: center;
    justify-content: center;
    min-width: 28px;
    height: 20px;
    border-radius: 9999px;
    background: none;
    border: none;
    color: var(--soft-foreground);
    font-size: 9px;
    font-weight: 700;
    cursor: pointer;
    padding: 0 4px;
    opacity: 0.5;
    transition: opacity 0.15s ease;
    font-family: inherit;
  }

  .waveform-rate-btn:hover {
    opacity: 1;
  }

  .waveform-bars-interactive {
    cursor: pointer;
  }

  .waveform-bars-interactive:hover .waveform-bar {
    opacity: 0.35;
  }

  .waveform-bars-interactive:hover .waveform-bar-played {
    opacity: 0.9;
  }

  .waveform-bar-played {
    opacity: 0.8;
    background: var(--accent);
  }

  .waveform-bar-unplayed {
    opacity: 0.2;
    background: var(--accent);
  }

  .waveform-time {
    display: flex;
    align-items: center;
    justify-content: flex-end;
  }

  .waveform-time-text {
    font-size: 10px;
    font-variant-numeric: tabular-nums;
    color: var(--soft-foreground);
    opacity: 0.5;
    white-space: nowrap;
  }

  .waveform-status-label {
    font-size: 10px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.08em;
    color: var(--soft-foreground);
    opacity: 0.6;
    text-align: center;
    margin-top: 0.25rem;
  }

  .waveform-error {
    font-size: 10px;
    font-weight: 500;
    color: var(--danger);
    padding: 0 0.25rem;
  }

  .ml-px {
    margin-left: 1px;
  }

  @media (max-width: 1023px) {
    .waveform-player {
      margin-bottom: 0;
      min-width: 0;
    }

    .waveform-area,
    .waveform-area-unavailable {
      gap: 0.3rem;
    }

    .waveform-bars {
      height: 26px;
    }

    .waveform-generate-btn {
      min-width: 0;
    }

    .waveform-generate-overlay {
      justify-content: flex-start;
      padding: 0 0.35rem;
      gap: 0.35rem;
    }

    .waveform-generate-label,
    .waveform-status-label {
      font-size: 9px;
      letter-spacing: 0.1em;
    }

    .waveform-area-unavailable {
      justify-content: center;
    }

    .waveform-unavailable-copy {
      gap: 0.04rem;
    }

    .waveform-unavailable-text {
      font-size: 10px;
      line-height: 1.25;
    }

    .waveform-area-active {
      grid-template-columns: auto 1fr auto;
      gap: 0.45rem;
    }

    .waveform-controls {
      gap: 0.15rem;
    }

    .waveform-play-btn {
      width: 24px;
      height: 24px;
    }

    .waveform-rate-btn {
      min-width: 24px;
      height: 18px;
      font-size: 8px;
      padding: 0 3px;
    }

    .waveform-time-text {
      font-size: 9px;
    }
  }
</style>
