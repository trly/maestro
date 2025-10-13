<script lang="ts">
	import { page } from '$app/stores';
	import ConfirmDialog from '$lib/components/ui/ConfirmDialog.svelte';
	import PromptSidebar from '$lib/components/PromptSidebar.svelte';
	import { onMount, onDestroy } from 'svelte';
	import { subscribeToExecutions, unsubscribeFromExecutions } from '$lib/stores/executionBus';
	import { themeStore } from '$lib/stores/themeStore';
	import { Dialog } from 'bits-ui';
	import '../app.css';

	let { children } = $props();

	let sidebarCollapsed = $state(false);
	let mobileSidebarOpen = $state(false);

	onMount(async () => {
		subscribeToExecutions();
		await themeStore.init();
	});

	onDestroy(() => {
		unsubscribeFromExecutions();
		themeStore.cleanup();
	});
</script>

<div class="@container/app h-screen bg-background flex flex-col">
	<div class="flex-1 flex overflow-hidden min-h-0">
		<aside class="shrink-0 border-r border-border/20 bg-card overflow-hidden transition-all duration-300 flex flex-col @max-md/app:hidden @max-6xl/app:w-12 @6xl/app:{sidebarCollapsed ? 'w-12' : 'w-72 @lg/app:w-80'}">
			<PromptSidebar collapsed={sidebarCollapsed} onToggleCollapse={() => sidebarCollapsed = !sidebarCollapsed} />
		</aside>

		<main class="@container/main flex-1 min-w-0 overflow-hidden flex flex-col">
			{@render children()}
		</main>
	</div>
</div>

<ConfirmDialog />

{#if mobileSidebarOpen}
	<Dialog.Root bind:open={mobileSidebarOpen}>
		<Dialog.Portal>
			<Dialog.Overlay
				class="fixed inset-0 bg-black/40 z-50"
			/>
			<Dialog.Content
				class="fixed inset-y-0 left-0 w-[min(85vw,20rem)] bg-card border-r border-border shadow-xl p-0 z-50"
			>
				<PromptSidebar collapsed={false} onToggleCollapse={() => {}} />
			</Dialog.Content>
		</Dialog.Portal>
	</Dialog.Root>
{/if}
