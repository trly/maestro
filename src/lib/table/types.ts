export type SortDirection = "asc" | "desc"

export interface SortSpec {
	key: string
	dir: SortDirection
}

export type ColumnFilters = Record<string, any>
