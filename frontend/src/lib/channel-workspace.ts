import type { Channel } from './types';

export const WORKSPACE_STATE_KEY = "dastill.workspace.state.v1";

export interface WorkspaceStateSnapshot {
	selectedChannelId: string | null;
	selectedVideoId: string | null;
	contentMode: "transcript" | "summary" | "info";
	videoTypeFilter: "all" | "long" | "short";
	hideShorts?: boolean;
	acknowledgedFilter: "all" | "unack" | "ack";
	channelOrder: string[];
	channelSortMode?: "custom" | "alpha" | "newest";
}

export function prioritizeChannelOrder(
	channelOrder: string[],
	channelId: string
): string[] {
	return [channelId, ...channelOrder.filter((id) => id !== channelId)];
}

export function applySavedChannelOrder(
	nextChannels: Channel[],
	channelOrder: string[]
): Channel[] {
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

export function resolveInitialChannelSelection(
	channels: Channel[],
	selectedChannelId: string | null,
	preferredChannelId: string | null
): string | null {
	if (channels.length === 0) return null;

	if (preferredChannelId && channels.some((channel) => channel.id === preferredChannelId)) {
		return preferredChannelId;
	}

	if (selectedChannelId && channels.some((channel) => channel.id === selectedChannelId)) {
		return selectedChannelId;
	}

	return channels[0].id;
}
