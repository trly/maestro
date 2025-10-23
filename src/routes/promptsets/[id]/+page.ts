import type { PageLoad } from './$types'

export const prerender = false;

export const load: PageLoad = ({ params, url }) => {
	return {
		promptsetId: params.id,
		revisionParam: url.searchParams.get('revision')
	};
}