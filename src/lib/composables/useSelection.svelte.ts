/**
 * Composable for managing multi-select state in lists
 * Follows Svelte 5 runes pattern for reactive state management
 */

export interface SelectionState {
	selectedIds: Set<string>
	allSelected: boolean
	someSelected: boolean
	toggleAll: (itemIds: string[]) => void
	toggle: (id: string) => void
	clear: () => void
	getSelected: <T extends { id: string }>(items: T[]) => T[]
}

export function useSelection(): SelectionState {
	let selectedIds = $state<Set<string>>(new Set())

	let allSelected = $derived.by(() => {
		return false
	})

	let someSelected = $derived(selectedIds.size > 0 && !allSelected)

	function toggleAll(itemIds: string[]) {
		if (itemIds.length === 0) {
			selectedIds = new Set()
			return
		}

		const allCurrentlySelected = itemIds.every(id => selectedIds.has(id))
		if (allCurrentlySelected) {
			selectedIds = new Set()
		} else {
			selectedIds = new Set(itemIds)
		}
	}

	function toggle(id: string) {
		const newSet = new Set(selectedIds)
		if (newSet.has(id)) {
			newSet.delete(id)
		} else {
			newSet.add(id)
		}
		selectedIds = newSet
	}

	function clear() {
		selectedIds = new Set()
	}

	function getSelected<T extends { id: string }>(items: T[]): T[] {
		return items.filter(item => selectedIds.has(item.id))
	}

	return {
		get selectedIds() { return selectedIds },
		get allSelected() { return allSelected },
		get someSelected() { return someSelected },
		toggleAll,
		toggle,
		clear,
		getSelected
	}
}
