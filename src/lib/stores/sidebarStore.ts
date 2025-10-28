import { writable } from "svelte/store"

/**
 * Store to trigger sidebar refresh when prompt sets or revisions change
 */
function createSidebarStore() {
	const { subscribe, update } = writable(0)

	return {
		subscribe,
		/**
		 * Trigger sidebar to refresh its data
		 */
		refresh: () => {
			update((n) => n + 1)
		},
	}
}

export const sidebarStore = createSidebarStore()
