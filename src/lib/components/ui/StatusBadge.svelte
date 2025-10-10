<script lang="ts">
	import type { ExecutionStatus, ValidationStatus } from '../../types';

	let {
		status,
		prefix = '',
		animated = false
	}: {
		status: ExecutionStatus | ValidationStatus;
		prefix?: string;
		animated?: boolean;
	} = $props();

	const statusStyles: Record<string, string> = {
		completed: 'bg-success/10 text-success',
		failed: 'bg-destructive/10 text-destructive',
		running: 'bg-accent text-accent-foreground',
		pending: 'bg-muted text-muted-foreground',
		passed: 'bg-success/10 text-success',
		cancelled: 'bg-warning/10 text-warning',
	};

	let displayStatus = $derived(status || 'unknown');
	let className = $derived(statusStyles[displayStatus] || 'bg-muted text-muted-foreground');
	let animationClass = $derived(animated && status === 'running' ? 'animate-pulse' : '');
</script>

<span class={`px-3 py-1.5 rounded-sm text-xs font-bold uppercase tracking-wide ${className} ${animationClass}`}>
	{#if prefix}{prefix} {/if}{displayStatus}
</span>
