/**
 * Simple stale-while-revalidate cache for API responses.
 * Prevents redundant fetches when navigating between departments.
 */

const store = new Map<string, { data: unknown; timestamp: number }>();
const DEFAULT_TTL = 60_000; // 1 minute

/**
 * Returns cached data if fresh, otherwise fetches and caches.
 */
export async function cached<T>(
	key: string,
	fetcher: () => Promise<T>,
	ttl = DEFAULT_TTL
): Promise<T> {
	const entry = store.get(key);
	if (entry && Date.now() - entry.timestamp < ttl) {
		return entry.data as T;
	}
	const data = await fetcher();
	store.set(key, { data, timestamp: Date.now() });
	return data;
}

/**
 * Invalidate a specific cache key (e.g. after a mutation).
 */
export function invalidate(key: string): void {
	store.delete(key);
}

/**
 * Invalidate all keys matching a prefix (e.g. 'agents:' clears all agent caches).
 */
export function invalidatePrefix(prefix: string): void {
	for (const key of store.keys()) {
		if (key.startsWith(prefix)) store.delete(key);
	}
}

/**
 * Clear entire cache.
 */
export function invalidateAll(): void {
	store.clear();
}
