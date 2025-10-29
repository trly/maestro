import type { ColumnFilters } from "./types"

export function normalizeFilter(value?: string | null): string | undefined {
	return value == null || value === "" || value === "all" ? undefined : value
}

export function normalizeExecutionFilters(filters: ColumnFilters): ColumnFilters {
	const normalized = { ...filters }
	
	if (typeof normalized.repository === "string") {
		normalized.repository = normalized.repository.trim() || undefined
	}
	
	normalized.status = normalizeFilter(normalized.status)
	normalized.validationStatus = normalizeFilter(normalized.validationStatus)
	normalized.ciStatus = normalizeFilter(normalized.ciStatus)
	normalized.changes = normalizeFilter(normalized.changes)
	
	return normalized
}

export function normalizeAnalysisFilters(filters: ColumnFilters): ColumnFilters {
	const normalized = { ...filters }
	
	normalized.type = normalizeFilter(normalized.type)
	normalized.status = normalizeFilter(normalized.status)
	
	return normalized
}
