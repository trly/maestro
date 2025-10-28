import type { Execution } from "$lib/types"

export type ColumnFilters = {
	repository?: string
	status?: Execution["status"] | "all"
	validationStatus?: Execution["validationStatus"] | "all"
	ciStatus?: Execution["ciStatus"] | "all"
	changes?: "has-changes" | "no-changes" | "all"
}

export type SortKey =
	| "repo"
	| "status"
	| "validation"
	| "ci"
	| "commit"
	| "diff"
	| "createdAt"
	| "completedAt"

export type SortDir = "asc" | "desc"

export interface SortSpec {
	key: SortKey
	dir: SortDir
}
