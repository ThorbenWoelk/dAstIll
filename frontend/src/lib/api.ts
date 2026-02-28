import type {
	Channel,
	CleanTranscriptResponse,
	Summary,
	Transcript,
	VideoInfo,
	Video,
	VideoTypeFilter
} from './types';

const API_BASE = import.meta.env.VITE_API_BASE ?? 'http://localhost:3001';

async function request<T>(path: string, init?: RequestInit): Promise<T> {
	const response = await fetch(`${API_BASE}${path}`, {
		headers: {
			'Content-Type': 'application/json'
		},
		...init
	});

	if (!response.ok) {
		const message = await response.text();
		throw new Error(message || `Request failed (${response.status})`);
	}

	if (response.status === 204) {
		return undefined as T;
	}

	return (await response.json()) as T;
}

export function listChannels() {
	return request<Channel[]>('/api/channels');
}

export function addChannel(input: string) {
	return request<Channel>('/api/channels', {
		method: 'POST',
		body: JSON.stringify({ input })
	});
}

export function deleteChannel(id: string) {
	return request<void>(`/api/channels/${id}`, { method: 'DELETE' });
}

export function refreshChannel(id: string) {
	return request<{ videos_added: number }>(`/api/channels/${id}/refresh`, { method: 'POST' });
}

export function backfillChannelVideos(id: string, limit = 15) {
	const params = new URLSearchParams({
		limit: `${limit}`
	});
	return request<{ videos_added: number; fetched_count: number }>(
		`/api/channels/${id}/backfill?${params.toString()}`,
		{ method: 'POST' }
	);
}

export function listVideos(
	channelId: string,
	limit = 12,
	offset = 0,
	videoType: VideoTypeFilter = 'all',
	acknowledged?: boolean
) {
	const params = new URLSearchParams({
		limit: `${limit}`,
		offset: `${offset}`,
		video_type: videoType
	});
	if (acknowledged !== undefined) {
		params.append('acknowledged', acknowledged.toString());
	}
	return request<Video[]>(`/api/channels/${channelId}/videos?${params.toString()}`);
}

export function updateAcknowledged(videoId: string, acknowledged: boolean) {
	return request<Video>(`/api/videos/${videoId}/acknowledged`, {
		method: 'PUT',
		body: JSON.stringify({ acknowledged })
	});
}

export function getVideo(videoId: string) {
	return request<Video>(`/api/videos/${videoId}`);
}

export function getVideoInfo(videoId: string) {
	return request<VideoInfo>(`/api/videos/${videoId}/info`);
}

export function getTranscript(videoId: string) {
	return request<Transcript>(`/api/videos/${videoId}/transcript`);
}

export function updateTranscript(videoId: string, content: string) {
	return request<Transcript>(`/api/videos/${videoId}/transcript`, {
		method: 'PUT',
		body: JSON.stringify({ content })
	});
}

export function cleanTranscriptFormatting(videoId: string, content: string) {
	return request<CleanTranscriptResponse>(`/api/videos/${videoId}/transcript/clean`, {
		method: 'POST',
		body: JSON.stringify({ content })
	});
}

export function getSummary(videoId: string) {
	return request<Summary>(`/api/videos/${videoId}/summary`);
}

export function updateSummary(videoId: string, content: string) {
	return request<Summary>(`/api/videos/${videoId}/summary`, {
		method: 'PUT',
		body: JSON.stringify({ content })
	});
}
