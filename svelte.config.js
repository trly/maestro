import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
	preprocess: vitePreprocess(),

	kit: {
		adapter: adapter({
			pages: 'dist',
			assets: 'dist',
			fallback: 'index.html', // SPA mode
			precompress: false,
			strict: false
		}),

		prerender: {
			handleMissingId: 'warn',
			handleHttpError: 'warn',
			entries: ['/'],
			crawl: false
		},

		alias: {
			$lib: './src/lib',
			'$lib/*': './src/lib/*'
		}
	}
};

export default config;
