import { writable, get } from 'svelte/store';
import { getCurrentWindow } from '@tauri-apps/api/window';

export type Theme = 'light' | 'dark' | 'auto';

const STORAGE_KEY = 'maestro_theme';

function getSystemTheme(): 'light' | 'dark' {
	if (typeof window !== 'undefined' && window.matchMedia) {
		return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light';
	}
	return 'dark';
}

function createThemeStore() {
	const stored = typeof localStorage !== 'undefined' 
		? (localStorage.getItem(STORAGE_KEY) as Theme)
		: null;
	
	const { subscribe, set, update } = writable<Theme>(stored || 'auto');

	let systemThemeListener: ((e: MediaQueryListEvent) => void) | null = null;

	async function applyTheme(theme: Theme) {
		const effectiveTheme = theme === 'auto' ? getSystemTheme() : theme;
		
		if (typeof document !== 'undefined') {
			document.documentElement.classList.remove('light', 'dark');
			document.documentElement.classList.add(effectiveTheme);
		}

		try {
			const appWindow = getCurrentWindow();
			await appWindow.setTheme(effectiveTheme);
		} catch (e) {
			console.error('Failed to set Tauri theme:', e);
		}
	}

	function setupSystemThemeListener() {
		if (typeof window === 'undefined' || !window.matchMedia) return;

		const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
		
		systemThemeListener = (e: MediaQueryListEvent) => {
			const currentTheme = get({ subscribe });
			if (currentTheme === 'auto') {
				applyTheme('auto');
			}
		};

		mediaQuery.addEventListener('change', systemThemeListener);
	}

	function cleanupSystemThemeListener() {
		if (systemThemeListener && typeof window !== 'undefined' && window.matchMedia) {
			const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
			mediaQuery.removeEventListener('change', systemThemeListener);
			systemThemeListener = null;
		}
	}

	return {
		subscribe,
		setTheme: async (theme: Theme) => {
			set(theme);
			if (typeof localStorage !== 'undefined') {
				localStorage.setItem(STORAGE_KEY, theme);
			}
			await applyTheme(theme);
		},
		init: async () => {
			const currentTheme = get({ subscribe });
			await applyTheme(currentTheme);
			setupSystemThemeListener();
		},
		cleanup: () => {
			cleanupSystemThemeListener();
		}
	};
}

export const themeStore = createThemeStore();
