<script lang="ts">
	import type { CiStatus } from '../../types';
	import UiTooltip from './UiTooltip.svelte';
	import { CircleCheck, CircleX, LoaderCircle, CircleMinus } from 'lucide-svelte';
	import { openInBrowser } from '$lib/utils/browser';

	let props: {
		ciStatus: CiStatus | null;
		ciUrl?: string | null;
		onRefresh?: () => void;
		isRefreshing?: boolean;
	} = $props();

	let icon = $derived.by(() => {
		if (!props.ciStatus) return null;
		switch (props.ciStatus) {
			case 'pending': return { Icon: LoaderCircle, class: 'text-primary animate-spin', label: 'CI running' };
			case 'passed': return { Icon: CircleCheck, class: 'text-success', label: 'CI passed' };
			case 'failed': return { Icon: CircleX, class: 'text-destructive', label: 'CI failed' };
			case 'skipped': return { Icon: CircleMinus, class: 'text-muted-foreground', label: 'No CI configured for this branch' };
			case 'not_configured': return { Icon: CircleMinus, class: 'text-muted-foreground', label: 'No CI configured' };
			default: return null;
		}
	});

	function handleClick() {
		if (props.ciUrl) {
			openInBrowser(props.ciUrl);
		} else if (props.onRefresh) {
			props.onRefresh();
		}
	}
</script>

{#if icon}
	<UiTooltip content={props.isRefreshing ? "Refreshing..." : props.ciUrl ? `${icon.label} - Click to view` : (props.onRefresh && props.ciStatus !== 'skipped' && props.ciStatus !== 'not_configured') ? `${icon.label} - Click to refresh` : icon.label}>
		{#snippet children({ props: triggerProps })}
			{@const Icon = props.isRefreshing ? LoaderCircle : icon.Icon}
			{@const iconClass = props.isRefreshing ? 'text-primary animate-spin' : icon.class}
			<button
				{...triggerProps}
				onclick={handleClick}
				class="flex items-center hover:opacity-80 transition-opacity disabled:opacity-50 disabled:cursor-not-allowed"
				disabled={props.isRefreshing || (!props.ciUrl && !props.onRefresh)}
			>
				<Icon class={`w-4 h-4 ${iconClass}`} />
			</button>
		{/snippet}
	</UiTooltip>
{/if}
