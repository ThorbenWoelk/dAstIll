import { getChannelSuggestions, getVideoSuggestions } from "$lib/chat-api";

export type ChatMentionKind = "channel" | "video";

export type ChatMentionToken = {
  kind: ChatMentionKind;
  raw: string;
  query: string;
};

export type ChatMentionSegment =
  | { type: "text"; value: string }
  | { type: "mention"; mention: ChatMentionToken };

export type ResolvedChatMention = {
  kind: ChatMentionKind;
  raw: string;
  query: string;
  label: string;
  resolved: boolean;
};

const MENTION_PATTERN = /([@+])\{([^{}]+)\}/g;

const resolutionCache = new Map<string, Promise<ResolvedChatMention>>();

function normalizeMentionValue(value: string): string {
  return value.trim().replace(/\s+/g, " ").toLocaleLowerCase();
}

function resolutionKey(mention: ChatMentionToken): string {
  return `${mention.kind}:${normalizeMentionValue(mention.query)}`;
}

export function parseChatMentionSegments(text: string): ChatMentionSegment[] {
  const segments: ChatMentionSegment[] = [];
  let cursor = 0;

  for (const match of text.matchAll(MENTION_PATTERN)) {
    const start = match.index ?? -1;
    if (start < 0) {
      continue;
    }
    if (start > cursor) {
      segments.push({ type: "text", value: text.slice(cursor, start) });
    }

    const trigger = match[1];
    const query = (match[2] ?? "").trim();
    const raw = match[0];
    if (!query) {
      segments.push({ type: "text", value: raw });
    } else {
      segments.push({
        type: "mention",
        mention: {
          kind: trigger === "@" ? "channel" : "video",
          raw,
          query,
        },
      });
    }

    cursor = start + raw.length;
  }

  if (cursor < text.length) {
    segments.push({ type: "text", value: text.slice(cursor) });
  }

  if (segments.length === 0) {
    segments.push({ type: "text", value: text });
  }

  return segments;
}

export function extractChatMentions(text: string): ChatMentionToken[] {
  return parseChatMentionSegments(text)
    .filter(
      (segment): segment is Extract<ChatMentionSegment, { type: "mention" }> =>
        segment.type === "mention",
    )
    .map((segment) => segment.mention);
}

export function mentionDisplayFallback(mention: ChatMentionToken): string {
  return mention.query;
}

export function buildResolvedMentionLabel(
  kind: ChatMentionKind,
  label: string,
  subtitle?: string | null,
): string {
  if (kind === "video" && subtitle?.trim()) {
    return `${subtitle.trim()} - ${label.trim()}`;
  }
  return label.trim();
}

export function pickExactMentionSuggestion<
  T extends { label: string; subtitle?: string | null },
>(mention: ChatMentionToken, items: T[]): T | null {
  const target = normalizeMentionValue(mention.query);
  return (
    items.find((item) => normalizeMentionValue(item.label) === target) ?? null
  );
}

export async function resolveChatMention(
  mention: ChatMentionToken,
): Promise<ResolvedChatMention> {
  const key = resolutionKey(mention);
  const cached = resolutionCache.get(key);
  if (cached) {
    return cached;
  }

  const resolution = (async (): Promise<ResolvedChatMention> => {
    const items =
      mention.kind === "channel"
        ? await getChannelSuggestions(mention.query, { limit: 8 })
        : await getVideoSuggestions(mention.query, { limit: 8 });
    const exact = pickExactMentionSuggestion(mention, items);

    return {
      kind: mention.kind,
      raw: mention.raw,
      query: mention.query,
      label: exact
        ? buildResolvedMentionLabel(mention.kind, exact.label, exact.subtitle)
        : mentionDisplayFallback(mention),
      resolved: Boolean(exact),
    };
  })();

  resolutionCache.set(key, resolution);
  return resolution;
}
