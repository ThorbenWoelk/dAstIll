export type SummaryAudioStatus =
  | "missing"
  | "generating"
  | "ready"
  | "playing"
  | "loading";

export type SummaryAudioDebugPayload = {
  word_count?: number | null;
  estimated_secs?: number | null;
  cache_hit?: boolean;
};

export type SummaryAudioSession = {
  status: SummaryAudioStatus;
  summaryAudioError: string | null;
  audioSrc: string | null;
  currentTime: number;
  duration: number;
  playbackRate: number;
  summaryWordCount: number | null;
  estimatedSecs: number | null;
};

type SummaryAudioListener = (session: SummaryAudioSession) => void;

const sessions = new Map<string, SummaryAudioSession>();
const listeners = new Map<string, Set<SummaryAudioListener>>();
const generationRequests = new Map<string, Promise<void>>();

function createDefaultSession(_videoId: string): SummaryAudioSession {
  return {
    status: "missing",
    summaryAudioError: null,
    audioSrc: null,
    currentTime: 0,
    duration: 0,
    playbackRate: 1,
    summaryWordCount: null,
    estimatedSecs: null,
  };
}

function cloneSession(session: SummaryAudioSession): SummaryAudioSession {
  return { ...session };
}

function audioSrcForVideo(videoId: string) {
  return `/api/videos/${videoId}/summary/audio`;
}

function sanitizeFinitePositive(value: number) {
  return Number.isFinite(value) && value > 0 ? value : 0;
}

function emit(videoId: string, session: SummaryAudioSession) {
  const next = cloneSession(session);
  for (const listener of listeners.get(videoId) ?? []) {
    listener(next);
  }
}

function ensureSession(videoId: string) {
  const existing = sessions.get(videoId);
  if (existing) {
    return existing;
  }

  const next = createDefaultSession(videoId);
  sessions.set(videoId, next);
  return next;
}

function updateSession(
  videoId: string,
  updater: (session: SummaryAudioSession) => SummaryAudioSession,
) {
  const current = ensureSession(videoId);
  const next = updater(cloneSession(current));
  sessions.set(videoId, next);
  emit(videoId, next);
  return next;
}

export function subscribeToSummaryAudioSession(
  videoId: string,
  listener: SummaryAudioListener,
) {
  const set = listeners.get(videoId) ?? new Set<SummaryAudioListener>();
  set.add(listener);
  listeners.set(videoId, set);
  listener(cloneSession(ensureSession(videoId)));

  return () => {
    const current = listeners.get(videoId);
    if (!current) {
      return;
    }
    current.delete(listener);
    if (current.size === 0) {
      listeners.delete(videoId);
    }
  };
}

export function readSummaryAudioSession(videoId: string) {
  return cloneSession(ensureSession(videoId));
}

export function syncSummaryAudioDebugState(
  videoId: string,
  payload: SummaryAudioDebugPayload,
) {
  updateSession(videoId, (session) => {
    session.summaryWordCount = payload.word_count ?? null;
    session.estimatedSecs = payload.estimated_secs ?? null;

    if (payload.cache_hit) {
      session.status = "ready";
      session.audioSrc = audioSrcForVideo(videoId);
      session.summaryAudioError = null;
      return session;
    }

    if (generationRequests.has(videoId)) {
      if (session.status === "missing") {
        session.status = "generating";
      }
      return session;
    }

    if (session.status === "playing" || session.status === "loading") {
      return session;
    }

    session.status = "missing";
    session.audioSrc = null;
    return session;
  });
}

export async function generateSummaryAudio(
  videoId: string,
  request: () => Promise<Response>,
) {
  const inFlight = generationRequests.get(videoId);
  if (inFlight) {
    return inFlight;
  }

  updateSession(videoId, (session) => ({
    ...session,
    status: "generating",
    summaryAudioError: null,
  }));

  const generation = (async () => {
    try {
      const response = await request();
      if (response.ok) {
        updateSession(videoId, (session) => ({
          ...session,
          status: "ready",
          summaryAudioError: null,
          audioSrc: audioSrcForVideo(videoId),
        }));
        return;
      }

      const message = (await response.text()) || "Failed to generate audio.";
      updateSession(videoId, (session) => ({
        ...session,
        status: "missing",
        summaryAudioError: message,
        audioSrc: null,
      }));
    } catch {
      updateSession(videoId, (session) => ({
        ...session,
        status: "missing",
        summaryAudioError: "Failed to generate audio.",
        audioSrc: null,
      }));
    } finally {
      generationRequests.delete(videoId);
    }
  })();

  generationRequests.set(videoId, generation);
  return generation;
}

export function updateSummaryAudioPlaybackRate(
  videoId: string,
  playbackRate: number,
) {
  updateSession(videoId, (session) => ({
    ...session,
    playbackRate,
  }));
}

export function updateSummaryAudioCurrentTime(
  videoId: string,
  currentTime: number,
) {
  updateSession(videoId, (session) => ({
    ...session,
    currentTime: Math.max(0, currentTime),
  }));
}

export function updateSummaryAudioDuration(videoId: string, duration: number) {
  const nextDuration = sanitizeFinitePositive(duration);
  if (nextDuration <= 0) {
    return;
  }

  updateSession(videoId, (session) => ({
    ...session,
    duration: nextDuration,
  }));
}

export function setSummaryAudioStatus(
  videoId: string,
  status: SummaryAudioStatus,
) {
  updateSession(videoId, (session) => ({
    ...session,
    status,
  }));
}

export function markSummaryAudioPlaybackStopped(videoId: string) {
  updateSession(videoId, (session) => ({
    ...session,
    status:
      session.status === "playing" || session.status === "loading"
        ? "ready"
        : session.status,
  }));
}

export function resetSummaryAudioPlayback(videoId: string) {
  updateSession(videoId, (session) => ({
    ...session,
    status: "ready",
    currentTime: 0,
  }));
}

export function resolveSummaryAudioTimelineState(
  currentTime: number,
  duration: number,
) {
  const knownDuration = Number.isFinite(duration) && duration > 0;
  if (!knownDuration) {
    return {
      knownDuration: false,
      sliderMax: 100,
      sliderValue: 0,
      progressPercent: 0,
    };
  }

  const safeCurrentTime = Math.max(0, Math.min(currentTime, duration));
  return {
    knownDuration: true,
    sliderMax: duration,
    sliderValue: safeCurrentTime,
    progressPercent: (safeCurrentTime / duration) * 100,
  };
}

export function resetSummaryAudioSessionsForTesting() {
  sessions.clear();
  listeners.clear();
  generationRequests.clear();
}
