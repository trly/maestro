import type { PageLoad } from "./$types"

export const prerender = false

export const load: PageLoad = ({ url }) => {
	return {
		promptsetParam: url.searchParams.get("promptset"),
	}
}
