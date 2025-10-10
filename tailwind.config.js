/** @type {import('tailwindcss').Config} */
export default {
	content: ['./src/**/*.{html,js,svelte,ts}'],
	darkMode: 'class',
	theme: {
		extend: {
			colors: {
				gruvbox: {
					light: {
						bg0: '#f9f5d7',
						bg1: '#ebdbb2',
						bg2: '#d5c4a1',
						bg3: '#bdae93',
						bg4: '#a89984',
						fg0: '#1d2021',
						fg1: '#282828',
						fg2: '#3c3836',
						fg3: '#504945',
						fg4: '#665c54',
						red: '#cc241d',
						green: '#98971a',
						yellow: '#d79921',
						blue: '#458588',
						purple: '#b16286',
						aqua: '#689d6a',
						orange: '#d65d0e',
						gray: '#928374'
					},
					dark: {
						bg0: '#1d2021',
						bg1: '#282828',
						bg2: '#3c3836',
						bg3: '#504945',
						bg4: '#665c54',
						fg0: '#f9f5d7',
						fg1: '#ebdbb2',
						fg2: '#d5c4a1',
						fg3: '#bdae93',
						fg4: '#928374',
						red: '#fb4934',
						green: '#b8bb26',
						yellow: '#fabd2f',
						blue: '#83a598',
						purple: '#d3869b',
						aqua: '#8ec07c',
						orange: '#fe8019',
						gray: '#928374'
					}
				},
				background: 'var(--background)',
				foreground: 'var(--foreground)',
				card: 'var(--card)',
				'card-foreground': 'var(--card-foreground)',
				border: 'var(--border)',
				accent: 'var(--accent)',
				'accent-foreground': 'var(--accent-foreground)',
				muted: 'var(--muted)',
				'muted-foreground': 'var(--muted-foreground)',
				primary: 'var(--primary)',
				'primary-foreground': 'var(--primary-foreground)',
				destructive: 'var(--destructive)',
				'destructive-foreground': 'var(--destructive-foreground)'
			}
		}
	},
	plugins: [
		require('@tailwindcss/container-queries')
	]
};
