function normalizeApiBase(value?: string) {
  const normalized = value?.trim();
  if (!normalized) {
    return "";
  }

  return normalized.endsWith("/") ? normalized.slice(0, -1) : normalized;
}

export const API_BASE = normalizeApiBase(
  (import.meta as { env?: { VITE_API_BASE?: string } }).env?.VITE_API_BASE,
);

export class BackendUnavailableError extends Error {
  constructor(message = "Backend is unreachable.") {
    super(message);
    this.name = "BackendUnavailableError";
  }
}

export function isAbortError(error: unknown): boolean {
  return error instanceof Error && error.name === "AbortError";
}

export function createAbortError(): Error {
  if (typeof DOMException !== "undefined") {
    return new DOMException("The operation was aborted.", "AbortError");
  }
  const error = new Error("The operation was aborted.");
  error.name = "AbortError";
  return error;
}

export function resolveApiUrl(path: string): string {
  return `${API_BASE}${path}`;
}

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  let response: Response;
  try {
    response = await fetch(resolveApiUrl(path), {
      headers: {
        "Content-Type": "application/json",
      },
      ...init,
    });
  } catch (error) {
    if (isAbortError(error)) {
      throw error;
    }
    throw new BackendUnavailableError();
  }

  if (!response.ok) {
    const message = await response.text();
    throw new Error(message || `Request failed (${response.status})`);
  }

  if (response.status === 204) {
    return undefined as T;
  }

  return (await response.json()) as T;
}
