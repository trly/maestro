import { diffLines, diffWords, parsePatch } from 'diff'
import type { DiffItem } from './types/diff'

export type DiffSegment = {
	value: string
	added?: boolean
	removed?: boolean
}

export function processTextDiff(oldText: string, newText: string): DiffItem[] {
	const changes = diffLines(oldText, newText)
	const result: DiffItem[] = []
	let i = 0

	while (i < changes.length) {
		const change = changes[i]

		if (change.removed && i + 1 < changes.length && changes[i + 1].added) {
			const oldLines = change.value.split('\n').filter((_, idx, arr) => idx < arr.length - 1 || _)
			const newLines = changes[i + 1].value.split('\n').filter((_, idx, arr) => idx < arr.length - 1 || _)

			const maxLen = Math.max(oldLines.length, newLines.length)
			for (let j = 0; j < maxLen; j++) {
				const oldLine = oldLines[j] || ''
				const newLine = newLines[j] || ''
				result.push({
					type: 'modified',
					oldLine,
					newLine,
					segments: diffWords(oldLine, newLine)
				})
			}
			i += 2
		} else if (change.added) {
			const lines = change.value.split('\n').filter((_, idx, arr) => idx < arr.length - 1 || _)
			lines.forEach((line) => result.push({ type: 'added', newLine: line }))
			i++
		} else if (change.removed) {
			const lines = change.value.split('\n').filter((_, idx, arr) => idx < arr.length - 1 || _)
			lines.forEach((line) => result.push({ type: 'removed', oldLine: line }))
			i++
		} else {
			const lines = change.value.split('\n').filter((_, idx, arr) => idx < arr.length - 1 || _)
			lines.forEach((line) => result.push({ type: 'unchanged', oldLine: line, newLine: line }))
			i++
		}
	}

	return result
}

export function processPatchDiff(patchText: string): DiffItem[] {
	if (!patchText.trim()) {
		return []
	}

	const parsed = parsePatch(patchText)
	if (!parsed || parsed.length === 0) {
		return []
	}

	const result: DiffItem[] = []
	let oldLineNum = 1
	let newLineNum = 1

	for (const file of parsed) {
		for (const hunk of file.hunks) {
			oldLineNum = hunk.oldStart
			newLineNum = hunk.newStart

			let i = 0
			while (i < hunk.lines.length) {
				const line = hunk.lines[i]
				const prefix = line[0]
				const content = line.slice(1)

				if (prefix === '-' && i + 1 < hunk.lines.length && hunk.lines[i + 1][0] === '+') {
					const oldContent = content
					const newContent = hunk.lines[i + 1].slice(1)

					result.push({
						type: 'modified',
						oldLine: oldContent,
						newLine: newContent,
						segments: diffWords(oldContent, newContent),
						oldLineNumber: oldLineNum,
						newLineNumber: newLineNum
					})

					oldLineNum++
					newLineNum++
					i += 2
				} else if (prefix === '+') {
					result.push({
						type: 'added',
						newLine: content,
						newLineNumber: newLineNum
					})
					newLineNum++
					i++
				} else if (prefix === '-') {
					result.push({
						type: 'removed',
						oldLine: content,
						oldLineNumber: oldLineNum
					})
					oldLineNum++
					i++
				} else {
					result.push({
						type: 'unchanged',
						oldLine: content,
						newLine: content,
						oldLineNumber: oldLineNum,
						newLineNumber: newLineNum
					})
					oldLineNum++
					newLineNum++
					i++
				}
			}
		}
	}

	return result
}
