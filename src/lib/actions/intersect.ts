export function intersectOnce(
	node: Element,
	opts?: {
		root?: Element | null
		rootMargin?: string
		threshold?: number | number[]
		onEnter?: () => void
	}
) {
	const observer = new IntersectionObserver(
		(entries) => {
			for (const e of entries) {
				if (e.isIntersecting) {
					opts?.onEnter?.()
					observer.unobserve(node)
					observer.disconnect()
					break
				}
			}
		},
		{
			root: opts?.root ?? null,
			rootMargin: opts?.rootMargin ?? "0px",
			threshold: opts?.threshold ?? 0.1,
		}
	)
	observer.observe(node)
	return {
		destroy() {
			observer.disconnect()
		},
	}
}
