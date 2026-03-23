export function looksLikeYouTubeVideoInput(input: string): boolean {
  const trimmed = input.trim();
  if (!trimmed) {
    return false;
  }

  if (/^[A-Za-z0-9_-]{11}$/.test(trimmed)) {
    return true;
  }

  try {
    const url = new URL(trimmed);
    const host = url.hostname.toLowerCase();

    if (host === "youtu.be") {
      const id = url.pathname.split("/").filter(Boolean)[0] ?? "";
      return /^[A-Za-z0-9_-]{11}$/.test(id);
    }

    if (!["youtube.com", "www.youtube.com", "m.youtube.com"].includes(host)) {
      return false;
    }

    if (url.pathname === "/watch") {
      return /^[A-Za-z0-9_-]{11}$/.test(url.searchParams.get("v") ?? "");
    }

    const [kind, id] = url.pathname.split("/").filter(Boolean);
    if (!["shorts", "embed", "live"].includes(kind ?? "")) {
      return false;
    }
    return /^[A-Za-z0-9_-]{11}$/.test(id ?? "");
  } catch {
    return false;
  }
}
