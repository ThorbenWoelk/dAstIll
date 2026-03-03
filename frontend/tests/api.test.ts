import { afterEach, describe, expect, it } from 'bun:test';
import { listChannelsWhenAvailable } from '../src/lib/api';
import type { Channel } from '../src/lib/types';

const originalFetch = globalThis.fetch;

function channel(id: string): Channel {
	return {
		id,
		name: `Channel ${id}`,
		added_at: '2026-03-02T00:00:00.000Z'
	};
}

afterEach(() => {
	globalThis.fetch = originalFetch;
});

describe('listChannelsWhenAvailable', () => {
	it('retries when backend is unreachable and resolves once reachable', async () => {
		const expected = [channel('abc')];
		let attempts = 0;

		globalThis.fetch = (async () => {
			attempts += 1;
			if (attempts === 1) {
				throw new TypeError('fetch failed');
			}
			return new Response(JSON.stringify(expected), { status: 200 });
		}) as typeof fetch;

		const result = await listChannelsWhenAvailable({ retryDelayMs: 0 });
		expect(result).toEqual(expected);
		expect(attempts).toBe(2);
	});

	it('does not retry non-reachability failures', async () => {
		let attempts = 0;

		globalThis.fetch = (async () => {
			attempts += 1;
			return new Response('bad request', { status: 400 });
		}) as typeof fetch;

		await expect(
			listChannelsWhenAvailable({ retryDelayMs: 0 })
		).rejects.toThrow('bad request');
		expect(attempts).toBe(1);
	});
});
