import type { AnalysisStatus, AnalysisType } from "$lib/types"

export interface SortSpec {
	key: "type" | "status" | "createdAt" | "completedAt"
	dir: "asc" | "desc"
}

export interface ColumnFilters {
	type?: AnalysisType
	status?: AnalysisStatus
}
