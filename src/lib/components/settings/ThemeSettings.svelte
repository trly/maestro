<script lang="ts">
	import { themeStore, type Theme } from "$lib/stores/themeStore.svelte"

	let currentTheme = $derived(themeStore.current)

	async function setTheme(theme: Theme) {
		await themeStore.setTheme(theme)
	}
</script>

<div>
	<h3 class="text-lg font-semibold mb-4">Appearance</h3>
	<p class="text-sm text-muted-foreground mb-6">Choose your preferred color theme</p>

	<div>
		<div class="block text-sm font-medium mb-3">Theme</div>
		<div class="flex gap-3">
			<button
				type="button"
				onclick={() => setTheme("light")}
				class="flex-1 px-4 py-3 border rounded-md transition-colors {currentTheme === 'light'
					? 'bg-primary text-primary-foreground border-primary'
					: 'bg-card hover:bg-muted border-border/30'}"
			>
				Light
			</button>
			<button
				type="button"
				onclick={() => setTheme("dark")}
				class="flex-1 px-4 py-3 border rounded-md transition-colors {currentTheme === 'dark'
					? 'bg-primary text-primary-foreground border-primary'
					: 'bg-card hover:bg-muted border-border/30'}"
			>
				Dark
			</button>
			<button
				type="button"
				onclick={() => setTheme("auto")}
				class="flex-1 px-4 py-3 border rounded-md transition-colors {currentTheme === 'auto'
					? 'bg-primary text-primary-foreground border-primary'
					: 'bg-card hover:bg-muted border-border/30'}"
			>
				Auto
			</button>
		</div>
		<p class="text-xs text-muted-foreground mt-2">
			{currentTheme === "auto"
				? "Automatically matches your system theme"
				: `Using ${currentTheme} theme`}
		</p>
	</div>
</div>
