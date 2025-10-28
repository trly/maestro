export interface DiffItem {
	type: "added" | "removed" | "unchanged" | "modified"
	oldLine?: string
	newLine?: string
	oldLineNumber?: number
	newLineNumber?: number
	segments?: Array<{ added?: boolean; removed?: boolean; value: string }>
}
