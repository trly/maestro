<script lang="ts">
	import ConfirmDialog from "$lib/components/ui/ConfirmDialog.svelte"
	import PromptSidebar from "$lib/components/PromptSidebar.svelte"
	import { onMount, onDestroy } from "svelte"
	import { subscribeToExecutions, unsubscribeFromExecutions } from "$lib/stores/executionBus"
	import { themeStore } from "$lib/stores/themeStore.svelte"
	import { settingsStore } from "$lib/stores/settingsStore"
	import { Dialog } from "bits-ui"
	import { PaneGroup, Pane, PaneResizer } from "paneforge"
	import "../app.css"

	let { data, children } = $props()

	let sidebarCollapsed = $state(false)
	let mobileSidebarOpen = $state(false)
	let settings = $state<any>({})

	$effect(() => {
		settingsStore.subscribe((s) => (settings = s))
	})

	function handleSidebarResize(sizes: number[]) {
		if (sizes[0] !== undefined) {
			settingsStore.updateUI({ sidebarPct: sizes[0] })
		}
	}

	onMount(async () => {
		subscribeToExecutions()
		await themeStore.init()
		await settingsStore.load()
	})

	onDestroy(() => {
		unsubscribeFromExecutions()
		themeStore.cleanup()
	})
</script>

<div class="@container/app h-screen max-h-screen bg-background flex flex-col">
	<div class="flex-1 flex overflow-hidden min-h-0 max-h-screen @max-md/app:block">
		<div class="@md/app:hidden h-full">
			<main class="@container/main h-full overflow-hidden flex flex-col">
				{@render children()}
			</main>
		</div>

		<div class="hidden @md/app:flex @md/app:h-full @md/app:flex-1">
			<PaneGroup direction="horizontal" onLayoutChange={handleSidebarResize}>
				<Pane
					defaultSize={sidebarCollapsed ? 3 : (settings.ui?.sidebarPct ?? 20)}
					minSize={sidebarCollapsed ? 3 : 12}
					maxSize={sidebarCollapsed ? 3 : 40}
				>
					<aside class="h-full border-r border-border/20 bg-card overflow-hidden flex flex-col">
						<PromptSidebar
							collapsed={sidebarCollapsed}
							onToggleCollapse={() => (sidebarCollapsed = !sidebarCollapsed)}
							pathname={data.pathname}
							searchParams={data.searchParams}
						/>
					</aside>
				</Pane>
				{#if !sidebarCollapsed}
					<PaneResizer
						class="w-1 bg-border/40 hover:bg-primary/40 transition-colors cursor-col-resize"
					/>
				{/if}
				<Pane>
					<main class="@container/main h-full overflow-hidden flex flex-col">
						{@render children()}
					</main>
				</Pane>
			</PaneGroup>
		</div>
	</div>
</div>

<ConfirmDialog />

{#if mobileSidebarOpen}
	<Dialog.Root bind:open={mobileSidebarOpen}>
		<Dialog.Portal>
			<Dialog.Overlay class="fixed inset-0 bg-black/40 z-50" />
			<Dialog.Content
				class="fixed inset-y-0 left-0 w-[min(85vw,20rem)] bg-card border-r border-border shadow-xl p-0 z-50"
			>
				<PromptSidebar
					collapsed={false}
					onToggleCollapse={() => {}}
					pathname={data.pathname}
					searchParams={data.searchParams}
				/>
			</Dialog.Content>
		</Dialog.Portal>
	</Dialog.Root>
{/if}
