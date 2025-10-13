<script lang="ts">
	import { Tooltip } from 'bits-ui';

	let {
		content,
		side = 'top',
		delayMs = 300,
		children
	}: {
		content: string;
		side?: 'top' | 'bottom' | 'left' | 'right';
		delayMs?: number;
		children: import('svelte').Snippet<[{ props: Record<string, unknown> }]>;
	} = $props();
</script>

<Tooltip.Provider>
	<Tooltip.Root delayDuration={delayMs}>
		<Tooltip.Trigger>
			{#snippet child({ props })}
				{@render children({ props })}
			{/snippet}
		</Tooltip.Trigger>
		<Tooltip.Portal>
		<Tooltip.Content
		 {side}
		 sideOffset={5}
		  class="z-50 px-3 py-2 text-xs font-medium text-white bg-gray-900 rounded-lg shadow-lg border-0 max-w-xs"
		>
		  {content}
		</Tooltip.Content>
	</Tooltip.Portal>
	</Tooltip.Root>
</Tooltip.Provider>
