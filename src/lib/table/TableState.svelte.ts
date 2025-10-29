import type { SortSpec, ColumnFilters } from "./types"

export class TableState<T> {
	items = $state<T[]>([])
	filters = $state<ColumnFilters>({})
	sort = $state<SortSpec>({ key: "createdAt", dir: "desc" })

	itemsById = $derived.by(() => {
		const map = new Map<string, T>()
		for (const item of this.items) {
			map.set(this.getItemId(item), item)
		}
		return map
	})

	filteredItems = $derived.by(() => this.items.filter((item) => this.filterFn(this.filters, item)))

	sortedItems = $derived.by(() => {
		const items = [...this.filteredItems]
		const dir = this.sort.dir === "asc" ? 1 : -1
		const comparator = this.comparators[this.sort.key]

		if (!comparator) {
			return items
		}

		items.sort((a, b) => {
			const result = comparator(a, b)
			return result === 0
				? dir * this.collator.compare(this.getItemId(a), this.getItemId(b))
				: dir * result
		})

		return items
	})

	sortedIds = $derived(this.sortedItems.map((item) => this.getItemId(item)))

	constructor(
		private getItemId: (item: T) => string,
		private filterFn: (filters: ColumnFilters, item: T) => boolean,
		private comparators: Record<string, (a: T, b: T) => number>,
		private normalizeFilters?: (filters: ColumnFilters) => ColumnFilters,
		defaultSort?: SortSpec
	) {
		if (defaultSort) {
			this.sort = defaultSort
		}
	}

	private collator = new Intl.Collator(undefined, {
		numeric: true,
		sensitivity: "base",
	})

	setFilters(nextFilters: ColumnFilters) {
		const normalized = this.normalizeFilters ? this.normalizeFilters(nextFilters) : nextFilters

		for (const key of Object.keys(this.filters)) {
			delete this.filters[key]
		}
		Object.assign(this.filters, normalized)
	}

	toggleSort(key: string) {
		if (this.sort.key === key) {
			this.sort.dir = this.sort.dir === "asc" ? "desc" : "asc"
		} else {
			this.sort = { key, dir: "asc" }
		}
	}

	clearFilters() {
		for (const key of Object.keys(this.filters)) {
			delete this.filters[key]
		}
	}
}
