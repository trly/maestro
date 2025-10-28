import type { LayoutLoad } from "./$types"

export const prerender = false

export const load: LayoutLoad = ({ url }) => {
	return {
		pathname: url.pathname,
		searchParams: Object.fromEntries(url.searchParams),
	}
}
