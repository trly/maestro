/**
 * Convert a full UUID to a short hash (first 8 characters) for display purposes.
 * Storage and file paths should always use full UUIDs.
 */
export function toShortHash(uuid: string): string {
	return uuid.slice(0, 8);
}

/**
 * Format multiple UUIDs as short hashes joined by a separator.
 */
export function toShortHashPath(...uuids: string[]): string {
	return uuids.map(toShortHash).join('/');
}
