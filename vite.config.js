import { sveltekit } from '@sveltejs/kit/vite';
import { defineConfig } from 'vite';

// Detect if running in Tauri mode (no proxy needed)
const isTauri = process.env.TAURI_ENV_PLATFORM !== undefined;

export default defineConfig({
	plugins: [sveltekit()],

	server: {
		strictPort: false,
		// Only proxy API requests in non-Tauri mode (when using Bun server)
		proxy: isTauri ? undefined : {
			'/api': {
				target: 'http://localhost:3000',
				changeOrigin: true
			}
		}
	},

	// Prevent Vite from clearing the screen in Tauri mode
	clearScreen: false
});
