export function useResizeObserver(
	getElement: () => HTMLElement | null,
	callback: (rect: DOMRectReadOnly) => void
) {
	let observer: ResizeObserver | null = null

	$effect(() => {
		const element = getElement()
		if (!element) return

		observer = new ResizeObserver((entries) => {
			const entry = entries[0]
			if (entry) callback(entry.contentRect)
		})

		observer.observe(element)

		return () => {
			observer?.disconnect()
			observer = null
		}
	})
}
