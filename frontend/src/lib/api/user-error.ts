type UserErrorOptions = {
  status?: number;
  fallback?: string;
};

function fallbackForStatus(status?: number) {
  switch (status) {
    case 400:
      return "Sorry, that request could not be completed.";
    case 401:
    case 403:
      return "You do not have access to do that.";
    case 404:
      return "Sorry, we could not find that.";
    case 409:
      return "That action is already in progress. Please try again in a moment.";
    case 429:
      return "Too many requests right now. Please wait a moment and try again.";
    case 500:
    case 502:
    case 503:
    case 504:
      return "Sorry, that part of the app is unavailable right now.";
    default:
      return "Sorry, something went wrong. Please try again.";
  }
}

export function normalizeUserErrorMessage(
  message: string,
  options: UserErrorOptions = {},
) {
  const raw = message.trim();
  if (!raw) {
    return options.fallback ?? fallbackForStatus(options.status);
  }

  const lower = raw.toLowerCase();

  if (
    lower === "sign-in required" ||
    lower.includes("sign-in required") ||
    lower === "sign in to continue."
  ) {
    return "Sign in to continue.";
  }

  if (
    lower.includes("polly tts is not configured") ||
    lower.includes("text-to-speech is currently not available") ||
    lower.includes("tts unavailable")
  ) {
    return "Sorry, audio playback is not available right now.";
  }

  if (
    lower.includes("summary audio not yet generated") ||
    lower.includes("audio not yet generated")
  ) {
    return "Audio is not ready yet. Please try again in a moment.";
  }

  if (
    lower.includes("requires a subscription") ||
    lower.includes("subscription limit") ||
    lower.includes("usage limit") ||
    lower.includes("quota exceeded") ||
    lower.includes("ollama cloud usage limit")
  ) {
    return "AI model quota is used up. Please try again later.";
  }

  if (lower.includes("ai generation is temporarily unavailable")) {
    return "AI is temporarily unavailable. Please try again later.";
  }

  if (
    lower.includes("ollama") ||
    lower.includes("summarizer unavailable") ||
    lower.includes("summary evaluator") ||
    lower.includes("generation failed") ||
    lower.includes("operator access required")
  ) {
    return "Sorry, AI features are not available right now.";
  }

  if (
    lower.includes("backend is unreachable") ||
    lower.includes("failed to fetch") ||
    lower.includes("connection refused") ||
    lower.includes("networkerror")
  ) {
    return "Sorry, we could not connect right now. Please try again.";
  }

  if (
    lower.startsWith("request failed (") ||
    lower === "service unavailable" ||
    lower.includes("bad gateway")
  ) {
    return options.fallback ?? fallbackForStatus(options.status);
  }

  if (
    lower.includes("s3 error") ||
    lower.includes("s3 vectors error") ||
    lower.includes("serialization error") ||
    lower.includes("database") ||
    lower.includes("libsql") ||
    lower.includes("gzip") ||
    lower.includes("firebase token") ||
    lower.includes("upstream")
  ) {
    return options.fallback ?? fallbackForStatus(options.status);
  }

  if (lower === "conversation not found") {
    return "That conversation could not be found.";
  }

  if (lower === "channel not found") {
    return "That source could not be found.";
  }

  if (lower === "summary not found") {
    return "The summary is not available yet.";
  }

  if (lower === "unauthorized") {
    return "You do not have access to do that.";
  }

  if (lower === "rate limit exceeded") {
    return "Too many requests right now. Please wait a moment and try again.";
  }

  return raw;
}

export function getUserErrorMessage(
  error: unknown,
  options: UserErrorOptions = {},
) {
  if (error instanceof Error) {
    return normalizeUserErrorMessage(error.message, options);
  }

  if (typeof error === "string") {
    return normalizeUserErrorMessage(error, options);
  }

  return options.fallback ?? fallbackForStatus(options.status);
}
