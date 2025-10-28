/**
 * Composable for managing multi-select state in lists
 * Follows Svelte 5 runes pattern for reactive state management
 */
import { SvelteSet } from "svelte/reactivity"

export interface SelectionState {
	selectedIds: SvelteSet<string>
	toggleAll: (itemIds: string[]) => void
	toggle: (id: string) => void
	clear: () => void
	getSelected: <T extends { id: string }>(items: T[]) => T[]
}

export function useSelection(): SelectionState {
	let selectedIds = $state(new SvelteSet<string>())

	function toggleAll(itemIds: string[]) {
		if (itemIds.length === 0) {
			selectedIds.clear()
			return
		}

		const allCurrentlySelected = itemIds.every((id) => selectedIds.has(id))
		if (allCurrentlySelected) {
			selectedIds.clear()
		} else {
			selectedIds.clear()
			itemIds.forEach((id) => selectedIds.add(id))
		}
	}

	function toggle(id: string) {
		if (selectedIds.has(id)) {
			selectedIds.delete(id)
		} else {
			selectedIds.add(id)
		}
	}

	function clear() {
		selectedIds.clear()
	}

	function getSelected<T extends { id: string }>(items: T[]): T[] {
		return items.filter((item) => selectedIds.has(item.id))
	}

	return {
		selectedIds, // Expose rune directly (no getter)
		toggleAll,
		toggle,
		clear,
		getSelected,
	}
}
