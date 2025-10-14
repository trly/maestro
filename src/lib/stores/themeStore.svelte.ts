import { getCurrentWindow } from '@tauri-apps/api/window';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';
import * as ipc from '$lib/ipc';

export type Theme = 'light' | 'dark' | 'auto';

function getSystemTheme(): 'light' | 'dark' {
	if (typeof window !== 'undefined' && window.matchMedia) {
		return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
	}
	return 'dark';
}

let theme = $state<Theme>('auto');
let systemThemeListener: ((e: MediaQueryListEvent) => void) | null = null;
let tauriThemeUnlisten: UnlistenFn | null = null;

async function applyTheme(newTheme: Theme) {
	const effectiveTheme = newTheme === 'auto' ? getSystemTheme() : newTheme;
	
	if (typeof document !== 'undefined') {
		document.documentElement.classList.remove('light', 'dark');
		document.documentElement.classList.add(effectiveTheme);
		console.log('[Theme] Applied theme:', newTheme, 'effective:', effectiveTheme, 'classes:', document.documentElement.className);
	}

	try {
		const appWindow = getCurrentWindow();
		// Pass null to let Tauri auto-track system theme when in auto mode
		await appWindow.setTheme(newTheme === 'auto' ? null : effectiveTheme as 'light' | 'dark');
	} catch (e) {
		console.error('Failed to set Tauri theme:', e);
	}
}

async function setupSystemThemeListener() {
	if (typeof window === 'undefined' || !window.matchMedia) return;

	// Listen to CSS media query changes
	const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
	
	systemThemeListener = (e: MediaQueryListEvent) => {
		console.log('[Theme] CSS media query changed:', e.matches ? 'dark' : 'light', 'current theme:', theme);
		if (theme === 'auto') {
			applyTheme('auto');
		}
	};

	// Use addEventListener if available, otherwise fall back to legacy addListener for WKWebView
	if ('addEventListener' in mediaQuery) {
		mediaQuery.addEventListener('change', systemThemeListener);
	} else if ('addListener' in mediaQuery) {
		// @ts-expect-error: legacy Safari/WKWebView API
		mediaQuery.addListener(systemThemeListener);
	} else {
		console.warn('[Theme] matchMedia has no change listener support');
	}

	// Listen to Tauri's system theme change events
	try {
		tauriThemeUnlisten = await listen<string>('tauri://theme-changed', (event) => {
			console.log('[Theme] Tauri theme changed:', event.payload, 'current theme:', theme);
			if (theme === 'auto') {
				applyTheme('auto');
			}
		});
		console.log('[Theme] Successfully listening to Tauri theme changes');
	} catch (e) {
		console.error('Failed to listen to Tauri theme changes:', e);
	}
}

function cleanupSystemThemeListener() {
	if (systemThemeListener && typeof window !== 'undefined' && window.matchMedia) {
		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		if ('removeEventListener' in mediaQuery) {
			mediaQuery.removeEventListener('change', systemThemeListener);
		} else if ('removeListener' in mediaQuery) {
			// @ts-expect-error: legacy Safari/WKWebView API
			mediaQuery.removeListener(systemThemeListener);
		}
		systemThemeListener = null;
	}

	if (tauriThemeUnlisten) {
		tauriThemeUnlisten();
		tauriThemeUnlisten = null;
	}
}

export const themeStore = {
	get current() {
		return theme;
	},
	async load() {
		const storedTheme = await ipc.getSetting('theme');
		theme = (storedTheme as Theme) || 'auto';
	},
	setTheme: async (newTheme: Theme) => {
		theme = newTheme;
		await ipc.setSetting('theme', newTheme);
		await applyTheme(newTheme);
	},
	init: async () => {
		await themeStore.load();
		await applyTheme(theme);
		await setupSystemThemeListener();
	},
	cleanup: () => {
		cleanupSystemThemeListener();
	}
};
