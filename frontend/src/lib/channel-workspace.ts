import type { Channel } from './types';

export function prioritizeChannelOrder(
	channelOrder: string[],
	channelId: string
): string[] {
	return [channelId, ...channelOrder.filter((id) => id !== channelId)];
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
